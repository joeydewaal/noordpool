use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl AppError {
    pub fn unauthorized(err: impl Into<String>) -> Self {
        Self::Unauthorized(err.into())
    }

    #[allow(unused)]
    pub fn not_found(err: impl Into<String>) -> Self {
        Self::NotFound(err.into())
    }

    pub fn internal(err: impl Display) -> Self {
        Self::Internal(err.to_string())
    }

    pub fn conflict(err: impl Into<String>) -> Self {
        Self::Conflict(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        match &self {
            AppError::Internal(_) => tracing::error!(error = %message, "internal server error"),
            AppError::Unauthorized(_) => tracing::warn!(error = %message, "unauthorized"),
            _ => {}
        }
        let body = axum::Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<toasty::Error> for AppError {
    fn from(value: toasty::Error) -> Self {
        dbg!(&value);
        if value.is_record_not_found() {
            Self::NotFound("Record not found".into())
        } else {
            Self::Internal(value.to_string())
        }
    }
}
