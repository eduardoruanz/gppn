# GIP-0001: GPPN Protocol v1

| Field | Value |
|-------|-------|
| GIP | 0001 |
| Title | GPPN Protocol v1 |
| Status | Draft |
| Type | Standards Track |
| Created | 2025-01-15 |

## Abstract

This GIP defines the initial version (v1) of the GPPN protocol, establishing the core message formats, state machine, routing algorithm, settlement abstraction, identity system, and network transport.

## Motivation

The global payment ecosystem lacks a universal interoperability protocol. Existing solutions are either centralized (SWIFT, card networks), blockchain-specific (Lightning Network, IBC), or limited in scope. GPPN provides a general-purpose, decentralized protocol that bridges all payment systems.

## Specification

### Protocol Identifier

All GPPN v1 protocols use the prefix `/gppn/*/1.0.0`:
- `/gppn/req/1.0.0` — Request-response protocol
- GossipSub topics: `/gppn/payments/1.0.0`, `/gppn/routing/1.0.0`

### Message Types

#### Request Messages

1. **RouteRequest**: Discover routes to a destination
   - `request_id`: Unique request identifier
   - `target_did`: Destination DID
   - `source_currency`, `destination_currency`: Currency pair
   - `amount`: Payment amount (u128, smallest unit)
   - `max_hops`: Maximum path length

2. **PaymentMessage**: Direct payment data
   - `data`: Serialized protobuf PaymentMessage bytes

3. **Ping**: Liveness check

#### Response Messages

1. **RouteResponse**: Route discovery result
   - `request_id`: Matching request ID
   - `found`: Whether a route was found
   - `path`: Ordered list of peer IDs
   - `estimated_fee`: Total fee in smallest unit
   - `hop_count`: Number of hops

2. **PaymentAck**: Payment acknowledgement
   - `accepted`: Whether the payment was accepted
   - `reason`: Optional rejection reason

3. **Pong**: Liveness response

4. **Error**: Error response with message

### Serialization

- Request/response: CBOR encoding via libp2p cbor codec
- Payment messages: Protocol Buffers (proto3)
- Hashing: BLAKE3 for content addressing

### Cryptographic Algorithms

| Purpose | Algorithm | Key Size |
|---------|-----------|----------|
| Signing | Ed25519 | 256-bit |
| Key Exchange | X25519 | 256-bit |
| Encryption | ChaCha20-Poly1305 | 256-bit |
| Hashing | BLAKE3 | 256-bit |
| KDF | Argon2id | Configurable |

### Network Requirements

- Transport: TCP with Noise encryption and Yamux multiplexing
- Discovery: Kademlia DHT + mDNS (local)
- Broadcast: GossipSub v1.1
- Maximum message size: 1 MB
- Default P2P port: 9000

## Rationale

### Why libp2p?

libp2p provides battle-tested networking primitives (transport encryption, multiplexing, DHT, gossip) used by IPFS and Ethereum 2.0. Building on libp2p avoids reinventing these complex components.

### Why Ed25519?

Ed25519 offers fast signing/verification, small key/signature sizes, and resistance to side-channel attacks. It's the standard for modern decentralized identity (DID:key method).

### Why BLAKE3?

BLAKE3 is significantly faster than SHA-256 while providing equivalent security. Its tree-hashing mode enables parallelized hashing for large payloads.

### Why HTLC for settlement?

HTLCs provide atomic multi-hop settlement without requiring trust in intermediate nodes. This pattern is proven in Lightning Network and adapts well to GPPN's multi-network settlement model.

## Backwards Compatibility

This is the initial protocol version. No backwards compatibility concerns.

## Reference Implementation

The reference implementation is located in the GPPN repository:
- Rust: `crates/` (8 crates)
- Go: `services/` (5 services)
- TypeScript: `sdks/typescript/`
- Protobuf: `proto/gppn/v1/`

## Copyright

This document is licensed under Apache License, Version 2.0.
