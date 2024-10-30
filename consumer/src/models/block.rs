use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub hash: String,
    pub number: String,
    pub timestamp: String,
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub number: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

#[Object]
impl Block {
    async fn hash(&self) -> &str {
        &self.hash
    }

    async fn number(&self) -> u64 {
        self.number
    }

    async fn timestamp(&self) -> u64 {
        self.timestamp
    }

    async fn transactions(&self) -> &Vec<String> {
        &self.transactions
    }
}
