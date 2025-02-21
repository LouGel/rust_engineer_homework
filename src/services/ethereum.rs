use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::{TransactionInput as TxData, TransactionRequest};
use eyre::Result;
use std::{str::FromStr, sync::Arc, time::Duration, u128};

use crate::{
    config::AppConfig,
    error::Error,
    models::transaction::{GasEstimation, TransactionInput, TransactionType},
    utils::cache::cached_gas_price,
};

const DEFAULT_PRIORITY_FEE: u128 = 1_500_000_000; // 1.5 Gwei
const WEI_PER_ETH: f64 = 1_000_000_000_000_000_000f64;

#[derive(Clone)]
pub struct EthereumService {
    provider: Arc<RootProvider>,
    cache_duration: Duration,
}

impl EthereumService {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let provider = RootProvider::new_http(config.ethereum_rpc_url.parse()?);

        // Test provider connection
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
        let transaction = self.build_transaction_request(&tx)?;
        let tx_type = self.determine_transaction_type(&tx);

        // Parallel fetching of gas price and limit
        let (gas_price, gas_limit) = tokio::join!(
            self.get_gas_price(tx_type.clone(), &tx),
            self.provider.estimate_gas(&transaction)
        );

        let gas_price = gas_price?;
        let gas_limit = gas_limit.map_err(|e| Error::Provider(format!("{}", e)))?;

        let total_cost = gas_price.saturating_mul(gas_limit.into());

        Ok(GasEstimation {
            gas_limit: gas_limit.to_string(),
            gas_price: gas_price.to_string(),
            estimated_cost_wei: total_cost.to_string(),
            estimated_cost_eth: format_ether(total_cost),
            estimated_execution_time: self.estimate_execution_time(&tx_type),
            type_of_transaction: tx_type.to_string(),
        })
    }

    fn build_transaction_request(&self, tx: &TransactionInput) -> Result<TransactionRequest> {
        let mut transaction = TransactionRequest::default();

        // Set addresses
        transaction.from = Some(parse_address(&tx.from)?);
        transaction.to = Some(parse_address(&tx.to)?.into());

        // Set optional data
        if let Some(data) = &tx.data {
            transaction.input = TxData::new(parse_bytes(data)?);
        }

        // Set optional value
        if let Some(value) = &tx.value {
            transaction.value = Some(parse_u256(value)?);
        }

        // Set gas parameters
        if let Some(max_fee) = &tx.max_fee_per_gas {
            transaction.max_fee_per_gas = Some(parse_u128(max_fee)?);
        }
        if let Some(max_priority_fee) = &tx.max_priority_fee_per_gas {
            transaction.max_priority_fee_per_gas = Some(parse_u128(max_priority_fee)?);
        }
        if let Some(gas_price) = &tx.gas_price {
            transaction.gas_price = Some(parse_u128(gas_price)?);
        }

        Ok(transaction)
    }

    fn determine_transaction_type(&self, tx: &TransactionInput) -> TransactionType {
        if tx.max_fee_per_gas.is_some() || tx.max_priority_fee_per_gas.is_some() {
            TransactionType::EIP1559
        } else {
            TransactionType::Legacy
        }
    }

    async fn get_gas_price(&self, tx_type: TransactionType, tx: &TransactionInput) -> Result<u128> {
        match tx_type {
            TransactionType::Legacy => self.get_legacy_gas_price(tx).await,
            TransactionType::EIP1559 => self.get_eip1559_gas_price(tx).await,
        }
    }

    async fn get_legacy_gas_price(&self, tx: &TransactionInput) -> Result<u128> {
        if let Some(gas_price_str) = &tx.gas_price {
            return Ok(parse_u128(gas_price_str)?);
        }

        Ok(cached_gas_price(self.provider.clone(), self.cache_duration)
            .await
            .map_err(|e| Error::Provider(format!("Failed to get gas price: {}", e)))?)
    }

    async fn get_eip1559_gas_price(&self, tx: &TransactionInput) -> Result<u128> {
        let suggested_priority_fee = tx
            .max_priority_fee_per_gas
            .as_ref()
            .map(|fee| parse_u128(fee))
            .transpose()?
            .unwrap_or(DEFAULT_PRIORITY_FEE);

        let current_gas_price = self.provider.get_gas_price().await?;
        Ok(std::cmp::max(current_gas_price, suggested_priority_fee))
    }

    fn estimate_execution_time(&self, tx_type: &TransactionType) -> Option<String> {
        Some(
            match tx_type {
                TransactionType::Legacy => "~30 seconds",
                TransactionType::EIP1559 => "~15 seconds",
            }
            .to_string(),
        )
    }
}

// Helper functions for parsing
fn parse_address(input: &str) -> Result<Address, Error> {
    Address::from_str(input).map_err(|_| Error::InvalidInput(format!("Invalid address: {}", input)))
}

fn parse_bytes(input: &str) -> Result<Bytes, Error> {
    Bytes::from_str(input).map_err(|_| Error::InvalidInput("Invalid transaction data".into()))
}

fn parse_u256(input: &str) -> Result<U256, Error> {
    U256::from_str(input).map_err(|_| Error::InvalidInput("Invalid U256 value".into()))
}

fn parse_u128(input: &str) -> Result<u128, Error> {
    u128::from_str(input).map_err(|_| Error::InvalidInput("Invalid u128 value".into()))
}

fn format_ether(wei: u128) -> String {
    let ether_value = wei as f64 / WEI_PER_ETH;
    format!("{:.18}", ether_value)
}
