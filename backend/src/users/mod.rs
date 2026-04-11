pub mod handlers;

use axum::{
    Router,
    routing::{get, patch},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list))
        .route("/{id}", patch(handlers::update))
}
