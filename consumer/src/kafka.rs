use crate::{
    models::{
        block::{Block, BlockData},
        storage::Storage,
        transaction::{Transaction, TransactionData},
    },
    utils::hex_to_dec,
};
use kafka::{
    client::{FetchOffset, GroupOffsetStorage},
    consumer::Consumer,
    Error,
};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct KafkaConsumer {
    consumer: Arc<Mutex<Consumer>>,
    storage: Arc<RwLock<Storage>>,
}

impl KafkaConsumer {
    pub fn new(
        hosts: Vec<String>,
        topics: Vec<String>,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<Self, Error> {
        let mut consumer_builder = Consumer::from_hosts(hosts)
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("group".to_owned())
            .with_offset_storage(Some(GroupOffsetStorage::Kafka));
        for topic in topics {
            consumer_builder = consumer_builder.with_topic(topic);
        }
        let consumer = consumer_builder.create().unwrap();

        Ok(Self {
            consumer: Arc::new(Mutex::new(consumer)),
            storage,
        })
    }

    pub async fn start_consuming(&self) {
        loop {
            let mut consumer = self.consumer.lock().await;
            for ms in consumer.poll().unwrap().iter() {
                for message in ms.messages() {
                    match ms.topic() {
                        "block" => match serde_json::from_slice::<BlockData>(message.value) {
                            Ok(block) => {
                                let mut storage = self.storage.write().await;
                                storage
                                    .add_block(Block {
                                        hash: block.hash,
                                        number: hex_to_dec(block.number),
                                        timestamp: hex_to_dec(block.timestamp),
                                        transactions: block.transactions,
                                    })
                                    .await;
                            }
                            Err(e) => eprintln!("Failed to parse block: {}", e),
                        },
                        "tx" => match serde_json::from_slice::<TransactionData>(message.value) {
                            Ok(transaction) => {
                                let mut storage = self.storage.write().await;

                                storage
                                    .add_transaction(Transaction {
                                        hash: transaction.hash,
                                        block_hash: transaction.block_hash,
                                        from: transaction.from,
                                        to: transaction.to,
                                        value: hex_to_dec(transaction.value),
                                        block_number: hex_to_dec(transaction.block_number),
                                    })
                                    .await;
                            }
                            Err(e) => eprintln!("Failed to parse transaction: {}", e),
                        },
                        _ => {}
                    }
                }
                let _ = consumer.consume_messageset(ms);
            }
            consumer.commit_consumed().unwrap();
        }
    }
}
