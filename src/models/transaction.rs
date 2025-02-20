use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct TransactionInput {
    pub from: String,
    pub to: String,
    pub data: Option<String>,
    pub value: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub _nonce: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GasEstimation {
    pub gas_limit: String,
    pub gas_price: String,
    pub estimated_cost_wei: String,
    pub estimated_cost_eth: String,
    pub estimated_execution_time: Option<String>,
    pub type_of_transaction: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum TransactionType {
    Legacy,
    EIP1559,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Legacy => write!(f, "legacy"),
            TransactionType::EIP1559 => write!(f, "eip1559"),
        }
    }
}
