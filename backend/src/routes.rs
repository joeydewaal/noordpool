use axum::{Router, routing::get};
use axum_security::oidc::OidcExt as _;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services::ServeDir};

use crate::{app_state::AppState, auth, events, games, lineup, players, push, stats, teams, users};

async fn health() -> &'static str {
    "ok"
}

pub fn app(state: AppState) -> Router {
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

    // In local-storage mode, serve uploaded avatars from disk and accept
    // browser uploads at the signed internal route.
    if let Some(local) = state.r2.local_config() {
        let avatars = ServeDir::new(&local.dir);
        app = app
            .nest_service("/avatars", avatars)
            .merge(users::avatar::local_upload_router());
    }

    app.layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(state)
}
