# VIP-0001: Veritas Protocol v1

| Field | Value |
|-------|-------|
| VIP | 0001 |
| Title | Veritas Protocol v1 |
| Status | Draft |
| Type | Standards Track |
| Created | 2025-01-15 |

## Abstract

This VIP defines the initial version (v1) of the Veritas protocol, establishing the core credential formats, state machine, zero-knowledge proof types, identity system, and network transport.

## Motivation

The identity ecosystem lacks a universal, privacy-preserving verification protocol. Existing solutions are either centralized (government databases, KYC providers), privacy-invasive (sharing full documents), or blockchain-specific. Veritas provides a general-purpose, decentralized protocol that enables verifiable credentials with zero-knowledge proofs across any trust domain.

## Specification

### Protocol Identifier

All Veritas v1 protocols use the prefix `/veritas/*/1.0.0`:
- `/veritas/req/1.0.0` — Request-response protocol
- GossipSub topics: `/veritas/credentials/v1`, `/veritas/proof-requests/v1`

### Message Types

#### Request Messages

1. **CredentialRequest**: Request a credential from an issuer
   - `request_id`: Unique request identifier
   - `subject_did`: Subject DID
   - `credential_type`: Requested credential type
   - `claims`: Claim data for issuance

2. **ProofRequest**: Request a zero-knowledge proof from a holder
   - `request_id`: Unique request identifier
   - `verifier_did`: Verifier DID
   - `proof_type`: Type of proof (age, residency, kyc_level)
   - `params`: Proof parameters (min_age, allowed_countries, etc.)

3. **DidResolve**: Resolve a DID to its document

4. **TrustAttestation**: Submit a trust attestation for a peer

5. **Ping**: Liveness check

#### Response Messages

1. **CredentialResponse**: Issued credential
   - `credential`: Signed VerifiableCredential

2. **ProofResponse**: Zero-knowledge proof
   - `valid`: Whether the proof was generated successfully
   - `proof_data`: Serialized ZK proof

3. **DidDocument**: Resolved DID document

4. **TrustUpdate**: Updated trust score

5. **Pong**: Liveness response

6. **Error**: Error response with message

### Serialization

- Request/response: CBOR encoding via libp2p cbor codec
- Credentials: Protocol Buffers (proto3)
- Proofs: CBOR for ZK proof data
- Hashing: BLAKE3 for commitments and content addressing

### Cryptographic Algorithms

| Purpose | Algorithm | Key Size |
|---------|-----------|----------|
| Signing | Ed25519 | 256-bit |
| Key Exchange | X25519 | 256-bit |
| Encryption | ChaCha20-Poly1305 | 256-bit |
| Hashing | BLAKE3 | 256-bit |
| KDF | Argon2id | Configurable |
| ZKP Commitments | BLAKE3 | 256-bit |

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

### Why BLAKE3 for ZKP?

BLAKE3 is significantly faster than SHA-256 while providing equivalent security. Its use as both a hash function and commitment scheme simplifies the proof system while maintaining cryptographic soundness.

### Why Sigma Protocols?

Sigma protocols (with Fiat-Shamir heuristic) provide a well-understood framework for zero-knowledge proofs that don't require trusted setup or heavy cryptographic machinery. This keeps the implementation simple and auditable while providing real privacy guarantees.

## Backwards Compatibility

This is the initial protocol version. No backwards compatibility concerns.

## Reference Implementation

The reference implementation is located in the Veritas repository:
- Rust: `crates/` (8 crates)
- Go: `services/` (4 services)
- TypeScript: `sdks/typescript/`
- Protobuf: `proto/veritas/v1/`

## Copyright

This document is licensed under Apache License, Version 2.0.
