use crate::{
    kafka::KafkaProducer,
    tracer::{end_span, start_span},
    utils::{filter_block, filter_transaction},
};

use anyhow::Result;

use ethers::{
    prelude::{
        providers::{JsonRpcClient, Middleware, Provider},
        types::H256,
    },
    types::{Block, Transaction},
};
use futures::{stream, StreamExt};
use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

const BUFFER_SIZE: usize = 10;

pub struct FilterOption {
    pub tx_hash_filter: Option<String>,
    pub block_hash_filter: Option<String>,
}

pub struct Crawler<T: JsonRpcClient> {
    provider: Arc<Provider<T>>,
    from_block: u64,
    kafka_producer: Arc<KafkaProducer>,
    tsx_topic: Arc<String>,
    block_topic: Arc<String>,
    delay_time: u64,
    filter_options: Arc<FilterOption>,
}

async fn fetch_block(
    provider: &Arc<Provider<impl JsonRpcClient>>,
    block_number: u64,
) -> Option<Block<H256>> {
    // println!("Fetching block {}", block_number);
    let span = start_span("fetch_block");
    let maybe_block = provider.get_block(block_number).await;
    end_span(span);
    match maybe_block {
        Ok(Some(block)) => Some(block),
        Ok(None) => {
            println!("Block number {} not found.", block_number);
            None
        }
        Err(err) => {
            println!("Error fetching block {}: {}", block_number, err);
            None
        }
    }
}

async fn fetch_transaction(
    provider: &Arc<Provider<impl JsonRpcClient>>,
    tx: H256,
) -> Option<Transaction> {
    // println!("Fetching transaction {}", tx);
    let span = start_span("fetch_transaction");
    let maybe_transaction = provider.get_transaction(tx).await;
    end_span(span);
    match maybe_transaction {
        Ok(Some(transaction)) => Some(transaction),
        Ok(None) => {
            println!("Tx {} not found.", tx);
            None
        }
        Err(err) => {
            println!("Error fetching tx {}: {}", tx, err);
            None
        }
    }
}

impl<T: JsonRpcClient> Crawler<T> {
    pub fn new(
        provider: Arc<Provider<T>>,
        from_block: u64,
        delay_time: u64,
        filter_options: FilterOption,
    ) -> Self {
        let hosts: Vec<String> = env::var("KAFKA_BROKER_HOST")
            .expect("KAFKA_BROKER_HOST not set")
            .split(',')
            .map(|s| s.to_string())
            .collect();
        let tsx_topic = env::var("KAFKA_TX_TOPIC").expect("KAFKA_TX_TOPIC not set");
        let block_topic = env::var("KAFKA_BLOCK_TOPIC").expect("KAFKA_BLOCK_TOPIC not set");
        let kafka_producer = Arc::new(KafkaProducer::new(hosts).expect("Setup Kafka error"));

        Self {
            provider,
            from_block,
            kafka_producer,
            tsx_topic: Arc::new(tsx_topic),
            block_topic: Arc::new(block_topic),
            delay_time,
            filter_options: Arc::new(filter_options),
        }
    }

    pub async fn get_transactions(self) -> Result<Vec<Transaction>> {
        let span = start_span("get_transactions");
        let Crawler {
            provider,
            from_block,
            kafka_producer,
            tsx_topic,
            block_topic,
            delay_time,
            filter_options,
            ..
        } = self;

        let to_block = provider.get_block_number().await?.as_u64();

        let address_transactions: Vec<Transaction> = stream::iter(from_block..=to_block)
            .map(|block_number| {
                let block_topic_clone = Arc::clone(&block_topic);
                let provider_clone = Arc::clone(&provider);
                let kafka_producer_clone = Arc::clone(&kafka_producer);
                let filter_options_clone = Arc::clone(&filter_options);
                async move {
                    sleep(Duration::from_millis(delay_time)).await;
                    match fetch_block(&provider_clone, block_number).await {
                        Some(block) => {
                            if filter_block(&block, &filter_options_clone.block_hash_filter) {
                                let span = start_span("kafka_send_message");
                                if let Err(e) =
                                    &kafka_producer_clone.send_message(&block_topic_clone, &block)
                                {
                                    eprintln!("Failed to send message: {:?}", e);
                                }
                                end_span(span);
                                println!("Sent block {:?}", block.hash);
                            } else {
                                println!("Skip block {:?}", block.hash);
                            }
                            block.transactions
                        }
                        None => {
                            println!("Block number {} not found.", block_number);
                            Vec::new()
                        }
                    }
                }
            })
            .buffered(BUFFER_SIZE)
            .flat_map(stream::iter)
            .map(|tx| {
                let tsx_topic_clone = Arc::clone(&tsx_topic);
                let provider_clone = Arc::clone(&provider);
                let kafka_producer_clone = Arc::clone(&kafka_producer);
                let filter_options_clone = Arc::clone(&filter_options);
                async move {
                    sleep(Duration::from_millis(delay_time)).await;
                    let transaction = fetch_transaction(&provider_clone, tx).await;
                    if let Some(tx) = &transaction {
                        if filter_transaction(tx, &filter_options_clone.tx_hash_filter) {
                            if let Err(e) = &kafka_producer_clone.send_message(&tsx_topic_clone, tx)
                            {
                                eprintln!("Failed to send message: {:?}", e);
                            }
                            println!("Sent tx {:?}", tx.hash);
                        } else {
                            println!("Skip tx {:?}", tx.hash);
                        }
                    }
                    transaction
                }
            })
            .buffered(BUFFER_SIZE)
            .filter_map(|tx| async { tx })
            .collect()
            .await;

        end_span(span);
        Ok(address_transactions)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::mock::{get_mock, setup_provider};
    use dotenv::dotenv;
    use ethers::prelude::{
        providers::MockProvider,
        types::{Block, Transaction, H256},
    };
    use std::env;

    #[tokio::test]
    async fn test_fetch_block() -> Result<()> {
        let mock_provider = MockProvider::new();

        let block: Block<H256> = Block::default();
        mock_provider.push(block)?;

        let block = fetch_block(&setup_provider(mock_provider), 1).await;
        assert_eq!(block, Some(Block::default()));
        Ok(())
    }

    async fn get_test_transaction(
        tx: H256,
        update_transaction: impl Fn(&mut Transaction),
    ) -> Result<Option<Transaction>> {
        let mock_provider = MockProvider::new();

        let mut transaction: Transaction = Transaction::default();

        update_transaction(&mut transaction);

        mock_provider.push(transaction)?;

        let transaction = fetch_transaction(&setup_provider(mock_provider), tx).await;

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
    #[ignore = "This test requires the Kafka service"]
    async fn test_crawler_success() -> Result<()> {
        let mock_provider = get_mock()?;

        dotenv().ok();
        env::set_var(
            "KAFKA_BROKER_HOST",
            "localhost:9092,localhost:9093,localhost:9094",
        );
        env::set_var("KAFKA_TX_TOPIC", "tx");
        env::set_var("KAFKA_BLOCK_TOPIC", "block");
        let _crawler = Crawler::new(
            setup_provider(mock_provider),
            1,
            0,
            FilterOption {
                tx_hash_filter: None,
                block_hash_filter: None,
            },
        );

        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_crawler_fail_missing_env() {
        let mock_provider = get_mock().unwrap();

        env::remove_var("KAFKA_BROKER_HOST");
        env::remove_var("KAFKA_TX_TOPIC");
        env::remove_var("KAFKA_BLOCK_TOPIC");
        Crawler::new(
            setup_provider(mock_provider),
            1,
            0,
            FilterOption {
                tx_hash_filter: None,
                block_hash_filter: None,
            },
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn test_crawler_fail_wrong_env() {
        let mock_provider = get_mock().unwrap();

        env::set_var("KAFKA_BROKER_HOST", "invalid_url");
        env::set_var("KAFKA_TX_TOPIC", "invalid_url");
        env::set_var("KAFKA_BLOCK_TOPIC", "invalid_url");
        Crawler::new(
            setup_provider(mock_provider),
            1,
            0,
            FilterOption {
                tx_hash_filter: None,
                block_hash_filter: None,
            },
        );
    }
}
