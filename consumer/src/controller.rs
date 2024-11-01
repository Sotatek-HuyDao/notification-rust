use async_graphql::*;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::{block::Block, storage::Storage, transaction::Transaction};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn block(&self, ctx: &Context<'_>, hash: String) -> Result<Option<Block>, Error> {
        let storage = ctx.data::<Arc<RwLock<Storage>>>().unwrap();
        let storage = storage.read().await;
        Ok(storage.get_block(&hash).await)
    }
    async fn blocks_by_number(&self, ctx: &Context<'_>, number: u64) -> Result<Vec<Block>, Error> {
        let storage = ctx.data::<Arc<RwLock<Storage>>>().unwrap();
        let storage = storage.read().await;

        Ok(storage
            .blocks
            .values()
            .filter(|block| block.number == number)
            .cloned()
            .collect())
    }

    async fn latest_blocks(&self, ctx: &Context<'_>, limit: i32) -> Result<Vec<Block>, Error> {
        let storage = ctx.data::<Arc<RwLock<Storage>>>().unwrap();
        let storage = storage.read().await;
        let mut blocks = storage.blocks.values().cloned().collect::<Vec<_>>();
        blocks.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(blocks.into_iter().take(limit as usize).collect())
    }
    async fn transaction(
        &self,
        ctx: &Context<'_>,
        hash: String,
    ) -> Result<Option<Transaction>, Error> {
        let storage = ctx.data::<Arc<RwLock<Storage>>>().unwrap();
        let storage = storage.read().await;
        Ok(storage.get_transaction(&hash).await)
    }

    async fn transactions_for_block(
        &self,
        ctx: &Context<'_>,
        block_hash: Option<String>,
        block_number: Option<u64>,
    ) -> Result<Vec<Transaction>, Error> {
        let storage = ctx.data::<Arc<RwLock<Storage>>>().unwrap();
        let storage = storage.read().await;

        Ok(storage
            .transactions
            .values()
            .filter(|tx| {
                block_hash
                    .as_ref()
                    .map_or(true, |hash| &tx.block_hash == hash)
                    && block_number.map_or(true, |number| tx.block_number == number)
            })
            .cloned()
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::Schema;
    use std::collections::HashMap;

    async fn create_test_storage() -> Arc<RwLock<Storage>> {
        let mut blocks = HashMap::new();
        let mut transactions = HashMap::new();

        let block1 = Block {
            hash: "hash1".to_string(),
            number: 1,
            timestamp: 1000,
            transactions: vec!["tx1".to_string(), "tx2".to_string()],
        };

        let block2 = Block {
            hash: "hash2".to_string(),
            number: 2,
            timestamp: 1100,
            transactions: vec!["tx3".to_string()],
        };

        let block3 = Block {
            hash: "hash3".to_string(),
            number: 3,
            timestamp: 1200,
            transactions: vec!["tx4".to_string()],
        };

        blocks.insert(block1.hash.clone(), block1);
        blocks.insert(block2.hash.clone(), block2);
        blocks.insert(block3.hash.clone(), block3);

        // Create sample transactions
        let tx1 = Transaction {
            hash: "tx1".to_string(),
            block_hash: "hash1".to_string(),
            block_number: 1,
            from: "addr1".to_string(),
            to: "addr2".to_string(),
            value: 100,
        };

        let tx2 = Transaction {
            hash: "tx2".to_string(),
            block_hash: "hash1".to_string(),
            block_number: 1,
            from: "addr2".to_string(),
            to: "addr3".to_string(),
            value: 200,
        };

        transactions.insert(tx1.hash.clone(), tx1);
        transactions.insert(tx2.hash.clone(), tx2);

        Arc::new(RwLock::new(Storage {
            blocks,
            transactions,
        }))
    }

    #[tokio::test]
    async fn test_get_block_by_hash() {
        let storage = create_test_storage().await;
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage)
            .finish();

        let query = r#"
            query {
                block(hash: "hash1") {
                    hash
                    number
                    timestamp
                }
            }
        "#;

        let res = schema.execute(query).await;
        println!("{res:?}");
        assert!(res.data.to_string().contains("hash1"));
        assert!(res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_blocks_by_number() {
        let storage = create_test_storage().await;
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage)
            .finish();

        let query = r#"
            query {
                blocksByNumber(number: 1) {
                    hash
                    number
                    timestamp
                }
            }
        "#;

        let res = schema.execute(query).await;
        let data = res.data.to_string();

        // Should return both blocks with number 1
        assert!(data.contains("hash1"));
        assert!(!data.contains("hash2"));
        assert!(!data.contains("hash3"));
        assert!(res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_latest_blocks() {
        let storage = create_test_storage().await;
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage)
            .finish();

        let query = r#"
            query {
                latestBlocks(limit: 2) {
                    hash
                    timestamp
                }
            }
        "#;

        let res = schema.execute(query).await;
        let data = res.data.to_string();

        assert!(data.contains("hash3"));
        assert!(data.contains("hash2"));
        assert!(!data.contains("hash1"));
        assert!(res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_transaction() {
        let storage = create_test_storage().await;
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage)
            .finish();

        let query = r#"
            query {
                transaction(hash: "tx1") {
                    hash
                    blockHash
                    blockNumber
                    from
                    to
                    value
                }
            }
        "#;

        let res = schema.execute(query).await;
        let data = res.data.to_string();

        assert!(data.contains("tx1"));
        assert!(data.contains("hash1"));
        assert!(data.contains("100"));
        assert!(res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_transactions_for_block() {
        let storage = create_test_storage().await;
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage)
            .finish();

        let query = r#"
            query {
                transactionsForBlock(blockHash: "hash1") {
                    hash
                    blockNumber
                    value
                }
            }
        "#;

        let res = schema.execute(query).await;
        let data = res.data.to_string();

        assert!(data.contains("tx1"));
        assert!(data.contains("tx2"));
        assert!(res.errors.is_empty());

        let query = r#"
            query {
                transactionsForBlock(blockNumber: 1) {
                    hash
                    blockNumber
                    value
                }
            }
        "#;

        let res = schema.execute(query).await;
        let data = res.data.to_string();

        assert!(data.contains("tx1"));
        assert!(data.contains("tx2"));
        assert!(res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_nonexistent_block() {
        let storage = create_test_storage().await;
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage)
            .finish();

        let query = r#"
            query {
                block(hash: "nonexistent") {
                    hash
                    number
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(res.data.to_string().contains("null"));
        assert!(res.errors.is_empty());
    }
}
