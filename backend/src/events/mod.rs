pub mod handlers;

use axum::{Router, routing::{delete, get}};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route("/{event_id}", delete(handlers::delete))
}
