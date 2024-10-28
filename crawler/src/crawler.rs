use anyhow::Result;

use ethers::{
    prelude::{
        providers::{JsonRpcClient, Middleware, Provider},
        types::H256,
    },
    types::Transaction,
};
use futures::{stream, StreamExt};
use std::sync::Arc;
use std::time::Instant;

const BUFFER_SIZE: usize = 10;

pub struct Crawler<T: JsonRpcClient> {
    provider: Arc<Provider<T>>,
    from_block: u64,
}

async fn fetch_block(provider: Arc<Provider<impl JsonRpcClient>>, block_number: u64) -> Vec<H256> {
    println!("Fetching block {}", block_number);
    let maybe_block = provider.get_block(block_number).await;

    match maybe_block {
        Ok(Some(block)) => block.transactions,
        _ => vec![],
    }
}

async fn fetch_transaction(
    provider: Arc<Provider<impl JsonRpcClient>>,
    tx: H256,
) -> Option<Transaction> {
    println!("Fetching transaction {}", tx);
    let maybe_transaction = provider.get_transaction(tx).await;

    match maybe_transaction {
        Ok(Some(transaction)) => Some(transaction),
        _ => None,
    }
}

impl<T: JsonRpcClient> Crawler<T> {
    pub fn new(provider: Arc<Provider<T>>, from_block: u64) -> Self {
        Self {
            provider,
            from_block,
        }
    }
    pub async fn get_transactions(self) -> Result<Vec<Transaction>> {
        let Crawler {
            provider,
            from_block,
        } = self;

        let to_block = provider.get_block_number().await?.as_u64();

        // For benchmarking
        let now = Instant::now();

        let address_transactions: Vec<Transaction> = stream::iter(from_block..=to_block)
            .map(|block_number| fetch_block(provider.clone(), block_number))
            .buffered(BUFFER_SIZE)
            .flat_map(stream::iter)
            .map(|tx| fetch_transaction(provider.clone(), tx))
            .buffered(BUFFER_SIZE)
            .filter_map(|tx| async { tx })
            .collect()
            .await;

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        Ok(address_transactions)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::mock::{get_mock, setup_provider};
    use ethers::prelude::{
        providers::MockProvider,
        types::{Block, Transaction, H256},
    };

    #[tokio::test]
    async fn test_fetch_block() -> Result<()> {
        let mock_provider = MockProvider::new();

        let mut block: Block<H256> = Block::default();

        block.transactions = vec![H256::zero()];
        mock_provider.push(block)?;

        let transactions = fetch_block(setup_provider(mock_provider), 1).await;

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0], H256::zero());

        Ok(())
    }

    async fn get_test_transaction(
        tx: H256,
        update_transaction: impl Fn(&mut Transaction) -> (),
    ) -> Result<Option<Transaction>> {
        let mock_provider = MockProvider::new();

        let mut transaction: Transaction = Transaction::default();

        update_transaction(&mut transaction);

        mock_provider.push(transaction)?;

        let transaction = fetch_transaction(setup_provider(mock_provider), tx).await;

        Ok(transaction)
    }

    #[tokio::test]
    async fn test_fetch_transaction() -> Result<()> {
        let tx = H256::zero();

        let transaction = get_test_transaction(tx, |_| {}).await?;
        assert_eq!(transaction, Some(Transaction::default()));

        Ok(())
    }

    #[tokio::test]
    async fn test_crawler() -> Result<()> {
        let mock_provider = get_mock()?;

        let crawler = Crawler::new(setup_provider(mock_provider), 1);

        let transactions = crawler.get_transactions().await?;
        assert_eq!(
            transactions,
            [Transaction::default(), Transaction::default()]
        );

        Ok(())
    }
}
