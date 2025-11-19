//! High-level application wiring.
//!
//! `lib.rs` exposes the Axum router so integration tests can instantiate the
//! full stack without spinning up a TCP listener.
//!
//! # Middleware (Tower)
//!
//! Axum is built on top of `tower`, a library for modular networking components.
//! "Layers" allow us to wrap our application with cross-cutting concerns like:
//! - **Compression**: Gzip/Brotli responses automatically.
//! - **CORS**: Allow/deny requests from different origins (e.g., frontend apps).
//! - **Tracing**: Log every incoming request and outgoing response.

pub mod config;
pub mod errors;
pub mod models;
pub mod routes;
pub mod state;

use axum::{routing::get, Router};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

pub use state::AppState;

pub fn app(state: AppState) -> Router {
    // Each call to `route` returns a new router, so we can keep chaining.
    Router::new()
    .route("/health", get(routes::health))
        .route(
            "/todos",
            get(routes::list_todos).post(routes::create_todo),
        )
        .route(
            "/todos/:id",
            get(routes::get_todo)
                .put(routes::update_todo)
                .delete(routes::delete_todo),
        )
        // Layers run from bottom to top; we build them here so every handler
        // benefits from compression, permissive CORS, and request tracing.
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
