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

        Ok(storage
            .blocks
            .values()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(limit as usize)
            .collect())
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
