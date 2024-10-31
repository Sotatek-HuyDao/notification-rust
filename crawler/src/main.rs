pub mod crawler;
pub mod kafka;
#[cfg(test)]
pub mod mock;
pub mod tracer;
pub mod utils;
use crate::crawler::Crawler;
use anyhow::Result;
use clap::Parser;
use crawler::FilterOption;
use dotenv::dotenv;
use ethers::prelude::providers::Http;
use ethers::prelude::{
    providers::{JsonRpcClient, Provider},
    HttpRateLimitRetryPolicy, RetryClient,
};
use opentelemetry::global;
use std::env::var;
use std::str::FromStr;
use std::sync::Arc;
use tracer::init_tracer_provider;

fn get_provider() -> Result<Provider<impl JsonRpcClient>> {
    let http_provider = var("HTTP_PROVIDER").expect("HTTP_PROVIDER not set");

    Ok(Provider::new(RetryClient::new(
        Http::from_str(&http_provider)?,
        Box::new(HttpRateLimitRetryPolicy),
        10,
        500,
    )))
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    tx_hash_filter: Option<String>,

    #[arg(long)]
    block_hash_filter: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();
    init_tracer_provider().expect("Failed to initialize tracer provider.");

    let _tracer = global::tracer("tracing-jaeger");

    let provider = Arc::new(get_provider().unwrap());
    let from_block = var("FROM_BLOCK")
        .expect("FROM_BLOCK not set")
        .parse::<u64>()
        .expect("Invalid FROM_BLOCK");
    let delay_time = var("DELAY_TIME")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(1000);

    let crawler = Crawler::new(
        provider,
        from_block,
        delay_time,
        FilterOption {
            tx_hash_filter: args.tx_hash_filter,
            block_hash_filter: args.block_hash_filter,
        },
    );
    let _ = crawler.get_transactions().await;

    global::shutdown_tracer_provider();
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[test]
    fn test_get_provider_success() {
        dotenv().ok();
        env::set_var("HTTP_PROVIDER", "http://localhost:8545");
        let provider = get_provider();
        assert!(
            provider.is_ok(),
            "Provider should be successfully created with valid HTTP_PROVIDER."
        );
    }

    #[test]
    #[should_panic(expected = "HTTP_PROVIDER not set")]
    fn test_get_provider_fail_missing_env() {
        env::remove_var("HTTP_PROVIDER");
        let _ = get_provider();
    }
}
