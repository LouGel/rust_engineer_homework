pub mod gas;

use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

pub async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
