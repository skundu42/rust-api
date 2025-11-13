//! Application state and repository implementation.
//!
//! Axum expects state to be cloneable, so we wrap our repository in an `Arc`.
//! The repo itself is a trait, which keeps us flexible about where data lives.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    errors::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};

/// CRUD contract shared by handlers and tests.
#[async_trait]
pub trait TodoRepo: Send + Sync + 'static {
    async fn list(&self) -> Result<Vec<Todo>, AppError>;
    async fn create(&self, input: CreateTodo) -> Result<Todo, AppError>;
    async fn get(&self, id: u64) -> Result<Todo, AppError>;
    async fn update(&self, id: u64, input: UpdateTodo) -> Result<Todo, AppError>;
    async fn delete(&self, id: u64) -> Result<(), AppError>;
}

/// Minimal in-memory store guarded by a mutex so async tasks can mutate it
/// sequentially.
#[derive(Default)]
struct InMemory {
    next_id: u64,
    items: HashMap<u64, Todo>,
}

#[async_trait]
impl TodoRepo for Mutex<InMemory> {
    async fn list(&self) -> Result<Vec<Todo>, AppError> {
        let guard = self.lock().await;
        Ok(guard.items.values().cloned().collect())
    }

    async fn create(&self, input: CreateTodo) -> Result<Todo, AppError> {
        // Trimming avoids storing strings that only differ by leading/trailing
        // whitespace.
        if input.title.trim().is_empty() {
            return Err(AppError::Validation(
                "title cannot be empty".to_string(),
            ));
        }

        let mut guard = self.lock().await;
        guard.next_id += 1;

        let todo = Todo {
            id: guard.next_id,
            title: input.title,
            done: false,
        };
        guard.items.insert(todo.id, todo.clone());
        Ok(todo)
    }

    async fn get(&self, id: u64) -> Result<Todo, AppError> {
        let guard = self.lock().await;
        guard.items.get(&id).cloned().ok_or(AppError::NotFound)
    }

    async fn update(&self, id: u64, input: UpdateTodo) -> Result<Todo, AppError> {
        let mut title = input.title;
        let done = input.done;

        // Peek at the title before we move it.
        if let Some(title) = title.as_ref() {
            if title.trim().is_empty() {
                return Err(AppError::Validation(
                    "title cannot be empty".to_string(),
                ));
            }
        }

        // PUT/patching nothing is usually a client mistake.
        if title.is_none() && done.is_none() {
            return Err(AppError::Validation(
                "provide at least one field to update".to_string(),
            ));
        }

        let mut guard = self.lock().await;
        let todo = guard
            .items
            .get_mut(&id)
            .ok_or(AppError::NotFound)?;

        if let Some(title) = title.take() {
            todo.title = title;
        }

        if let Some(done) = done {
            todo.done = done;
        }

        Ok(todo.clone())
    }

    async fn delete(&self, id: u64) -> Result<(), AppError> {
        let mut guard = self.lock().await;
        guard
            .items
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
    }
}

#[derive(Clone)]
pub struct AppState {
    repo: Arc<dyn TodoRepo>,
}

impl AppState {
    /// Provide a ready-to-go state object backed by the in-memory repo.
    pub fn new_in_memory() -> Self {
        Self {
            repo: Arc::new(Mutex::new(InMemory::default())),
        }
    }

    /// Returns a clone of the repository handle. Cheap thanks to `Arc`.
    pub fn repo(&self) -> Arc<dyn TodoRepo> {
        Arc::clone(&self.repo)
    }
}
