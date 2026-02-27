# SDK Integration Guide

## TypeScript SDK

The `@gppn/sdk` package provides a high-level client for interacting with GPPN nodes.

### Installation

```bash
npm install @gppn/sdk
```

### Quick Start

```typescript
import { GppnClient } from "@gppn/sdk";

// Connect to a GPPN node
const client = new GppnClient({ url: "http://localhost:9000" });
await client.connect();

// Create or load an identity
const identity = await client.createIdentity();
console.log(`My DID: ${identity.did}`);

// Send a payment
const payment = await client.sendPayment(
  "did:gppn:key:recipient_public_key",
  "100.00",
  { code: "USD", decimals: 2 },
  "Invoice #1234"
);

console.log(`Payment ID: ${payment.id}`);
console.log(`Status: ${payment.status}`);

// Check payment status
const status = await client.getPaymentStatus(payment.id);
console.log(`Updated status: ${status?.status}`);

// Disconnect
await client.disconnect();
```

### Client Configuration

```typescript
import { GppnClient, GppnIdentity } from "@gppn/sdk";

// Use an existing keypair
const keypair = await GppnIdentity.createIdentity();
const client = new GppnClient({
  url: "http://localhost:9000",
  keypair,
});
```

### Identity Management

```typescript
// Create a new identity (Ed25519 keypair)
const identity = await GppnIdentity.createIdentity();

// Sign data
const message = new TextEncoder().encode("Hello GPPN");
const signature = await identity.sign(message);

// Verify signature
const valid = await GppnIdentity.verify(
  signature,
  message,
  identity.publicKey
);

// Get DID
const did = identity.did; // "did:gppn:key:<base64url_pubkey>"
```

### Payment Operations

```typescript
// Send a payment
const payment = await client.sendPayment(
  recipient,  // DID or public key
  "50.00",    // Amount as string
  { code: "USD", decimals: 2 },  // Currency
  "Optional memo"
);

// Payment response
// {
//   id: "pay_...",
//   sender: "did:gppn:key:...",
//   recipient: "did:gppn:key:...",
//   amount: { value: "50.00", currency: { code: "USD", decimals: 2 } },
//   status: "created",
//   memo: "Optional memo",
//   createdAt: "2025-01-15T10:30:00Z"
// }

// Check status
const status = await client.getPaymentStatus(payment.id);
```

### Route Discovery

```typescript
// Find routes to a recipient
const routes = await client.findRoutes("did:gppn:key:recipient", {
  value: "100",
  currency: { code: "USD", decimals: 2 },
});

// Routes are sorted by score (best first)
for (const route of routes) {
  console.log(`Path: ${route.path.join(" → ")}`);
  console.log(`Fee: ${route.estimatedFee}`);
  console.log(`Hops: ${route.hopCount}`);
  console.log(`Score: ${route.score}`);
}
```

### Trust Scores

```typescript
// Get trust score for a peer
const trust = await client.getTrustScore("peer_id");
console.log(`Trust: ${trust.score}`);  // 0.0 to 1.0
console.log(`Components: ${JSON.stringify(trust.components)}`);
```

### Network Operations

```typescript
// List connected peers
const peers = await client.getPeers();
for (const peer of peers) {
  console.log(`${peer.id}: ${peer.addresses.join(", ")}`);
}

// Get node status
const status = await client.getNodeStatus();
console.log(`Version: ${status.version}`);
console.log(`Connected: ${status.connected}`);
console.log(`Peer count: ${status.peerCount}`);
```

### Error Handling

```typescript
import { ConnectionError, PaymentError, IdentityError } from "@gppn/sdk";

try {
  await client.sendPayment(recipient, amount, currency);
} catch (error) {
  if (error instanceof ConnectionError) {
    console.error("Not connected to node");
  } else if (error instanceof PaymentError) {
    console.error("Payment failed:", error.message);
  } else if (error instanceof IdentityError) {
    console.error("Identity issue:", error.message);
  }
}
```
