pub mod crawler;
pub mod kafka_producer;
#[cfg(test)]
pub mod mock;
use crate::crawler::Crawler;
use anyhow::Result;

use dotenv::dotenv;
use ethers::prelude::providers::Http;
use ethers::prelude::{
    providers::{JsonRpcClient, Provider},
    HttpRateLimitRetryPolicy, RetryClient,
};
use std::env::var;
use std::str::FromStr;
use std::sync::Arc;

fn get_provider() -> Result<Provider<impl JsonRpcClient>> {
    let http_provider = var("HTTP_PROVIDER").expect("HTTP_PROVIDER not set");

    Ok(Provider::new(RetryClient::new(
        Http::from_str(&http_provider)?,
        Box::new(HttpRateLimitRetryPolicy),
        10,
        500,
    )))
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let provider = Arc::new(get_provider().unwrap());
    let from_block = var("FROM_BLOCK")
        .expect("FROM_BLOCK not set")
        .parse::<u64>()
        .expect("Invalid FROM_BLOCK");
    let crawler = Crawler::new(provider, from_block.clone());
    let _ = crawler.get_transactions().await;
}
