use ethers::{
    providers::{Http, Middleware, Provider, ProviderError},
    types::{transaction::eip2718::TypedTransaction, TransactionRequest, H160, U256},
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::config::AppConfig;
use crate::error::{Error, Result};
use crate::models::transaction::{GasEstimation, TransactionInput, TransactionType};
use crate::utils::cache::cached_gas_price;

#[derive(Clone)]
pub struct EthereumService {
    provider: Arc<Provider<Http>>,
    cache_duration: Duration,
}

impl EthereumService {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let provider: Provider<Http> = Provider::<Http>::try_from(config.ethereum_rpc_url.clone())
            .expect("Issue with provider");

        provider
            .get_block_number()
            .await
            .map_err(|e| Error::Provider(format!("Failed to connect to Ethereum node: {}", e)))?;

        Ok(Self {
            provider: Arc::new(provider),
            cache_duration: config.cache_duration,
        })
    }

    pub async fn estimate_gas(&self, tx: TransactionInput) -> Result<GasEstimation> {
        let from = tx
            .from
            .parse::<H160>()
            .map_err(|_| Error::InvalidInput(format!("Invalid 'from' address: {}", tx.from)))?;

        let to = tx
            .to
            .parse::<H160>()
            .map_err(|_| Error::InvalidInput(format!("Invalid 'to' address: {}", tx.to)))?;

        let mut transaction = TransactionRequest::new().from(from).to(to);

        if let Some(data) = &tx.data {
            transaction = transaction.data(
                ethers::types::Bytes::from_str(data)
                    .map_err(|_| Error::InvalidInput("Invalid transaction data".into()))?,
            );
        }

        if let Some(value) = &tx.value {
            transaction = transaction.value(
                U256::from_dec_str(value)
                    .map_err(|_| Error::InvalidInput("Invalid transaction value".into()))?,
            );
        }

        let tx_type = if tx.max_fee_per_gas.is_some() || tx.max_priority_fee_per_gas.is_some() {
            TransactionType::EIP1559
        } else {
            TransactionType::Legacy
        };
        let typed_tx: TypedTransaction = transaction.into();

        let (gas_price_result, gas_limit_result) = tokio::join!(
            self.get_gas_price(tx_type.clone(), &tx),
            self.provider.estimate_gas(&typed_tx, None)
        );

        let gas_price = gas_price_result?;
        let gas_limit = gas_limit_result.map_err(|e| handle_estimate_error(e))?;

        let total_cost = gas_price.saturating_mul(gas_limit);
        let eth_cost = ethers::utils::format_ether(total_cost);

        let estimated_time = self.estimate_execution_time(&tx_type).await;

        Ok(GasEstimation {
            gas_limit: gas_limit.to_string(),
            gas_price: gas_price.to_string(),
            estimated_cost_wei: total_cost.to_string(),
            estimated_cost_eth: eth_cost,
            estimated_execution_time: estimated_time,
            type_of_transaction: tx_type.to_string(),
        })
    }

    async fn get_gas_price(&self, tx_type: TransactionType, tx: &TransactionInput) -> Result<U256> {
        match tx_type {
            TransactionType::Legacy => {
                if let Some(gas_price_str) = &tx.gas_price {
                    return U256::from_dec_str(gas_price_str)
                        .map_err(|_| Error::InvalidInput("Invalid gas price".into()));
                }

                cached_gas_price(self.provider.clone(), self.cache_duration)
                    .await
                    .map_err(|e| Error::Provider(format!("Failed to get gas price: {}", e)))
            }
            TransactionType::EIP1559 => {
                let base_fee = self.get_base_fee().await?;
                let priority_fee = if let Some(priority_fee_str) = &tx.max_priority_fee_per_gas {
                    U256::from_dec_str(priority_fee_str)
                        .map_err(|_| Error::InvalidInput("Invalid priority fee".into()))?
                } else {
                    U256::from(1_500_000_000u64)
                };

                Ok(base_fee.saturating_add(priority_fee))
            }
        }
    }

    async fn get_base_fee(&self) -> Result<U256> {
        let block = self
            .provider
            .get_block(ethers::types::BlockNumber::Latest)
            .await
            .map_err(|e| Error::Provider(format!("Failed to get latest block: {}", e)))?
            .ok_or_else(|| Error::Provider("Latest block not found".into()))?;

        block
            .base_fee_per_gas
            .ok_or_else(|| Error::Provider("Base fee not available (pre-EIP1559 network?)".into()))
    }

    async fn estimate_execution_time(&self, tx_type: &TransactionType) -> Option<String> {
        match tx_type {
            TransactionType::Legacy => Some("~30 seconds".to_string()),
            TransactionType::EIP1559 => Some("~15 seconds".to_string()),
        }
    }
}

fn handle_estimate_error(error: ProviderError) -> Error {
    match error {
        ProviderError::EnsError(_) => Error::InvalidInput("Invalid ENS name in address".into()),
        e => Error::Provider(format!("{}", e)),
    }
}
