//! Configuration management.
//!
//! This module is responsible for loading and validating environment variables
//! required for the application to run. By centralizing config logic here,
//! we ensure that the app fails early (at startup) if something is missing,
//! rather than failing at runtime.

use std::env;
use std::net::SocketAddr;

use anyhow::Context;

/// Holds all the configuration values needed by the application.
#[derive(Clone, Debug)]
pub struct Config {
    /// The address and port the server will listen on.
    pub server_addr: SocketAddr,
    /// The log level filter (e.g., "info", "debug", "rust_api=trace").
    pub rust_log: String,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `PORT` is missing or not a valid number.
    /// - `HOST` is provided but not a valid IP address (defaults to 0.0.0.0).
    pub fn from_env() -> anyhow::Result<Self> {
        // `std::env::var` returns a Result, which is idiomatic Rust for "this might fail".
        // We use `unwrap_or` to provide sensible defaults for local development.
        
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        
        let server_addr = format!("{host}:{port}")
            .parse::<SocketAddr>()
            .context("failed to parse HOST:PORT as a socket address")?;

        // RUST_LOG is used by the `tracing` crate to filter logs.
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| {
            "rust_api=info,axum::rejection=trace,tower_http=info".to_string()
        });

        Ok(Self {
            server_addr,
            rust_log,
        })
    }
}
