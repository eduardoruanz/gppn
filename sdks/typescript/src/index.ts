/**
 * Veritas TypeScript SDK
 *
 * A complete SDK for interacting with the Veritas decentralized identity network.
 */

// Core types
export {
  CredentialState,
  CredentialType,
  ProofType,
  type Claim,
  type VerifiableCredential,
  type CredentialProof,
  type VerifiablePresentation,
  type ZkProof,
  type ProofRequest,
  type TrustScore,
  type PeerInfo,
  type NodeStatus,
  type CredentialSchema,
  type VerificationResult,
  type VerificationCheck,
} from "./types.js";

// Error classes
export {
  VeritasError,
  ConnectionError,
  CredentialError,
  ProofError,
  IdentityError,
  VerificationError,
} from "./errors.js";

// Identity management
export { VeritasIdentity } from "./identity.js";

// Credential building
export { CredentialBuilder } from "./credentials.js";

// Proof requests
export { ProofRequester } from "./proofs.js";

// Credential verification
export { CredentialVerifier } from "./verification.js";

// Trust management
export { TrustManager } from "./trust.js";

// Main client
export { VeritasClient, type VeritasClientOptions } from "./client.js";
