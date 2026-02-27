# Getting Started with Veritas

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Core protocol implementation |
| Go | 1.21+ | Identity service APIs |
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

# Run all tests (315+ tests)
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
cd services/issuer-api && go build ./...
cd services/verifier-api && go build ./...
cd services/registry-api && go build ./...
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
cargo run -p veritas-cli -- init --data-dir ./node-data
```

This creates a `veritas.toml` configuration file:

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
cargo run -p veritas-cli -- start --data-dir ./node-data
```

### Check Status

```bash
cargo run -p veritas-cli -- status --api-url http://localhost:9001
```

### Issue a Credential

```bash
cargo run -p veritas-cli -- issue \
  --subject did:veritas:key:12D3KooW... \
  --credential-type KycBasic \
  --claims '{"full_name":"Alice Smith","country":"US"}' \
  --api-url http://localhost:9001
```

### Verify a Credential

```bash
cargo run -p veritas-cli -- verify \
  --credential ./credential.json \
  --api-url http://localhost:9001
```

## Local Testnet with Docker

Spin up a 3-node testnet with supporting infrastructure:

```bash
docker compose -f infra/docker/docker-compose.yml up -d
```

This starts:
- **veritas-node-1**: P2P on :9000, API on :9001, metrics on :9002
- **veritas-node-2**: P2P on :9010, API on :9011, metrics on :9012
- **veritas-node-3**: P2P on :9020, API on :9021, metrics on :9022
- **registry-api**: on :8084 (DID + schema registry)
- **PostgreSQL 16**: on :5432
- **DragonflyDB**: on :6379 (for caching)

## Next Steps

- [Architecture Overview](architecture.md) — understand the protocol layers
- [SDK Integration Guide](sdk-integration.md) — build applications with the TypeScript SDK
- [Node Operator Guide](node-operator.md) — production deployment
- [Protocol Specification](../spec/VERITAS-SPEC-v1.md) — full protocol details
