use std::collections::HashMap;

use super::block::Block;
use super::transaction::Transaction;

pub struct Storage {
    pub blocks: HashMap<String, Block>,
    pub transactions: HashMap<String, Transaction>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub async fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.hash.clone(), block);
    }

    pub async fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions
            .insert(transaction.hash.clone(), transaction);
    }

    pub async fn get_block(&self, hash: &str) -> Option<Block> {
        self.blocks.get(hash).cloned()
    }

    pub async fn get_transaction(&self, hash: &str) -> Option<Transaction> {
        self.transactions.get(hash).cloned()
    }
}
