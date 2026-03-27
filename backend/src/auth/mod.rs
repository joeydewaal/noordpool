pub mod claims;
pub mod google;
pub mod handlers;
pub mod password;

use axum::{
    Router,
    routing::{get, post},
};


use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
        .route("/find-player", get(handlers::find_player))
}
