mod app_state;
mod auth;
mod config;
mod error;
mod events;
mod games;
mod import;
mod models;
mod players;
mod push;
mod routes;
mod stats;
mod users;

use std::sync::Arc;

use app_state::{AppState, VapidConfig};
use axum_security::{jwt::JwtContext, oidc::OidcContext};
use config::Config;

use crate::models::create_db;

pub type Result<T, E = error::AppError> = ::std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,noordpool_backend=debug".into()),
        )
        .init();

    let config = Config::from_env();

    let db = create_db(&config).await?;

    let jwt = JwtContext::builder()
        .jwt_secret(&config.jwt_secret)
        .build::<auth::claims::Claims>();

    let google_oidc = if let (Some(client_id), Some(client_secret), Some(redirect_url)) = (
        &config.google_client_id,
        &config.google_client_secret,
        &config.google_redirect_url,
    ) {
        let frontend_url = config
            .frontend_url
            .clone()
            .unwrap_or_else(|| "http://localhost:3000".into());

        let handler = auth::google::GoogleHandler {
            db: db.clone(),
            jwt: jwt.clone(),
            frontend_url,
        };

        let oidc = OidcContext::builder("google")
            .client_id(client_id)
            .client_secret(client_secret)
            .redirect_url(redirect_url)
            .login_path("/api/auth/google/login")
            .scopes(&["openid", "email", "profile"])
            .use_dev_cookies(true)
            .build(handler);

        tracing::info!("Google OIDC enabled");
        Some(oidc)
    } else {
        None
    };

    let vapid = match (
        config.vapid_public_key.clone(),
        config.vapid_private_key.clone(),
        config.vapid_subject.clone(),
    ) {
        (Some(public_key), Some(private_key), Some(subject)) => {
            tracing::info!("Web Push enabled (VAPID configured)");
            Some(Arc::new(VapidConfig {
                public_key,
                private_key,
                subject,
            }))
        }
        _ => {
            tracing::info!("Web Push disabled (VAPID not configured)");
            None
        }
    };

    let state = AppState {
        db,
        jwt,
        google_oidc,
        vapid,
    };

    let app = routes::app(state);

    #[cfg(feature = "prod")]
    lambda_http::run(app).await.unwrap();

    #[cfg(not(feature = "prod"))]
    {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
        tracing::info!("listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await?;
    }

    Ok(())
}
