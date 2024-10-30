use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    pub hash: String,
    pub block_hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub block_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub block_hash: String,
    pub from: String,
    pub to: String,
    pub value: u64,
    pub block_number: u64,
}

#[Object]
impl Transaction {
    async fn hash(&self) -> &str {
        &self.hash
    }

    async fn block_hash(&self) -> &str {
        &self.block_hash
    }

    async fn from(&self) -> &str {
        &self.from
    }

    async fn to(&self) -> &str {
        &self.to
    }

    async fn value(&self) -> u64 {
        self.value
    }

    async fn block_number(&self) -> u64 {
        self.block_number
    }
}
