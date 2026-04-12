pub mod handlers;

use axum::{Router, routing::get};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handlers::list).post(handlers::create))
}
