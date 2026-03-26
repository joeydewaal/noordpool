mod app_state;
mod auth;
mod config;
mod error;
mod events;
mod games;
mod import;
mod json;
mod models;
mod players;
mod routes;
mod stats;

use app_state::AppState;
use axum_security::{jwt::JwtContext, oidc::OidcContext};
use config::Config;
use tokio::net::TcpListener;

use crate::models::build_db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env();

    let mut db = build_db().connect(&config.database_url).await?;
    let _ = db.push_schema().await;
    models::init_db(&mut db).await?;

    let db = {
        let fresh = db.clone();
        drop(db);
        fresh
    };

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

    let state = AppState {
        db,
        jwt,
        google_oidc,
    };

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, routes::app(state)).await?;

    Ok(())
}
