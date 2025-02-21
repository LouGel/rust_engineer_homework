use alloy_json_rpc::RpcError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Deserialize)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Ethereum provider error: {0}")]
    Provider(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Gas estimation failed: {0}")]
    GasEstimation(String),
    #[error("Server error: {0}")]
    Server(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let err_type = self.error_type();
        let (status, error_message) = match self {
            Error::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::Provider(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            Error::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::GasEstimation(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::Server(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": err_type  // Use the error_type method here
            }
        }));

        (status, body).into_response()
    }
}

impl Error {
    pub fn error_type(&self) -> &'static str {
        match self {
            Error::Config(_) => "configuration_error",
            Error::Provider(_) => "provider_error",
            Error::InvalidInput(_) => "invalid_input",
            Error::GasEstimation(_) => "gas_estimation_error",
            Error::Server(_) => "server_error",
        }
    }
}

impl<T> From<RpcError<T>> for Error {
    fn from(error: RpcError<T>) -> Self {
        match error {
            RpcError::ErrorResp(payload) => {
                let message = payload.message.to_lowercase();
                if message.contains("execution reverted") {
                    Error::GasEstimation(format!(
                        "Transaction would fail: Details: {}",
                        payload.message
                    ))
                } else if message.contains("gas required exceeds allowance") {
                    Error::GasEstimation(
                        "Transaction would fail: gas required exceeds allowance".into(),
                    )
                } else {
                    Error::Provider(format!("RPC error: {}", payload.message))
                }
            }
            RpcError::Transport(_) => Error::Provider(format!("Transport error")),
            RpcError::NullResp => Error::Provider("Received null response".into()),
            RpcError::SerError(e) => Error::Provider(format!("Serialization error: {}", e)),
            RpcError::DeserError { err, text } => {
                Error::Provider(format!("Deserialization error: {} for text: {}", err, text))
            }
            RpcError::UnsupportedFeature(msg) => {
                Error::Provider(format!("Unsupported feature: {}", msg))
            }
            RpcError::LocalUsageError(e) => Error::Provider(format!("Local usage error: {}", e)),
        }
    }
}

// Other From implementations remain the same...
