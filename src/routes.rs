//! HTTP route handlers.
//!
//! # Axum Handlers
//!
//! Axum handlers are async functions that take "extractors" as arguments and
//! return something that implements `IntoResponse`.
//!
//! # Extractors
//!
//! - `State(app)`: Access shared application state (e.g., database connection).
//! - `Path(id)`: Extract parameters from the URL path (e.g., `/todos/:id`).
//! - `Json(payload)`: Parse the request body as JSON.
//!
//! The order of extractors matters! `State` and `Path` usually come first,
//! and `Json` (which consumes the body) comes last.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    errors::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
    state::AppState,
};

/// Tiny health check used by deployment platforms to know the process lives.
pub async fn health() -> &'static str {
    "ok"
}

/// `GET /todos` - list everything currently in the store.
pub async fn list_todos(
    State(app): State<AppState>,
) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = app.repo().list().await?;
    Ok(Json(todos))
}

/// `POST /todos` - accepts a JSON body and returns `201 Created`.
pub async fn create_todo(
    State(app): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    // Validate input before hitting the database.
    payload.validate()?;

    let todo = app.repo().create(payload).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

/// `GET /todos/:id` - fetch a single todo or bubble up `404`.
pub async fn get_todo(
    Path(id): Path<u64>,
    State(app): State<AppState>,
) -> Result<Json<Todo>, AppError> {
    let todo = app.repo().get(id).await?;
    Ok(Json(todo))
}

/// `PUT /todos/:id` - update existing todos.
pub async fn update_todo(
    Path(id): Path<u64>,
    State(app): State<AppState>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    // Validate input.
    payload.validate()?;

    let todo = app.repo().update(id, payload).await?;
    Ok(Json(todo))
}

/// `DELETE /todos/:id` - respond with `204 No Content`.
pub async fn delete_todo(
    Path(id): Path<u64>,
    State(app): State<AppState>,
) -> Result<StatusCode, AppError> {
    app.repo().delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
