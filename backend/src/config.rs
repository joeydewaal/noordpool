use crate::r2::R2Config;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_url: Option<String>,
    pub frontend_url: Option<String>,
    pub vapid_public_key: Option<String>,
    pub vapid_private_key: Option<String>,
    pub vapid_subject: Option<String>,
    pub admin_password: String,
    pub avatar_dir: String,
    pub api_base_url: String,
    pub r2: Option<R2Config>,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite::memory:".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-me".into()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            google_redirect_url: std::env::var("GOOGLE_REDIRECT_URL").ok(),
            frontend_url: std::env::var("FRONTEND_URL").ok(),
            vapid_public_key: std::env::var("VAPID_PUBLIC_KEY").ok(),
            vapid_private_key: std::env::var("VAPID_PRIVATE_KEY").ok(),
            vapid_subject: std::env::var("VAPID_SUBJECT").ok(),
            admin_password: std::env::var("ADMIN_PASSWORD")
                .unwrap_or_else(|_| "Admin123".to_string()),
            avatar_dir: std::env::var("AVATAR_DIR").unwrap_or_else(|_| "./avatar-data".to_string()),
            api_base_url: std::env::var("PUBLIC_API_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            r2: build_r2_config(),
        }
    }
}

fn build_r2_config() -> Option<R2Config> {
    Some(R2Config {
        account_id: std::env::var("R2_ACCOUNT_ID").ok()?,
        bucket: std::env::var("R2_BUCKET").ok()?,
        access_key_id: std::env::var("R2_ACCESS_KEY_ID").ok()?,
        secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY").ok()?,
        public_url: std::env::var("R2_PUBLIC_URL").ok()?,
    })
}
