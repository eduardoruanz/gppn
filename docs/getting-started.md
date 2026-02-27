# Getting Started with GPPN

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Core protocol implementation |
| Go | 1.21+ | Settlement adapter services |
| protoc | 3.x | Protocol Buffers compilation |
| cmake | 3.x | Build dependency for RocksDB |
| Node.js | 18+ | TypeScript SDK |
| Docker | 24+ | Containerization |

### Install on macOS

```bash
# Install Homebrew dependencies
brew install protobuf cmake go

# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
rustup component add clippy rustfmt
```

## Building from Source

### Rust Workspace

```bash
# Build all crates
cargo build --workspace

# Run all tests (292 tests)
cargo test --workspace

# Check for lint issues
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --check
```

### Go Services

```bash
# Build all services
cd services && bash build_all.sh

# Or build individually
cd services/sa-ethereum && go build ./...
cd services/sa-bitcoin && go build ./...
cd services/sa-stablecoin && go build ./...
cd services/explorer-api && go build ./...
cd services/gateway && go build ./...
```

### TypeScript SDK

```bash
cd sdks/typescript
npm install
npm run build
npm test
```

## Running a Node

### Initialize Configuration

```bash
# Create default config in ./node-data/
cargo run -p gppn-cli -- init --data-dir ./node-data
```

This creates a `gppn.toml` configuration file:

```toml
[network]
listen_port = 9000
bootstrap_peers = []
max_peers = 50

[api]
listen_port = 9001
enabled = true

[storage]
path = "./node-data/db"

[metrics]
listen_port = 9002
enabled = true
```

### Start the Node

```bash
cargo run -p gppn-cli -- start --data-dir ./node-data
```

### Check Status

```bash
cargo run -p gppn-cli -- status --api-url http://localhost:9001
```

### List Peers

```bash
cargo run -p gppn-cli -- peers --api-url http://localhost:9001
```

### Send a Payment

```bash
cargo run -p gppn-cli -- send \
  --recipient did:gppn:key:12D3KooW... \
  --amount 50.00 \
  --currency USD \
  --api-url http://localhost:9001
```

## Local Testnet with Docker

Spin up a 3-node testnet with supporting infrastructure:

```bash
docker compose -f infra/docker/docker-compose.yml up -d
```

This starts:
- **gppn-node-1**: P2P on :9000, API on :9001, metrics on :9002
- **gppn-node-2**: P2P on :9010, API on :9011, metrics on :9012
- **gppn-node-3**: P2P on :9020, API on :9021, metrics on :9022
- **PostgreSQL 16**: on :5432 (for explorer-api)
- **DragonflyDB**: on :6379 (for caching)

## Next Steps

- [Architecture Overview](architecture.md) — understand the protocol layers
- [SDK Integration Guide](sdk-integration.md) — build applications with the TypeScript SDK
- [Node Operator Guide](node-operator.md) — production deployment
- [Protocol Specification](../spec/GPPN-SPEC-v1.md) — full protocol details
