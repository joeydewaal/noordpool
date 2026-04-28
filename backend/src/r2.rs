//! Avatar storage backend.
//!
//! In production this signs PUT URLs against Cloudflare R2 (S3-compatible)
//! via `aws-sdk-s3`; the browser uploads directly and reads land on the R2
//! public URL with no backend involvement. In dev/tests we use [`Backend::Local`]
//! which signs URLs pointing at our own internal upload route and writes the
//! bytes to a local directory served by `ServeDir` at `/avatars`.

use std::{path::PathBuf, sync::Arc, time::Duration};

use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
    presigning::PresigningConfig,
};
use hmac::{Hmac, Mac};
use jiff::Timestamp;
use sha2::Sha256;

const PRESIGN_EXPIRES_SECS: u64 = 300;

#[derive(Debug, Clone)]
pub struct R2Config {
    pub account_id: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    /// Public base URL for objects (e.g. `https://pub-xxx.r2.dev`).
    pub public_url: String,
}

#[derive(Debug, Clone)]
pub struct LocalConfig {
    /// Filesystem dir where uploaded bytes land.
    pub dir: PathBuf,
    /// External base URL of this API (e.g. `http://localhost:3000`); the local
    /// upload route lives at `<api_base>/api/_avatars/upload/<key>` and the
    /// public URL points to `<api_base>/avatars/<key>`.
    pub api_base: String,
    /// HMAC key used to sign upload URL tokens. Reuse the JWT secret.
    pub signing_key: String,
}

#[derive(Clone)]
pub enum Backend {
    R2 {
        client: Client,
        bucket: Arc<String>,
        public_url: Arc<String>,
    },
    Local(Arc<LocalConfig>),
}

pub struct Presigned {
    pub upload_url: String,
    pub public_url: String,
}

impl Backend {
    pub fn r2(config: &R2Config) -> Self {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", config.account_id);
        let credentials = Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "static",
        );
        let s3_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .region(Region::new("auto"))
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();
        Backend::R2 {
            client: Client::from_conf(s3_config),
            bucket: Arc::new(config.bucket.clone()),
            public_url: Arc::new(config.public_url.trim_end_matches('/').to_string()),
        }
    }

    pub fn local(config: LocalConfig) -> std::io::Result<Self> {
        std::fs::create_dir_all(&config.dir)?;
        Ok(Backend::Local(Arc::new(config)))
    }

    /// Presigned PUT URL for `key`. The browser uploads directly. The same
    /// `Content-Type` must be sent with the upload.
    pub async fn presign_put(&self, key: &str, content_type: &str) -> Result<Presigned, String> {
        match self {
            Backend::R2 {
                client,
                bucket,
                public_url,
            } => {
                let presign_config =
                    PresigningConfig::expires_in(Duration::from_secs(PRESIGN_EXPIRES_SECS))
                        .map_err(|e| e.to_string())?;
                let req = client
                    .put_object()
                    .bucket(bucket.as_str())
                    .key(key)
                    .content_type(content_type)
                    .presigned(presign_config)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(Presigned {
                    upload_url: req.uri().to_string(),
                    public_url: format!("{}/{}", public_url, key),
                })
            }
            Backend::Local(local) => {
                let exp = Timestamp::now().as_second() + PRESIGN_EXPIRES_SECS as i64;
                let token = sign_local_upload(&local.signing_key, key, exp);
                let api_base = local.api_base.trim_end_matches('/');
                Ok(Presigned {
                    upload_url: format!(
                        "{api_base}/api/_avatars/upload/{key}?exp={exp}&sig={token}"
                    ),
                    public_url: format!("{api_base}/avatars/{key}"),
                })
            }
        }
    }

    /// Best-effort delete by key. Errors are logged, not propagated.
    pub async fn delete(&self, key: &str) {
        match self {
            Backend::R2 { client, bucket, .. } => {
                if let Err(err) = client
                    .delete_object()
                    .bucket(bucket.as_str())
                    .key(key)
                    .send()
                    .await
                {
                    tracing::warn!(error = %err, key, "r2: delete failed");
                }
            }
            Backend::Local(local) => {
                let path = local.dir.join(key);
                if path.exists()
                    && let Err(err) = tokio::fs::remove_file(&path).await
                {
                    tracing::warn!(error = %err, ?path, "local: delete failed");
                }
            }
        }
    }

    pub fn local_config(&self) -> Option<&LocalConfig> {
        match self {
            Backend::Local(c) => Some(c.as_ref()),
            _ => None,
        }
    }
}

pub fn sign_local_upload(secret: &str, key: &str, exp: i64) -> String {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(key.as_bytes());
    mac.update(b"|");
    mac.update(exp.to_string().as_bytes());
    let bytes = mac.finalize().into_bytes();
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

pub fn verify_local_upload(secret: &str, key: &str, exp: i64, sig: &str) -> bool {
    if Timestamp::now().as_second() > exp {
        return false;
    }
    let expected = sign_local_upload(secret, key, exp);
    expected.len() == sig.len() && expected.bytes().zip(sig.bytes()).all(|(a, b)| a == b)
}
