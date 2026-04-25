pub mod handlers;
pub mod live;
pub mod live_ws;

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
        .route("/{id}/players", get(handlers::game_players))
        .route("/{id}/ws", get(live_ws::ws_live))
        .route("/{id}/live/score", post(live::adjust_score))
}
