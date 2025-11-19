//! Data structures that travel across the API boundary.
//!
//! # Serde
//!
//! We use `serde` (SERialization-DEserialization) to convert between Rust structs
//! and JSON.
//! - `Serialize`: Rust -> JSON (for responses)
//! - `Deserialize`: JSON -> Rust (for requests)
//!
//! # Validation
//!
//! We implement `validate()` methods on our input models to ensure data integrity
//! before it reaches the repository. This keeps the domain logic clean.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// Representation of a todo item as it leaves the repository or gets
/// serialized back to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

/// Payload used when creating a new todo.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

impl CreateTodo {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation("title cannot be empty".to_string()));
        }
        if self.title.len() > 100 {
            return Err(AppError::Validation(
                "title cannot be longer than 100 characters".to_string(),
            ));
        }
        Ok(())
    }
}

/// PATCH/PUT payload that lets the caller flip the completion state or rename
/// the todo.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}

impl UpdateTodo {
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err(AppError::Validation("title cannot be empty".to_string()));
            }
            if title.len() > 100 {
                return Err(AppError::Validation(
                    "title cannot be longer than 100 characters".to_string(),
                ));
            }
        }
        Ok(())
    }
}
