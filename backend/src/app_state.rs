use std::sync::Arc;

use axum::extract::FromRef;
use axum_security::{jwt::JwtContext, oidc::OidcContext};
use toasty::Db;

use crate::auth::{claims::Claims, google::GoogleHandler};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub google_oidc: Option<OidcContext<GoogleHandler>>,
    pub vapid: Option<Arc<VapidConfig>>,
}

/// Loaded once at startup. If absent, push endpoints return 503.
pub struct VapidConfig {
    /// Base64url-encoded P-256 public key (raw, uncompressed, 65 bytes).
    /// Sent verbatim to the browser for `pushManager.subscribe`.
    pub public_key: String,
    /// Base64url-encoded P-256 private key (32 bytes).
    pub private_key: String,
    /// `mailto:` or `https:` URI per RFC 8292.
    pub subject: String,
}

impl FromRef<AppState> for JwtContext<Claims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
