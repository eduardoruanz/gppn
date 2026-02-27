# GPPN — Global Payment Protocol Network

[![CI](https://github.com/gppn-protocol/gppn/actions/workflows/ci.yml/badge.svg)](https://github.com/gppn-protocol/gppn/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

> The universal language of money — a decentralized protocol for payment routing, messaging, and settlement.

GPPN is a peer-to-peer protocol that enables universal payment interoperability across currencies, networks, and borders. Think of it as **TCP/IP for payments**: any payment system can connect, route through, and settle via the GPPN network.

## Architecture

GPPN is organized into 5 protocol layers:

| Layer | Name | Purpose |
|-------|------|---------|
| **PML** | Payment Message Layer | Standardized payment messages with 8-state FSM |
| **SRL** | Smart Routing Layer | Distributed routing table + multi-path discovery |
| **SAL** | Settlement Abstraction Layer | Pluggable adapters (Ethereum, Bitcoin, stablecoins) |
| **TIL** | Trust & Identity Layer | DIDs, verifiable credentials, EigenTrust scoring |
| **OGL** | Overlay & Gossip Layer | libp2p networking, gossipsub, Kademlia DHT |

## Project Structure

```
├── crates/                   # Rust workspace (8 crates)
│   ├── gppn-core/            # Payment messages, state machine, types
│   ├── gppn-crypto/          # Ed25519, X25519, ChaCha20, BLAKE3
│   ├── gppn-network/         # libp2p node, gossipsub, Kademlia
│   ├── gppn-routing/         # Distributed routing table, pathfinder
│   ├── gppn-settlement/      # HTLC engine, settlement adapters
│   ├── gppn-identity/        # DIDs, trust graph, credentials
│   ├── gppn-node/            # Full node binary
│   └── gppn-cli/             # CLI tool
├── services/                 # Go microservices
│   ├── sa-ethereum/          # Ethereum settlement adapter
│   ├── sa-bitcoin/           # Bitcoin settlement adapter
│   ├── sa-stablecoin/        # USDC/USDT settlement adapter
│   ├── explorer-api/         # Network explorer REST API
│   └── gateway/              # HTTP gateway for SDK clients
├── sdks/typescript/          # TypeScript SDK (@gppn/sdk)
├── proto/gppn/v1/            # Protobuf schemas (source of truth)
├── infra/docker/             # Dockerfiles + docker-compose
├── spec/                     # Protocol specification + GIPs
└── docs/                     # Documentation
```

## Quick Start

### Prerequisites

- Rust 1.75+ (with cargo)
- Go 1.21+
- Protocol Buffers compiler (`protoc`)
- Docker & Docker Compose (for local testnet)
- Node.js 18+ (for TypeScript SDK)

### Build Everything

```bash
# Rust workspace
cargo build --workspace

# Go services
cd services && bash build_all.sh

# TypeScript SDK
cd sdks/typescript && npm install && npm run build
```

### Run Tests

```bash
# Rust (292 tests)
cargo test --workspace

# Go services (31 tests)
cd services/sa-ethereum && go test ./...
cd services/sa-bitcoin && go test ./...
cd services/sa-stablecoin && go test ./...

# TypeScript SDK (34 tests)
cd sdks/typescript && npm test
```

### Start a Local Testnet

```bash
# Spin up 3 GPPN nodes + PostgreSQL + DragonflyDB
docker compose -f infra/docker/docker-compose.yml up -d

# Initialize a node
cargo run -p gppn-cli -- init --data-dir ./node-data

# Start a node
cargo run -p gppn-cli -- start --data-dir ./node-data

# Check node status
cargo run -p gppn-cli -- status --api-url http://localhost:9001
```

### TypeScript SDK Usage

```typescript
import { GppnClient } from "@gppn/sdk";

const client = new GppnClient({ url: "http://localhost:9000" });
await client.connect();
await client.createIdentity();

const payment = await client.sendPayment(
  "did:gppn:key:recipient",
  "50.00",
  { code: "USD", decimals: 2 },
  "Coffee payment"
);

console.log(`Payment ${payment.id}: ${payment.status}`);
```

## Documentation

- [Architecture Overview](docs/architecture.md)
- [Getting Started Guide](docs/getting-started.md)
- [SDK Integration Guide](docs/sdk-integration.md)
- [Node Operator Guide](docs/node-operator.md)
- [Protocol Specification](spec/GPPN-SPEC-v1.md)
- [GIP-0001: Protocol v1](spec/gips/GIP-0001-protocol-v1.md)

## Protocol Specification

See [spec/GPPN-SPEC-v1.md](spec/GPPN-SPEC-v1.md) for the full protocol specification, including:

- Payment message format and lifecycle
- Routing algorithm (modified Dijkstra + Yen's k-shortest paths)
- Settlement abstraction and HTLC mechanics
- DID-based identity and trust scoring
- Network topology and gossip protocol

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under [Apache License, Version 2.0](LICENSE).
