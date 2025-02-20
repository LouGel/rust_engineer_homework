mod app;
mod config;
mod error;
mod handlers;
mod models;
mod services;
mod utils;

use crate::app::create_app;
use crate::config::AppConfig;
use crate::error::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize config from environment
    let config = AppConfig::from_env()?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(config.log_level.clone())
        .init();

    tracing::info!("Starting Ethereum gas estimator service");
    tracing::debug!("Using configuration: {:?}", config);

    // Create and run the application
    let router = create_app(config.clone()).await?;
    let addr = config.server_address();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Couldn't create the server");

    tracing::info!("Listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| error::Error::Server(e.to_string()))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutdown signal received, starting graceful shutdown");
}
