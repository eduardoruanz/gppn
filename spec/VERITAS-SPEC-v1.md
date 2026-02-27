# Veritas Protocol Specification v1.0

## Abstract

Veritas is a decentralized, open protocol for privacy-preserving identity verification, verifiable credentials, and zero-knowledge proofs. Veritas enables any identity system to interoperate through a common protocol layer, providing AI-resistant proof of humanity without centralized authorities.

## 1. Introduction

### 1.1 Motivation

Today's identity infrastructure faces two fundamental challenges: centralization and privacy. Veritas addresses both by providing a protocol-level standard for:

- **Verifiable Credentials**: A universal format for issuing and verifying identity attributes
- **Zero-Knowledge Proofs**: Prove properties about credentials without revealing underlying data
- **Decentralized Identity**: Self-sovereign DIDs that don't depend on any central registry
- **Trust Scoring**: Reputation built from verification outcomes and peer attestations
- **Transport**: Peer-to-peer communication using proven libp2p networking primitives

### 1.2 Design Principles

1. **Protocol, not platform**: Veritas defines interfaces, not implementations
2. **Privacy by default**: Zero-knowledge proofs for all verification flows
3. **Trust-minimized**: Cryptographic proofs replace institutional trust
4. **Incrementally deployable**: Nodes can join with partial capability
5. **AI-resistant**: Multi-signal humanity proofs that resist automated attacks

## 2. Protocol Layers

### 2.1 Layer Stack

```
┌─────────────────────────────────────┐
│  VCL: Verifiable Credential Layer   │  Application
├─────────────────────────────────────┤
│  ZPL: Zero-Knowledge Proof Layer    │  Proofs
├─────────────────────────────────────┤
│  TIL: Trust & Identity Layer        │  Identity
├─────────────────────────────────────┤
│  OGL: Overlay & Gossip Layer        │  Transport
└─────────────────────────────────────┘
```

## 3. Verifiable Credential Layer (VCL)

### 3.1 Credential Format

All credentials follow W3C Verifiable Credentials compatible format:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique credential identifier |
| `issuer` | string | Issuer DID |
| `subject` | string | Subject DID |
| `credential_type` | string[] | Credential type tags |
| `claims` | map | Key-value claim pairs |
| `state` | CredentialState | Current lifecycle state |
| `issued_at` | timestamp | Issuance time |
| `expires_at` | timestamp | Expiration time (optional) |
| `proof` | CredentialProof | Ed25519 signature |

### 3.2 Credential State Machine

```
         ┌────────┐
         │ Draft  │
         └───┬────┘
             │
         ┌───▼────┐
    ┌────│ Issued │────┐
    │    └───┬────┘    │
    │        │         │
    │    ┌───▼────┐    │
    │    │ Active │    │
    │    └─┬──┬───┘    │
    │      │  │        │
┌───▼──┐  │  │  ┌─────▼───┐
│Suspen│  │  │  │ Revoked │
│ded   │  │  │  └─────────┘
└───┬──┘  │  │
    │     │  │
    └─────┘  │
         ┌───▼────┐
         │Expired │
         └────────┘
```

Valid transitions:
- `Draft → Issued` — Credential signed by issuer
- `Issued → Active` — Credential activated
- `Active → Suspended` — Temporarily suspended
- `Suspended → Active` — Reinstated
- `Active → Revoked` — Permanently revoked
- `Issued/Active → Expired` — TTL exceeded

### 3.3 Schemas

Built-in credential schemas:

| Schema | Claims |
|--------|--------|
| `kyc-basic-v1` | full_name, date_of_birth, country, kyc_level |
| `age-verification-v1` | date_of_birth, verified_at |
| `residency-v1` | country, region, verified_at |
| `humanity-proof-v1` | verification_method, confidence_score, verified_at |

## 4. Zero-Knowledge Proof Layer (ZPL)

### 4.1 Commitment Scheme

BLAKE3-based commitments: `C = BLAKE3(value || nonce)`

The prover can later reveal the value and nonce to verify the commitment, or use Sigma protocol proofs to demonstrate properties without revealing the value.

### 4.2 Proof Types

#### Age Proof
Proves `age >= min_age` without revealing date of birth.
- Prover commits to DOB
- Generates Sigma protocol proof that `current_date - DOB >= min_age * 365`
- Verifier checks proof against commitment

#### Residency Proof
Proves `country ∈ allowed_set` without revealing address.
- Prover commits to country code
- Generates Merkle set membership proof against allowed country set hash
- Verifier checks Merkle proof

#### KYC Level Proof
Proves `kyc_level >= required_level` without revealing exact level.
- Prover commits to KYC level
- Generates range proof via Sigma protocol
- Verifier checks proof

#### Humanity Proof Bundle
Composite proof combining multiple signals:
- Age proof (prove human age range)
- Social vouching attestations
- Cross-platform verification
- Confidence score from trust graph

### 4.3 Sigma Protocol Flow

```
Prover                          Verifier
  │                                │
  │──── Commitment (C) ──────────>│
  │                                │
  │<──── Challenge (e) ───────────│
  │                                │
  │──── Response (s) ────────────>│
  │                                │
  │        Verify: C, e, s         │
```

Using Fiat-Shamir heuristic for non-interactive proofs: `e = BLAKE3(C || context)`

## 5. Trust & Identity Layer (TIL)

### 5.1 Decentralized Identifiers (DIDs)

Format: `did:veritas:<method>:<identifier>`

Methods:
- `key` — Ed25519 public key based
- `web` — DNS-based resolution

### 5.2 DID Documents

```json
{
  "id": "did:veritas:key:z6Mk...",
  "verificationMethod": [{
    "id": "did:veritas:key:z6Mk...#keys-1",
    "type": "Ed25519VerificationKey2020",
    "publicKeyMultibase": "z6Mk..."
  }],
  "authentication": ["did:veritas:key:z6Mk...#keys-1"],
  "service": [{
    "type": "VeritasNode",
    "serviceEndpoint": "/ip4/203.0.113.1/tcp/9000"
  }]
}
```

### 5.3 Trust Scoring

Composite score (0.0 to 1.0):

| Component | Weight | Description |
|-----------|--------|-------------|
| Verification Success | 30% | Successful verification rate |
| Attestations | 25% | Peer trust endorsements |
| Uptime | 15% | Node availability |
| Age | 15% | Time in network |
| Credential Volume | 15% | Credentials issued/verified |

Global trust scores are computed using an EigenTrust-inspired iterative algorithm over the trust graph.

## 6. Overlay & Gossip Layer (OGL)

### 6.1 Transport Stack

```
TCP → Noise (encryption) → Yamux (multiplexing)
```

### 6.2 Protocols

| Protocol | ID | Purpose |
|----------|----|---------|
| GossipSub | `/meshsub/1.1.0` | Broadcast (credentials, proof requests) |
| Kademlia | `/ipfs/kad/1.0.0` | Peer discovery, DHT |
| Identify | `/ipfs/id/1.0.0` | Peer information exchange |
| Veritas Req/Res | `/veritas/req/1.0.0` | Direct peer messaging (CBOR) |
| mDNS | N/A | Local network discovery |

### 6.3 GossipSub Topics

- `/veritas/credentials/v1` — Credential announcements
- `/veritas/did-announce/v1` — DID registration broadcasts
- `/veritas/proof-requests/v1` — Proof request/response
- `/veritas/peer-announce/v1` — Peer discovery

## 7. Security Considerations

- **Replay Protection**: Unique credential IDs and timestamps
- **Signature Verification**: All credentials must have valid Ed25519 signatures
- **Zero-Knowledge**: Proof verification reveals no underlying data
- **Commitment Binding**: BLAKE3 commitments are computationally binding
- **Sybil Resistance**: EigenTrust scoring + humanity proof bundles
- **DoS Mitigation**: Rate limiting, proof-of-work for registration

## 8. Wire Format

All protocol messages use:
- **Protobuf** (prost) for structured data serialization
- **CBOR** for libp2p request-response codec
- **GossipSub** message format for broadcast messages

## 9. Versioning

The protocol uses semantic versioning. Breaking changes increment the major version in protocol identifiers (e.g., `/veritas/req/2.0.0`).
