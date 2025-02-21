use eth_gas_estimator::app::create_app;
use eth_gas_estimator::config::AppConfig;
use eth_gas_estimator::error::{Error, Result};
use eth_gas_estimator::utils::shutdown::shutdown_signal;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(config.log_level.clone())
        .init();

    tracing::info!("Starting Ethereum gas estimator service");
    tracing::debug!("Using configuration: {:?}", config);

    let router = create_app(config.clone()).await?;
    let addr = config.server_address();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Couldn't create the server");

    tracing::info!("Listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Server(e.to_string()))?;

    Ok(())
}
