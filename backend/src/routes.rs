use axum::Router;
use tower_http::cors::CorsLayer;

use crate::{app_state::AppState, auth, events, matches, players, stats};

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/players", players::router())
        .nest("/api/matches", matches::router())
        .nest("/api/matches/{match_id}/events", events::router())
        .nest("/api/stats", stats::router())
        .layer(state.jwt.clone())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
