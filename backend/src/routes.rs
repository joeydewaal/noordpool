use axum::{Router, http::HeaderValue, routing::get};
use axum_security::oidc::OidcExt as _;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::{app_state::AppState, auth, events, games, lineup, players, push, stats, teams, users};

async fn health() -> &'static str {
    "ok"
}

pub fn app(state: AppState, allowed_origins: Vec<String>) -> Router {
    let mut app = Router::new()
        .route("/health", get(health))
        .nest("/api/auth", auth::router())
        .nest("/api/players", players::router())
        .nest("/api/users", users::router())
        .nest("/api/teams", teams::router())
        .nest("/api/games", games::router())
        .nest("/api/games/{game_id}/events", events::router())
        .nest("/api/games/{game_id}/lineup", lineup::router())
        .nest("/api/stats", stats::router())
        .nest("/api/push", push::router())
        .layer(state.jwt.clone());

    if let Some(google_oidc) = state.google_oidc.clone() {
        app = app.with_oidc(google_oidc);
    }

    let avatars = ServeDir::new(state.avatar_dir.as_path());
    app.nest_service("/avatars", avatars)
        .layer(cors_layer(&allowed_origins))
        .layer(CompressionLayer::new())
        .with_state(state)
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    if origins.is_empty() {
        return CorsLayer::permissive();
    }
    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(parsed)
        .allow_methods(Any)
        .allow_headers(Any)
}
