# Veritas Architecture Overview

## Protocol Layers

Veritas implements a 4-layer architecture for decentralized identity:

### Layer 1: OGL — Overlay & Gossip Layer

**Crate**: `veritas-network`

The foundation layer handles peer-to-peer networking using libp2p:

- **Transport**: TCP + Noise encryption + Yamux multiplexing
- **Discovery**: Kademlia DHT for peer discovery, mDNS for local networks
- **Messaging**: GossipSub for broadcast (credential announcements, proof requests)
- **Direct**: Request-response protocol with CBOR serialization for peer-to-peer messages

Key components:
- `VeritasNode` — manages the libp2p swarm and event loop
- `VeritasBehaviour` — composed libp2p behaviour (gossipsub + Kademlia + mDNS + identify + request-response)
- `TopicManager` — GossipSub topic management for `/veritas/credentials/v1` and `/veritas/proof-requests/v1`

### Layer 2: TIL — Trust & Identity Layer

**Crate**: `veritas-identity`

Decentralized identity and trust scoring:

- **DIDs**: W3C-compatible Decentralized Identifiers (`did:veritas:<method>:<id>`)
- **DID Documents**: Public keys, service endpoints, authentication methods
- **Verifiable Credentials**: Issuer-signed attestations about subjects
- **Trust Graph**: Directed weighted graph with EigenTrust-inspired iterative scoring
- **Humanity Verification**: Multi-signal proof of humanity (social vouching, trusted issuers, cross-platform)

### Layer 3: ZPL — Zero-Knowledge Proof Layer

**Crates**: `veritas-proof`, `veritas-crypto`

Privacy-preserving proofs using BLAKE3 commitments and Sigma protocols:

- **Age Proofs**: Prove age >= threshold without revealing date of birth
- **Residency Proofs**: Prove country membership via Merkle set proof
- **KYC Level Proofs**: Prove KYC level >= required without revealing exact level
- **Humanity Proofs**: Composite bundle of multiple proof types + social attestations
- **Selective Disclosure**: Reveal only chosen claims from a credential

### Layer 4: VCL — Verifiable Credential Layer

**Crate**: `veritas-credentials`

Complete credential lifecycle management:

- **SchemaRegistry**: Define and validate credential schemas (KYC, age verification, residency)
- **CredentialIssuer**: Sign and issue W3C-compatible verifiable credentials
- **CredentialWallet**: Store, list, and present credentials
- **CredentialVerifier**: Verify signatures, check expiration, validate issuer trust
- **VerifiablePresentation**: Bundle credentials for selective presentation

## Cryptography

**Crate**: `veritas-crypto`

- **Signing**: Ed25519 (ed25519-dalek v2) for credential and message signatures
- **Key Exchange**: X25519 Diffie-Hellman for shared secret derivation
- **Encryption**: ChaCha20-Poly1305 AEAD for encrypted metadata
- **Hashing**: BLAKE3 for commitments, content addressing, and Merkle proofs
- **KDF**: Argon2id for key derivation from passwords
- **ZKP**: BLAKE3 commitment schemes with Fiat-Shamir Sigma protocol proofs
- **Memory Safety**: `zeroize` crate for private key material cleanup on drop

## Node Architecture

**Crate**: `veritas-node`

The full node binary orchestrates all layers:

```
┌─────────────────────────────────┐
│          veritas-node           │
├─────────────────────────────────┤
│  REST API (port 9001)           │
│  Prometheus metrics (port 9002) │
├─────────────────────────────────┤
│  Verifiable Credential Layer    │
│  Zero-Knowledge Proof Layer     │
│  Trust & Identity Layer         │
│  Overlay & Gossip Layer         │
├─────────────────────────────────┤
│  RocksDB Storage                │
│  (credentials|schemas|proofs|   │
│   identity|state|peers)         │
└─────────────────────────────────┘
```

Init sequence: keypair → storage (RocksDB) → identity (DID) → network → credentials → proofs → API → metrics → event loop

## Data Flow: Verifying Identity

1. **Issuer** creates a verifiable credential for a subject (e.g., KYC Basic with name, DOB, country)
2. **Holder** stores the credential in their wallet
3. **Verifier** requests proof: "prove age >= 18 and resident of US or BR"
4. **Holder** generates zero-knowledge proofs from their credential claims:
   - Age proof: BLAKE3 commitment of DOB + Sigma protocol range proof
   - Residency proof: Merkle set membership proof against allowed countries
5. **Verifier** checks the proofs cryptographically — no personal data revealed
6. **Verifier** attests trust in the holder via the trust graph
7. Trust scores propagate through the network via EigenTrust
