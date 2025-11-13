//! Centralized error handling.
//!
//! Axum lets us convert domain errors into HTTP responses by implementing
//! `IntoResponse`. Keeping everything in one enum curbs boilerplate in the
//! handlers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application-level error. Each variant maps to an HTTP status via the
/// `IntoResponse` impl at the bottom.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("internal error")]
    Internal,
}

/// Shape of the JSON error response sent back to clients.
#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
        };

        (status, Json(ErrorBody { error: msg })).into_response()
    }
}
