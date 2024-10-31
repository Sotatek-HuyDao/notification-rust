## Objective
Implement a notification queue for new transactions on a blockchain of your choice using Rust, Kafka and GraphQL

## Structutre
```
├── build
│   └── docker-compose.yaml
├── consumer
│   ├── Cargo.lock
│   ├── Cargo.toml
│   └── src
│       ├── controller.rs
│       ├── kafka.rs
│       ├── main.rs
│       ├── models
│       │   ├── block.rs
│       │   ├── mod.rs
│       │   ├── storage.rs
│       │   └── transaction.rs
│       ├── routes.rs
│       └── utils.rs
└── crawler
    ├── Cargo.lock
    ├── Cargo.toml
    ├── README.md
    └── src
        ├── crawler.rs
        ├── kafka.rs
        ├── main.rs
        ├── mock.rs
        └── tracer.rs
```
- /build: Contains pre-configured services
- /crawler: Service for crawling data, including transaction and block information
  - /src/crawler.rs: Data crawler
  - /src/kafka.rs: Kafka producer, sends data to Kafka
  - /src/tracer.rs: Tracing service
- /consumer: Service for consuming data from Kafka and exporting it to a GraphQL service
  - /src/controller.rs: Handles GraphQL query execution
  - /src/kafka.rs: Consumes data from Kafka
  - /src/models/: Defines module structures
  - /src/routes.rs: Routes for GraphQL
  
## Setup
- Start Kafka and Jaeger if they aren't already running.
```bash
docker compose -f ./build/docker-compose.yaml -p notification up
```
Waiting for the service to be created. Ensure all containers are running

- Update the configuration in the .env files for the crawler and consumer (you can use the default configuration for testing).

## Testing
1. Run crawler service
```bash
cd crawler

cargo run
```
- Check the log in terminal. Go to http://localhost:16686 to view the data tracing from the crawler service.
- You can apply filters for transaction hashes or block hashes using the command below:
```bash
# filter block
cargo run -- --block-hash-filter=block_hash_include_substring
# filter transaction
cargo run -- --tx-hash-filter=tx_hash_include_substring
```

2. Run consumer service
```bash
cd consumer

cargo run
```
- Go to http://localhost:3000 to query data 
- Support querying by transaction hash or block hash:
```
query {
  block(hash: "0xfd5653216de31a2e905e4d6a449cc8d6cc03c90f78c39d2abc041c25051ca1cc") {
    hash
    timestamp
    transactions
  }
}

query {
  transaction(hash: "0x1e0bb670fca670058e09be28136de11ff3f1400cba64072b8266c4a719f30d95") {
    hash
    from
    to
    value
  }
}
```
- Support querying by block number
```
query {
  blocksByNumber(number: 17166114) {
    hash
    timestamp
    transactions
  }
}

query {
  transactionsForBlock(blockHash: "0xfd5653216de31a2e905e4d6a449cc8d6cc03c90f78c39d2abc041c25051ca1cc"<!-- or use blockNumber: 17166114-->) { 
    hash
    from
    to
    value
  }
}
```
- Get latest blocks
```
query {
  latestBlocks(limit:10) {
    hash
    timestamp
    transactions
  }
}
```
