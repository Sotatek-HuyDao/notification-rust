mod controller;
mod kafka;
mod models;
mod routes;
mod utils;
use crate::controller::QueryRoot;
use crate::models::storage::Storage;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{extract::Extension, routing::get, Router, Server};
use dotenv::dotenv;
use kafka::KafkaConsumer;
use routes::{graphql_handler, graphql_playground};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let storage = Arc::new(RwLock::new(Storage::new()));
    let hosts: Vec<String> = env::var("KAFKA_BROKER_HOST")
        .expect("KAFKA_BROKER_HOST not set")
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let tsx_topic = env::var("KAFKA_TX_TOPIC").expect("KAFKA_TX_TOPIC not set");
    let block_topic = env::var("KAFKA_BLOCK_TOPIC").expect("KAFKA_BLOCK_TOPIC not set");
    let kafka_consumer = KafkaConsumer::new(hosts, vec![tsx_topic, block_topic], storage.clone())
        .expect("Failed to create Kafka consumer");

    let consumer_handle = tokio::spawn(async move {
        kafka_consumer.start_consuming().await;
    });

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(storage.clone())
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    let server_handle = tokio::spawn(async move {
        Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Wait for both tasks
    tokio::try_join!(consumer_handle, server_handle).unwrap();
}
