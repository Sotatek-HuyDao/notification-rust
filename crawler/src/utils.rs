use ethers::types::{Block, Transaction, H256};

pub fn filter_transaction(transaction: &Transaction, filter: &Option<String>) -> bool {
    match filter {
        Some(value) => transaction.hash.to_string().contains(value),
        None => true,
    }
}

pub fn filter_block(block: &Block<H256>, filter: &Option<String>) -> bool {
    match filter {
        Some(value) => match block.hash {
            Some(hash) => format!("{:?}", hash).contains(value),
            None => false,
        },
        None => true,
    }
}
