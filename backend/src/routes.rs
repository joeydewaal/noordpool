use axum::Router;
use tower_http::cors::CorsLayer;

use crate::{app_state::AppState, auth};

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/api/auth", auth::router())
        .layer(state.jwt.clone())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
