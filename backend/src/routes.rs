use axum::Router;
use axum_security::oidc::OidcExt as _;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{app_state::AppState, auth, events, games, players, push, stats, teams, users};

pub fn app(state: AppState) -> Router {
    let mut app = Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/players", players::router())
        .nest("/api/users", users::router())
        .nest("/api/teams", teams::router())
        .nest("/api/games", games::router())
        .nest("/api/games/{game_id}/events", events::router())
        .nest("/api/stats", stats::router())
        .nest("/api/push", push::router())
        .layer(state.jwt.clone());

    if let Some(google_oidc) = state.google_oidc.clone() {
        app = app.with_oidc(google_oidc);
    }

    let avatars = ServeDir::new(state.avatar_dir.as_path());
    app.nest_service("/avatars", avatars)
        .layer(CorsLayer::permissive())
        .with_state(state)
}
