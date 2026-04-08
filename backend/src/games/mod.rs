pub mod handlers;
pub mod live;

use axum::{
    Router,
    routing::{get, post},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route("/summary", get(handlers::summary))
        .route("/upcoming", get(handlers::upcoming))
        .route("/recent", get(handlers::recent))
        .route(
            "/{id}",
            get(handlers::get_one)
                .put(handlers::update)
                .delete(handlers::delete),
        )
        .route("/{id}/live", get(live::poll_live))
        .route(
            "/{id}/live/opponent_score",
            post(live::adjust_opponent_score),
        )
}
