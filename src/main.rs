//! Binary entry point that bootstraps the Axum server.
//!
//! Keeping the bulk of our logic inside `lib.rs` means the `main` function just
//! wires up logging, state, and graceful shutdown.

use std::time::Duration;

use anyhow::Result;
use axum::serve;
use rust_api::{app, AppState};
use tokio::net::TcpListener;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Loading `.env` files locally keeps credentials out of the shell session.
    dotenvy::dotenv().ok();

    // Load configuration from the environment.
    // This will fail fast if required variables are missing.
    let config = rust_api::config::Config::from_env()?;

    // Initialize the tracing subscriber for logging.
    // `EnvFilter` uses the `RUST_LOG` variable to determine what to log.
    let env_filter = EnvFilter::new(&config.rust_log);

    fmt().with_env_filter(env_filter).compact().init();

    let state = AppState::new_in_memory();
    let app = app(state);

    tracing::info!(addr = %config.server_addr, "starting server");

    // `TcpListener` + `serve` gives us finer control over graceful shutdown.
    let listener = TcpListener::bind(config.server_addr).await?;
    serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Waits for Ctrl+C (or SIGTERM on Unix) so we can exit cleanly.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("shutdown signal received, waiting 200ms...");
    tokio::time::sleep(Duration::from_millis(200)).await;
}
