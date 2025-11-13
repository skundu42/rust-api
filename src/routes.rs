//! HTTP route handlers.
//!
//! Each function maps roughly to a REST verb and returns `Result` so Axum can
//! convert our custom errors into HTTP responses automatically.

use axum::{extract::{Path, State}, http::StatusCode, Json};

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

/// `POST /todos` - accepts a JSON body and returns `201 Created` with the
/// stored item. Returning the whole struct is handy for clients so they learn
/// the server-assigned id immediately.
pub async fn create_todo(
    State(app): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
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

/// `PUT /todos/:id` - update existing todos. We reuse the repo validation to
/// ensure empty payloads or whitespace titles get rejected.
pub async fn update_todo(
    Path(id): Path<u64>,
    State(app): State<AppState>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    let todo = app.repo().update(id, payload).await?;
    Ok(Json(todo))
}

/// `DELETE /todos/:id` - respond with `204 No Content` so clients know the
/// deletion succeeded even though there is no body.
pub async fn delete_todo(
    Path(id): Path<u64>,
    State(app): State<AppState>,
) -> Result<StatusCode, AppError> {
    app.repo().delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
