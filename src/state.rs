//! Application state and repository implementation.
//!
//! # The Repository Pattern
//!
//! We use the Repository pattern to decouple our business logic (routes) from
//! the data access layer. This allows us to:
//! - Swap out the storage backend (e.g., in-memory -> Postgres) without changing routes.
//! - Mock the database for unit testing.
//!
//! # Concurrency
//!
//! Axum handlers run concurrently on multiple threads. To share state safely,
//! we wrap it in an `Arc` (Atomic Reference Counted) pointer.
//!
//! Inside the `Arc`, we need interior mutability. We use `RwLock` (Read-Write Lock)
//! instead of `Mutex` because it allows multiple concurrent readers (e.g., many
//! users listing todos at once) while ensuring exclusive access for writers.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    errors::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};

/// CRUD contract shared by handlers and tests.
///
/// `Send + Sync + 'static` ensures the trait object can be safely shared
/// across threads in the async runtime.
#[async_trait]
pub trait TodoRepo: Send + Sync + 'static {
    async fn list(&self) -> Result<Vec<Todo>, AppError>;
    async fn create(&self, input: CreateTodo) -> Result<Todo, AppError>;
    async fn get(&self, id: u64) -> Result<Todo, AppError>;
    async fn update(&self, id: u64, input: UpdateTodo) -> Result<Todo, AppError>;
    async fn delete(&self, id: u64) -> Result<(), AppError>;
}

/// Minimal in-memory store guarded by a RwLock.
#[derive(Default)]
struct InMemory {
    next_id: u64,
    items: HashMap<u64, Todo>,
}

#[async_trait]
impl TodoRepo for RwLock<InMemory> {
    async fn list(&self) -> Result<Vec<Todo>, AppError> {
        // Acquire a read lock. Multiple readers can hold this simultaneously.
        let guard = self.read().await;
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

        // Acquire a write lock. This blocks until all readers/writers are done.
        let mut guard = self.write().await;
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
        let guard = self.read().await;
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

        let mut guard = self.write().await;
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
        let mut guard = self.write().await;
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
            repo: Arc::new(RwLock::new(InMemory::default())),
        }
    }

    /// Returns a clone of the repository handle. Cheap thanks to `Arc`.
    pub fn repo(&self) -> Arc<dyn TodoRepo> {
        Arc::clone(&self.repo)
    }
}
