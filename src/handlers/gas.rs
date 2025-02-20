use axum::{extract::State, Json};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::models::transaction::{GasEstimation, TransactionInput};
use crate::services::ethereum::EthereumService;

pub async fn estimate_gas(
    State(service): State<Arc<EthereumService>>,
    Json(tx_input): Json<TransactionInput>,
) -> Result<Json<GasEstimation>> {
    if tx_input.from.is_empty() {
        return Err(Error::InvalidInput("Missing 'from' address".into()));
    }
    if tx_input.to.is_empty() {
        return Err(Error::InvalidInput("Missing 'to' address".into()));
    }

    tracing::debug!("Estimating gas for transaction: {:?}", tx_input);

    let estimation = service.estimate_gas(tx_input).await?;

    tracing::debug!("Estimated gas: {:?}", estimation);

    Ok(Json(estimation))
}
