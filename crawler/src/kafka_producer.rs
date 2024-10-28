use kafka::producer::{Producer, Record, RequiredAcks};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct KafkaProducer {
    producer: Arc<Mutex<Producer>>,
}

impl KafkaProducer {
    pub fn new(hosts: Vec<String>) -> Self {
        let producer = Producer::from_hosts(hosts) // Sử dụng hosts ở đây
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();

        KafkaProducer {
            producer: Arc::new(Mutex::new(producer)),
        }
    }

    pub fn send_message(&self, topic: String, payload: &impl Serialize) {
        let mut producer = self.producer.lock().unwrap();
        let buffer = serde_json::to_string(payload).unwrap();
        producer
            .send(&Record::from_value(&topic, buffer.as_bytes()))
            .unwrap();
    }
}
