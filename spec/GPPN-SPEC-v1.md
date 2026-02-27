# GPPN Protocol Specification v1.0

## Abstract

The Global Payment Protocol Network (GPPN) is a decentralized, open protocol for universal payment routing, messaging, and settlement. GPPN enables any payment system — fiat rails, cryptocurrencies, stablecoins, or proprietary networks — to interoperate through a common protocol layer.

## 1. Introduction

### 1.1 Motivation

Today's payment infrastructure is fragmented across incompatible networks. GPPN bridges this gap by providing a protocol-level standard for:

- **Payment Messages**: A universal message format for describing payments across any currency or network
- **Routing**: Intelligent, multi-hop payment routing that discovers optimal paths across heterogeneous networks
- **Settlement**: An abstraction layer that supports atomic settlement across different blockchains and payment rails
- **Identity**: Decentralized, self-sovereign identity using W3C DIDs with built-in trust scoring
- **Transport**: Peer-to-peer communication using proven libp2p networking primitives

### 1.2 Design Principles

1. **Protocol, not platform**: GPPN defines interfaces, not implementations
2. **Settlement agnostic**: Any settlement mechanism can be plugged in
3. **Trust-minimized**: Cryptographic proofs and HTLC atomicity reduce counterparty risk
4. **Incrementally deployable**: Nodes can join with partial capability
5. **Privacy-preserving**: End-to-end encrypted metadata, minimal on-chain footprint

## 2. Protocol Layers

### 2.1 Layer Stack

```
┌─────────────────────────────────────┐
│  PML: Payment Message Layer         │  Application
├─────────────────────────────────────┤
│  SRL: Smart Routing Layer           │  Routing
├─────────────────────────────────────┤
│  SAL: Settlement Abstraction Layer  │  Settlement
├─────────────────────────────────────┤
│  TIL: Trust & Identity Layer        │  Identity
├─────────────────────────────────────┤
│  OGL: Overlay & Gossip Layer        │  Transport
└─────────────────────────────────────┘
```

## 3. Payment Message Layer (PML)

### 3.1 PaymentMessage Format

All payment messages follow the protobuf schema defined in `proto/gppn/v1/payment_message.proto`:

| Field | Type | Description |
|-------|------|-------------|
| `pm_id` | string | UUID v7 (time-ordered) |
| `version` | uint32 | Protocol version (1) |
| `sender` | string | Sender DID |
| `receiver` | string | Receiver DID |
| `amount` | Amount | Value + currency |
| `state` | PaymentState | Current FSM state |
| `conditions` | Condition[] | Settlement conditions |
| `settlement_preferences` | string[] | Preferred settlement methods |
| `encrypted_metadata` | bytes | ChaCha20-Poly1305 encrypted payload |
| `routing_hints` | RoutingHint[] | Path preference hints |
| `ttl` | uint64 | Time-to-live (seconds) |
| `timestamp` | uint64 | Creation time (Unix seconds) |
| `signature` | bytes | Ed25519 signature |

### 3.2 Payment State Machine

```
            ┌──────────┐
            │ Created  │
            └────┬─────┘
                 │
            ┌────▼─────┐
     ┌──────│  Routed  │──────┐
     │      └────┬─────┘      │
     │           │             │
     │      ┌────▼─────┐      │
     │      │ Accepted │──────┤
     │      └────┬─────┘      │
     │           │             │
     │      ┌────▼─────┐      │
     │      │ Settling │      │
     │      └──┬───┬───┘      │
     │         │   │          │
     │    ┌────▼┐ ┌▼─────┐   │
     │    │Settl│ │Failed│   │
     │    │ ed  │ └──┬───┘   │
     │    └─────┘    │       │
     │          (re-route)   │
     │                       │
     ├───────────────────────┤
     │                       │
┌────▼─────┐          ┌─────▼────┐
│ Expired  │          │Cancelled │
└──────────┘          └──────────┘
```

Valid transitions:
- `Created → Routed` — Route discovered
- `Routed → Accepted` — Receiver accepted
- `Accepted → Settling` — Settlement initiated
- `Settling → Settled` — Settlement confirmed
- `Settling → Failed` — Settlement failed (may re-route)
- `Created → Expired` — TTL exceeded
- `Routed → Expired` — TTL exceeded
- `Accepted → Expired` — TTL exceeded
- `Created → Cancelled` — Sender cancelled
- `Routed → Cancelled` — Sender cancelled
- `Accepted → Cancelled` — Sender cancelled

### 3.3 Signing

Payment messages are signed using Ed25519. The signing payload is computed by deterministic serialization of all fields except the `signature` field itself.

## 4. Smart Routing Layer (SRL)

### 4.1 Distributed Routing Table (DRT)

Each node maintains a DRT — a concurrent hash map of route entries:

| Field | Type | Description |
|-------|------|-------------|
| `destination` | DID | Target node |
| `next_hop` | PeerId | Next peer in path |
| `currencies` | string[] | Supported currency pairs |
| `liquidity` | f64 | Available liquidity score |
| `fee_rate` | f64 | Fee rate for this hop |
| `latency_ms` | u64 | Measured latency |
| `trust_score` | f64 | Trust score of next hop |
| `ttl` | Duration | Route validity period |

### 4.2 Route Discovery

Three-phase algorithm:

1. **Discovery**: Broadcast Route Requests (RReq) via GossipSub on `/gppn/routing/1.0.0`
2. **Evaluation**: Collect Route Responses (RRes), compute composite scores
3. **Selection**: Modified Dijkstra + Yen's k-shortest paths for multi-path routing

### 4.3 Route Scoring

```
Score = α × (1/NormalizedCost) + β × (1/NormalizedLatency) + γ × TrustScore + δ × LiquidityScore
```

Default weights: α=0.3, β=0.2, γ=0.3, δ=0.2

## 5. Settlement Abstraction Layer (SAL)

### 5.1 Settlement Adapter Interface

Every settlement adapter implements:

```
interface ISettlement {
    initiate(request) → Result<SettlementStatus>
    confirm(transaction_id) → Result<SettlementStatus>
    rollback(transaction_id) → Result<SettlementStatus>
    get_status(transaction_id) → Result<SettlementStatus>
    estimate_cost(request) → Result<CostEstimate>
    estimate_latency(request) → Result<Duration>
    supported_currencies() → Vec<String>
}
```

### 5.2 HTLC Mechanics

For multi-hop payments, GPPN uses Hash Time-Locked Contracts:

1. Sender generates a random preimage `P` and computes `H = BLAKE3(P)`
2. Each hop creates an HTLC locked to `H` with cascading timeouts (T, T-Δ, T-2Δ, ...)
3. Receiver reveals `P` to claim the final HTLC
4. Each intermediate node claims by revealing `P` to the previous hop
5. If any HTLC expires, the entire chain refunds automatically

### 5.3 Built-in Adapters

| Adapter | Network | Latency | Cost |
|---------|---------|---------|------|
| SA-Internal | Off-chain ledger | ~0ms | Free |
| SA-Ethereum | Ethereum L1 | ~15s | Gas fees |
| SA-Bitcoin | Bitcoin | ~60min | Mining fees |
| SA-Stablecoin | ERC-20 (USDC/USDT/DAI) | ~15s | Gas fees |

## 6. Trust & Identity Layer (TIL)

### 6.1 Decentralized Identifiers (DIDs)

Format: `did:gppn:<method>:<identifier>`

Methods:
- `key` — Ed25519 public key based
- `web` — DNS-based resolution

### 6.2 DID Documents

```json
{
  "id": "did:gppn:key:z6Mk...",
  "verificationMethod": [{
    "id": "did:gppn:key:z6Mk...#keys-1",
    "type": "Ed25519VerificationKey2020",
    "publicKeyMultibase": "z6Mk..."
  }],
  "authentication": ["did:gppn:key:z6Mk...#keys-1"],
  "service": [{
    "type": "GppnNode",
    "serviceEndpoint": "/ip4/203.0.113.1/tcp/9000"
  }]
}
```

### 6.3 Trust Scoring

Composite score (0.0 to 1.0):

| Component | Weight | Description |
|-----------|--------|-------------|
| Uptime | 20% | Node availability |
| Success Rate | 25% | Payment completion rate |
| Latency | 15% | Average response time |
| Volume | 15% | Transaction throughput |
| Age | 10% | Time in network |
| Attestations | 15% | Peer endorsements |

Global trust scores are computed using an EigenTrust-inspired iterative algorithm over the trust graph.

## 7. Overlay & Gossip Layer (OGL)

### 7.1 Transport Stack

```
TCP → Noise (encryption) → Yamux (multiplexing)
```

### 7.2 Protocols

| Protocol | ID | Purpose |
|----------|----|---------|
| GossipSub | `/meshsub/1.1.0` | Broadcast (routing, announcements) |
| Kademlia | `/ipfs/kad/1.0.0` | Peer discovery, DHT |
| Identify | `/ipfs/id/1.0.0` | Peer information exchange |
| GPPN Req/Res | `/gppn/req/1.0.0` | Direct peer messaging (CBOR) |
| mDNS | N/A | Local network discovery |

### 7.3 GossipSub Topics

- `/gppn/payments/1.0.0` — Payment announcements and status updates
- `/gppn/routing/1.0.0` — Route advertisements and requests

## 8. Security Considerations

- **Replay Protection**: UUID v7 IDs are time-ordered and unique
- **Signature Verification**: All payment messages must have valid Ed25519 signatures
- **Encryption**: Payment metadata is end-to-end encrypted (ChaCha20-Poly1305)
- **HTLC Atomicity**: Settlement is atomic — either all hops settle or all refund
- **Sybil Resistance**: EigenTrust scoring penalizes low-reputation nodes
- **DoS Mitigation**: TTL-based message expiry, rate limiting at network layer

## 9. Wire Format

All protocol messages use:
- **Protobuf** (prost) for structured data serialization
- **CBOR** for libp2p request-response codec
- **GossipSub** message format for broadcast messages

## 10. Versioning

The protocol uses semantic versioning. Breaking changes increment the major version in protocol identifiers (e.g., `/gppn/req/2.0.0`).
