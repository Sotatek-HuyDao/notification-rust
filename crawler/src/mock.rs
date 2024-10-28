use anyhow::Result;
use ethers::prelude::{
    providers::{MockProvider, Provider},
    types::{Block, Transaction, H256, U64},
};
use std::sync::Arc;

pub fn setup_provider(mock_provider: MockProvider) -> Arc<Provider<MockProvider>> {
    Arc::new(Provider::new(mock_provider))
}

pub fn get_mock() -> Result<MockProvider> {
    let mock_provider = MockProvider::new();

    let transaction: Transaction = Transaction::default();
    // push result for second transaction
    mock_provider.push(Some(transaction.clone()))?;
    // push result for first transaction
    mock_provider.push(Some(transaction))?;

    // push result for second block
    let mut block: Block<H256> = Block::default();
    block.transactions = vec![H256::zero(), H256::zero()];
    mock_provider.push(block)?;

    // push result for first block
    let block: Block<H256> = Block::default();
    mock_provider.push(block)?;

    // push result for get_block_number
    mock_provider.push(U64::from(2))?;

    Ok(mock_provider)
}
