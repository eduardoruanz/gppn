# Veritas — Decentralized Identity Protocol

[![CI](https://github.com/veritas-protocol/veritas/actions/workflows/ci.yml/badge.svg)](https://github.com/veritas-protocol/veritas/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

> AI-resistant proof of humanity — a decentralized protocol for verifiable credentials, zero-knowledge proofs, and trust scoring.

Veritas is a peer-to-peer protocol that enables decentralized identity verification with privacy-preserving proofs. Issue verifiable credentials, prove attributes without revealing data, and build trust networks — all without centralized authorities.

## Architecture

Veritas is organized into 4 protocol layers:

| Layer | Name | Purpose |
|-------|------|---------|
| **VCL** | Verifiable Credential Layer | Credential issuance, holder wallet, verifier checks |
| **ZPL** | Zero-Knowledge Proof Layer | BLAKE3 commitment proofs (age, residency, KYC level) |
| **TIL** | Trust & Identity Layer | DIDs, DID documents, trust graph, humanity verification |
| **OGL** | Overlay & Gossip Layer | libp2p networking, gossipsub, Kademlia DHT |

## Project Structure

```
├── crates/                     # Rust workspace (8 crates)
│   ├── veritas-core/           # Credential types, state machine, config
│   ├── veritas-crypto/         # Ed25519, BLAKE3, ZKP commitments, selective disclosure
│   ├── veritas-network/        # libp2p node, gossipsub, Kademlia
│   ├── veritas-credentials/    # Issuer, holder wallet, verifier, schemas
│   ├── veritas-proof/          # Age, residency, KYC level, humanity proofs
│   ├── veritas-identity/       # DIDs, trust graph, humanity verification
│   ├── veritas-node/           # Full node binary
│   └── veritas-cli/            # CLI tool
├── services/                   # Go microservices
│   ├── issuer-api/             # Credential issuance API
│   ├── verifier-api/           # Credential verification API
│   ├── registry-api/           # DID + schema registry
│   └── gateway/                # HTTP gateway for SDK clients
├── sdks/typescript/            # TypeScript SDK (@veritas/sdk)
├── proto/veritas/v1/           # Protobuf schemas
├── infra/docker/               # Dockerfiles + docker-compose
├── spec/                       # Protocol specification + VIPs
└── docs/                       # Documentation
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
# Rust (315+ tests)
cargo test --workspace

# TypeScript SDK (55 tests)
cd sdks/typescript && npm test
```

### Start a Local Testnet

```bash
# Spin up 3 Veritas nodes + registry-api + PostgreSQL + DragonflyDB
docker compose -f infra/docker/docker-compose.yml up -d

# Initialize a node
cargo run -p veritas-cli -- init --data-dir ./node-data

# Start a node
cargo run -p veritas-cli -- start --data-dir ./node-data

# Check node status
cargo run -p veritas-cli -- status --api-url http://localhost:9001
```

### TypeScript SDK Usage

```typescript
import { VeritasClient } from "@veritas/sdk";

const client = new VeritasClient({ url: "http://localhost:9001" });
await client.connect();
await client.createIdentity();

// Issue a credential
const credential = await client.issueCredential(
  "did:veritas:key:subject_pubkey",
  ["KycBasic"],
  { full_name: "Alice Smith", country: "US", kyc_level: 2 }
);

// Request an age proof
const proofRequest = await client.requestAgeProof(18);

console.log(`Credential ${credential.id}: ${credential.state}`);
```

## Documentation

- [Architecture Overview](docs/architecture.md)
- [Getting Started Guide](docs/getting-started.md)
- [SDK Integration Guide](docs/sdk-integration.md)
- [Node Operator Guide](docs/node-operator.md)
- [Protocol Specification](spec/VERITAS-SPEC-v1.md)
- [VIP-0001: Protocol v1](spec/vips/VIP-0001-protocol-v1.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under [Apache License, Version 2.0](LICENSE).
