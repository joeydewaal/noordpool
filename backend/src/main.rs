mod app_state;
mod auth;
mod config;
mod error;
mod json;
mod models;
mod routes;

use app_state::AppState;
use axum_security::jwt::JwtContext;
use config::Config;
use models::{User, UserRole};
use tokio::net::TcpListener;
use toasty::Db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env();

    let mut builder = Db::builder();
    builder.register::<User>();
    builder.register::<UserRole>();
    let mut db = builder.connect(&config.database_url).await?;
    db.push_schema().await?;

    let jwt = JwtContext::builder()
        .jwt_secret(&config.jwt_secret)
        .build::<auth::claims::Claims>();

    let state = AppState { db, jwt };

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, routes::app(state)).await?;

    Ok(())
}
