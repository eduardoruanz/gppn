/**
 * Core types for the Veritas SDK.
 */

/** States a verifiable credential can be in during its lifecycle. */
export enum CredentialState {
  /** Credential has been drafted but not yet issued. */
  Draft = "draft",
  /** Credential has been issued and is active. */
  Issued = "issued",
  /** Credential is active and verified. */
  Active = "active",
  /** Credential has been suspended temporarily. */
  Suspended = "suspended",
  /** Credential has been permanently revoked. */
  Revoked = "revoked",
  /** Credential has expired. */
  Expired = "expired",
}

/** Supported credential types in the Veritas network. */
export enum CredentialType {
  KycBasic = "KycBasic",
  KycEnhanced = "KycEnhanced",
  AgeVerification = "AgeVerification",
  Residency = "Residency",
  HumanityProof = "HumanityProof",
  Custom = "Custom",
}

/** Supported proof types for zero-knowledge proofs. */
export enum ProofType {
  AgeProof = "age",
  ResidencyProof = "residency",
  KycLevelProof = "kyc_level",
  HumanityProof = "humanity",
}

/** A single claim within a verifiable credential. */
export interface Claim {
  /** The name of the claim (e.g. "date_of_birth", "country"). */
  name: string;
  /** The value of the claim. */
  value: string | number | boolean;
}

/** A verifiable credential issued to a subject. */
export interface VerifiableCredential {
  /** Unique identifier for the credential. */
  id: string;
  /** DID of the issuer. */
  issuer: string;
  /** DID of the subject (credential holder). */
  subject: string;
  /** The type(s) of this credential. */
  credentialType: string[];
  /** Claims contained in this credential. */
  claims: Record<string, string | number | boolean>;
  /** Current state of the credential. */
  state: CredentialState;
  /** Issuance timestamp (ISO 8601). */
  issuedAt: string;
  /** Expiration timestamp (ISO 8601), if applicable. */
  expiresAt?: string;
  /** Cryptographic proof of the credential. */
  proof?: CredentialProof;
}

/** Cryptographic proof attached to a credential. */
export interface CredentialProof {
  /** The type of proof (e.g. "Ed25519Signature2020"). */
  type: string;
  /** When the proof was created (ISO 8601). */
  created: string;
  /** DID of the verification method used. */
  verificationMethod: string;
  /** The proof value (signature bytes as hex). */
  proofValue: string;
}

/** A verifiable presentation containing one or more credentials. */
export interface VerifiablePresentation {
  /** Unique identifier for the presentation. */
  id: string;
  /** DID of the holder presenting credentials. */
  holder: string;
  /** The credentials being presented. */
  credentials: VerifiableCredential[];
  /** When the presentation was created (ISO 8601). */
  createdAt: string;
  /** Cryptographic proof of the presentation. */
  proof?: CredentialProof;
}

/** A zero-knowledge proof response. */
export interface ZkProof {
  /** The type of proof (age, residency, kyc_level, etc.). */
  proofType: string;
  /** Whether the proof is valid. */
  valid: boolean;
  /** The commitment hash. */
  commitment: string;
  /** The challenge value. */
  challenge: string;
  /** The response value. */
  response: string;
  /** When the proof was generated (ISO 8601). */
  generatedAt: string;
}

/** A request for a zero-knowledge proof. */
export interface ProofRequest {
  /** Unique identifier for the proof request. */
  id: string;
  /** DID of the verifier requesting the proof. */
  verifier: string;
  /** The type of proof requested. */
  proofType: ProofType;
  /** Parameters for the proof (e.g. min_age, allowed_countries). */
  params: Record<string, string | number | string[]>;
  /** When the request was created (ISO 8601). */
  createdAt: string;
}

/** Trust score for a peer in the network. */
export interface TrustScore {
  /** DID of the peer. */
  did: string;
  /** Numeric trust score between 0.0 and 1.0. */
  score: number;
  /** Number of successful verifications. */
  successCount: number;
  /** Number of failed verifications. */
  failureCount: number;
  /** Timestamp of last update (ISO 8601). */
  lastUpdated: string;
}

/** Information about a peer node in the network. */
export interface PeerInfo {
  /** DID of the peer. */
  did: string;
  /** Peer ID in the libp2p network. */
  peerId: string;
  /** Network address of the peer. */
  address: string;
  /** Whether the peer is currently connected. */
  connected: boolean;
  /** Trust score for this peer. */
  trustScore?: TrustScore;
  /** Timestamp of last seen activity (ISO 8601). */
  lastSeen: string;
}

/** Status of the local Veritas node. */
export interface NodeStatus {
  /** DID of this node. */
  did: string;
  /** Peer ID in the libp2p network. */
  peerId: string;
  /** Whether the node is currently connected to the network. */
  connected: boolean;
  /** Number of connected peers. */
  peerCount: number;
  /** Node version string. */
  version: string;
  /** Node uptime in seconds. */
  uptimeSeconds: number;
}

/** A credential schema defining allowed claims. */
export interface CredentialSchema {
  /** Unique identifier for the schema. */
  id: string;
  /** Human-readable name. */
  name: string;
  /** Schema version. */
  version: string;
  /** Allowed claim names. */
  claims: string[];
}

/** Result of a credential verification. */
export interface VerificationResult {
  /** Whether the credential is valid overall. */
  valid: boolean;
  /** Individual check results. */
  checks: VerificationCheck[];
}

/** A single verification check result. */
export interface VerificationCheck {
  /** Name of the check (e.g. "signature", "expiration", "issuer_trust"). */
  name: string;
  /** Whether this check passed. */
  passed: boolean;
  /** Details about the check result. */
  detail: string;
}
