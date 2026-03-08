mod app_state;
mod auth;
mod config;
mod error;
mod events;
mod json;
mod matches;
mod models;
mod players;
mod routes;
mod stats;

use app_state::AppState;
use axum_security::jwt::JwtContext;
use config::Config;
use models::{
    EventType, Game, HomeAway, MatchEvent, MatchStatus, Player, Position, Role, User, UserRole,
};
use toasty::Db;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env();

    let mut builder = Db::builder();
    builder.register::<User>();
    builder.register::<UserRole>();
    builder.register::<Role>();
    builder.register::<Player>();
    builder.register::<Position>();
    builder.register::<Game>();
    builder.register::<MatchStatus>();
    builder.register::<HomeAway>();
    builder.register::<MatchEvent>();
    builder.register::<EventType>();
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
