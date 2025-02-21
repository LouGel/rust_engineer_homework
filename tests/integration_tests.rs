// tests/integration_tests.rs

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use eth_gas_estimator::{app::create_app, config::AppConfig};
use ethers_core::utils::{Anvil, AnvilInstance};
use serde_json::json;
use std::time::Duration;
use tower::ServiceExt;

async fn setup_test_app() -> (axum::Router, AnvilInstance) {
    // Setup forked node
    let anvil = Anvil::new()
        .fork("https://eth.llamarpc.com")
        .fork_block_number(18_000_000u64)
        .spawn();

    // Create test config
    let config = AppConfig {
        ethereum_rpc_url: anvil.endpoint(),
        cache_duration: Duration::from_secs(15),
        host: "127.0.0.1".parse().unwrap(),
        port: 8080,
        log_level: "debug".to_string(),
    };

    // Initialize app
    let app = create_app(config).await.expect("Failed to create app");

    (app, anvil)
}

#[tokio::test]
async fn test_gas_estimation_endpoint() {
    // Setup
    let (app, _anvil) = setup_test_app().await;

    // Test valid ETH transfer
    let valid_request = Request::builder()
        .method("POST")
        .uri("/api/v1/estimate-gas")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "from": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
                "to": "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5",
                "value": "1000000000000000",
                "data": "0x"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(valid_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let invalid_request = Request::builder()
        .method("POST")
        .uri("/api/v1/estimate-gas")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "from": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
                "to": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
                "value": "1000000000000000",
                "data": "0x"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(invalid_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(response.into_body().into(), 1000000_usize)
        .await
        .unwrap();
    let error_response: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    println!("Invalid transaction error: {:?}", error_response);
}

#[tokio::test]
async fn test_health_endpoint() {
    let (app, _anvil) = setup_test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_input() {
    let (app, _anvil) = setup_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/estimate-gas")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "from": "",  // Empty from address
                "to": "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5",
                "value": "1000000000000000",
                "data": "0x"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
