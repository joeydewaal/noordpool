pub mod avatar;
pub mod handlers;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, patch, post},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list))
        .route(
            "/me/avatar",
            post(avatar::upload)
                .delete(avatar::delete)
                .layer(DefaultBodyLimit::max(avatar::MAX_UPLOAD_BYTES)),
        )
        .route("/{id}", patch(handlers::update))
}
