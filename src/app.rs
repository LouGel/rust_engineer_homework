use crate::config::AppConfig;
use crate::error::Result;
use crate::handlers;
use crate::services::ethereum::EthereumService;
use axum::{routing::post, Router};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub async fn create_app(config: AppConfig) -> Result<Router> {
    let ethereum_service = EthereumService::new(&config).await?;
    let service = Arc::new(ethereum_service);

    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .into_inner();

    let app = Router::new()
        .route("/api/v1/estimate-gas", post(handlers::gas::estimate_gas))
        .route("/health", axum::routing::get(handlers::health))
        .layer(middleware)
        .with_state(service);

    Ok(app)
}
