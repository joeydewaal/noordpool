pub mod avatar;
pub mod handlers;

use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list))
        .route("/me/avatar", post(avatar::upload).delete(avatar::delete))
        .route("/{id}", patch(handlers::update))
}
