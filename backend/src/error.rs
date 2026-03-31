use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
    Redirect(Redirect),
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
        let (status, message) = match self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Redirect(redirect) => return redirect.into_response(),
        };
        match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!(error = %message, "internal server error")
            }
            StatusCode::UNAUTHORIZED => tracing::warn!(error = %message, "unauthorized"),
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

impl From<Redirect> for AppError {
    fn from(value: Redirect) -> Self {
        Self::Redirect(value)
    }
}
