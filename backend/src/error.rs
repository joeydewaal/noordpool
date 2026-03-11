use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl AppError {
    pub fn unauthorized(err: impl Into<String>) -> Self {
        Self::Unauthorized(err.into())
    }

    pub fn not_found(err: impl Into<String>) -> Self {
        Self::NotFound(err.into())
    }

    pub fn internal(err: impl Display) -> Self {
        Self::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        let body = axum::Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<toasty::Error> for AppError {
    fn from(value: toasty::Error) -> Self {
        if value.is_record_not_found() {
            Self::NotFound("Record not found".into())
        } else {
            Self::Internal(value.to_string())
        }
    }
}
