# GPPN Architecture Overview

## Protocol Layers

GPPN implements a 5-layer architecture inspired by the OSI model, specialized for payment processing:

### Layer 1: OGL — Overlay & Gossip Layer

**Crate**: `gppn-network`

The foundation layer handles peer-to-peer networking using libp2p:

- **Transport**: TCP + Noise encryption + Yamux multiplexing
- **Discovery**: Kademlia DHT for peer discovery, mDNS for local networks
- **Messaging**: GossipSub for broadcast (route advertisements, payment announcements)
- **Direct**: Request-response protocol with CBOR serialization for peer-to-peer messages

Key components:
- `GppnNode` — manages the libp2p swarm and event loop
- `GppnBehaviour` — composed libp2p behaviour (gossipsub + Kademlia + mDNS + identify + request-response)
- `TopicManager` — GossipSub topic management for `/gppn/payments/1.0.0` and `/gppn/routing/1.0.0`

### Layer 2: TIL — Trust & Identity Layer

**Crate**: `gppn-identity`

Decentralized identity and trust scoring:

- **DIDs**: W3C-compatible Decentralized Identifiers (`did:gppn:<method>:<id>`)
- **DID Documents**: Public keys, service endpoints, authentication methods
- **Verifiable Credentials**: Issuer-signed attestations about peers
- **Trust Graph**: Directed weighted graph with EigenTrust-inspired iterative scoring
- **Trust Score**: Weighted composite of uptime (20%), success rate (25%), latency (15%), volume (15%), age (10%), attestations (15%)

### Layer 3: SRL — Smart Routing Layer

**Crate**: `gppn-routing`

Distributed payment routing:

- **DRT (Distributed Routing Table)**: Lock-free concurrent routing table backed by `DashMap`
- **Route Entries**: Next hop, destination, supported currencies, liquidity, fee rate, latency, trust score, TTL
- **PathFinder**: 3-phase algorithm:
  1. **Discovery**: Broadcast Route Requests (RReq)
  2. **Evaluation**: Score Route Responses (RRes) using composite scoring
  3. **Selection**: Modified Dijkstra + Yen's k-shortest paths
- **Scoring**: `RouteScore = α×(1/Cost) + β×(1/Latency) + γ×TrustScore + δ×Liquidity`

### Layer 4: SAL — Settlement Abstraction Layer

**Crate**: `gppn-settlement`, Go services

Pluggable settlement with blockchain adapters:

- **ISettlement trait**: Async interface for all settlement adapters (initiate, confirm, rollback, get_status, estimate_cost)
- **HTLC Engine**: Hash Time-Locked Contracts for atomic multi-hop settlement with cascading timeouts
- **Settlement Manager**: Adapter registry and orchestration
- **Adapters**:
  - `SA-Internal` (Rust): Off-chain double-entry ledger — zero cost, zero latency
  - `SA-Ethereum` (Go): Ethereum L1 settlement
  - `SA-Bitcoin` (Go): Bitcoin on-chain settlement
  - `SA-Stablecoin` (Go): USDC/USDT/DAI settlement

### Layer 5: PML — Payment Message Layer

**Crate**: `gppn-core`

The core payment message format and lifecycle:

- **PaymentMessage**: UUID v7 ID, sender/receiver DIDs, amount, currency, conditions, encrypted metadata, Ed25519 signature, routing hints
- **State Machine**: 8-state FSM with strict transition rules:
  ```
  Created → Routed → Accepted → Settling → Settled
                                         → Failed → (re-route)
  Created/Routed/Accepted → Expired
  Created/Routed/Accepted → Cancelled
  ```
- **Protobuf**: All message types defined in `proto/gppn/v1/` as the source of truth

## Cryptography

**Crate**: `gppn-crypto`

- **Signing**: Ed25519 (ed25519-dalek v2) for payment message signatures
- **Key Exchange**: X25519 Diffie-Hellman for shared secret derivation
- **Encryption**: ChaCha20-Poly1305 AEAD for payment metadata encryption
- **Hashing**: BLAKE3 for fast, secure content hashing and Merkle roots
- **KDF**: Argon2id for key derivation from passwords
- **Memory Safety**: `zeroize` crate for private key material cleanup on drop

## Node Architecture

**Crate**: `gppn-node`

The full node binary orchestrates all layers:

```
┌─────────────────────────────────┐
│           gppn-node             │
├─────────────────────────────────┤
│  JSON-RPC API (port 9001)       │
│  Prometheus metrics (port 9002) │
├─────────────────────────────────┤
│  PaymentMessage Layer (PML)     │
│  Smart Routing Layer (SRL)      │
│  Settlement Layer (SAL)         │
│  Trust & Identity Layer (TIL)   │
│  Overlay & Gossip Layer (OGL)   │
├─────────────────────────────────┤
│  RocksDB Storage                │
│  (payments|routing|identity|    │
│   state|peers)                  │
└─────────────────────────────────┘
```

Init sequence: keypair → storage (RocksDB) → identity → network → routing → settlement → API → metrics → event loop

## Data Flow: Sending a Payment

1. **Client** calls `sendPayment()` via SDK or CLI
2. **PML** creates a `PaymentMessage` (state: Created), signs with Ed25519
3. **SRL** discovers routes via PathFinder (broadcast RReq, collect RRes, score, select)
4. **PML** transitions to Routed, sets routing hints
5. **OGL** sends payment message to next hop via request-response
6. Intermediate nodes forward along the route
7. **Receiver** accepts → PML transitions to Accepted
8. **SAL** initiates settlement via appropriate adapter (HTLC for multi-hop)
9. Settlement confirms → PML transitions to Settled
10. HTLC claims propagate back along the route
