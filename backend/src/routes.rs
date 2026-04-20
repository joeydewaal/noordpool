use axum::Router;
use axum_security::oidc::OidcExt as _;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services::ServeDir};

use crate::{app_state::AppState, auth, events, games, lineup, players, push, stats, teams, users};

pub fn app(state: AppState) -> Router {
    let mut app = Router::new()
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
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(state)
}
