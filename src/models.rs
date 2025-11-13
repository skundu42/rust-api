//! Data structures that travel across the API boundary.
//!
//! Keeping models in their own module makes it easy to reuse the same
//! types when we talk to the database layer or send JSON to the client.

use serde::{Deserialize, Serialize};

/// Representation of a todo item as it leaves the repository or gets
/// serialized back to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

/// Payload used when creating a new todo. We only need a title here because
/// the server decides the new id and the `done` flag.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

/// PATCH/PUT payload that lets the caller flip the completion state or rename
/// the todo. All fields are optional so we can change either or both.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}
