# SDK Integration Guide

## TypeScript SDK

The `@veritas/sdk` package provides a high-level client for interacting with Veritas nodes.

### Installation

```bash
npm install @veritas/sdk
```

### Quick Start

```typescript
import { VeritasClient } from "@veritas/sdk";

// Connect to a Veritas node
const client = new VeritasClient({ url: "http://localhost:9001" });
await client.connect();

// Create or load an identity
const identity = await client.createIdentity();
console.log(`My DID: ${identity.did}`);

// Issue a credential
const credential = await client.issueCredential(
  "did:veritas:key:subject_public_key",
  ["KycBasic"],
  { full_name: "Alice Smith", country: "US", kyc_level: 2 }
);

console.log(`Credential ID: ${credential.id}`);
console.log(`State: ${credential.state}`);

// Disconnect
await client.disconnect();
```

### Client Configuration

```typescript
import { VeritasClient, VeritasIdentity } from "@veritas/sdk";

// Use an existing identity
const identity = await VeritasIdentity.createIdentity();
const client = new VeritasClient({
  url: "http://localhost:9001",
  identity,
});
```

### Identity Management

```typescript
// Create a new identity (Ed25519 keypair + DID)
const identity = await VeritasIdentity.createIdentity();
console.log(`DID: ${identity.did}`); // "did:veritas:key:<hex_pubkey>"

// Sign data
const message = new TextEncoder().encode("Hello Veritas");
const signature = await identity.sign(message);

// Verify signature
const valid = await VeritasIdentity.verify(
  signature,
  message,
  identity.publicKey
);
```

### Credential Operations

```typescript
// Issue a credential
const credential = await client.issueCredential(
  "did:veritas:key:subject_did",
  ["KycBasic", "AgeVerification"],
  { full_name: "Bob Jones", date_of_birth: "1990-01-15", country: "BR" },
  "2027-12-31T23:59:59Z" // optional expiration
);

// Credential response
// {
//   id: "vc_...",
//   issuer: "did:veritas:key:...",
//   subject: "did:veritas:key:...",
//   credentialType: ["KycBasic", "AgeVerification"],
//   claims: { full_name: "Bob Jones", ... },
//   state: "issued",
//   issuedAt: "2026-01-15T10:30:00Z"
// }

// List stored credentials
const credentials = await client.listCredentials();

// Get a specific credential
const vc = await client.getCredential(credential.id);
```

### Verification

```typescript
import { CredentialVerifier } from "@veritas/sdk";

// Set up verifier with trusted issuers
const verifier = client.verifier;
verifier.addTrustedIssuer("did:veritas:key:trusted_issuer_did");

// Verify a credential
const result = await client.verifyCredential(credential);
console.log(`Valid: ${result.valid}`);
for (const check of result.checks) {
  console.log(`  ${check.name}: ${check.passed ? "PASS" : "FAIL"} — ${check.detail}`);
}
```

### Zero-Knowledge Proof Requests

```typescript
// Request an age proof (prove age >= 18)
const ageRequest = await client.requestAgeProof(18);

// Request a residency proof
const residencyRequest = await client.requestResidencyProof(["US", "BR", "DE"]);

// Proof requests are sent to credential holders who generate ZK proofs
// without revealing their actual data
```

### Trust Scores

```typescript
// Get trust score for a peer
const trust = await client.getTrustScore("did:veritas:key:peer_did");
console.log(`Trust: ${trust.score}`); // 0.0 to 1.0

// Update trust after verification
await client.updateTrust("did:veritas:key:peer_did", true);
```

### Network Operations

```typescript
// List connected peers
const peers = await client.getPeers();

// Get node status
const status = await client.getNodeStatus();
console.log(`DID: ${status.did}`);
console.log(`Connected: ${status.connected}`);
console.log(`Peer count: ${status.peerCount}`);
```

### Error Handling

```typescript
import {
  ConnectionError,
  CredentialError,
  ProofError,
  IdentityError,
  VerificationError,
} from "@veritas/sdk";

try {
  await client.issueCredential(subjectDid, types, claims);
} catch (error) {
  if (error instanceof ConnectionError) {
    console.error("Not connected to node");
  } else if (error instanceof CredentialError) {
    console.error("Credential operation failed:", error.message);
  } else if (error instanceof IdentityError) {
    console.error("Identity issue:", error.message);
  }
}
```
