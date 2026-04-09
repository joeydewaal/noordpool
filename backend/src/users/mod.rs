pub mod handlers;

use axum::{Router, routing::patch};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/{id}", patch(handlers::update))
}
