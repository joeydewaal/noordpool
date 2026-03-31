pub mod app_state;
pub mod auth;
pub mod config;
pub mod error;
pub mod events;
pub mod games;
pub mod import;
pub mod models;
pub mod players;
pub mod routes;
pub mod stats;

pub type Result<T, E = error::AppError> = ::std::result::Result<T, E>;
