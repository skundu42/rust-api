//! Centralized error handling.
//!
//! # Error Handling in Rust
//!
//! Rust uses the `Result<T, E>` enum for error handling. We define our own
//! `AppError` enum to represent all possible things that can go wrong in our
//! domain.
//!
//! # `thiserror` vs `anyhow`
//!
//! - `thiserror`: Used for libraries and domain errors. It helps derive the
//!   `std::error::Error` trait automatically.
//! - `anyhow`: Used in application code (like `main.rs`) where we just want to
//!   propagate errors easily without defining custom types for everything.
//!
//! # `IntoResponse`
//!
//! Axum needs to know how to convert our `AppError` into an HTTP response.
//! By implementing `IntoResponse`, we can return `Result<T, AppError>` directly
//! from our handlers.

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
