use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
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
                "type": err_type,
            }
        }));

        (status, body).into_response()
    }
}

impl Error {
    fn error_type(&self) -> &'static str {
        match self {
            Error::Config(_) => "configuration_error",
            Error::Provider(_) => "provider_error",
            Error::InvalidInput(_) => "invalid_input",
            Error::GasEstimation(_) => "gas_estimation_error",
            Error::Server(_) => "server_error",
        }
    }
}

impl From<ethers::providers::ProviderError> for Error {
    fn from(err: ethers::providers::ProviderError) -> Self {
        Error::Provider(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::InvalidInput(err.to_string())
    }
}

impl From<eyre::Report> for Error {
    fn from(err: eyre::Report) -> Self {
        Error::Server(err.to_string())
    }
}
