/**
 * Main Veritas client — the primary entry point for the SDK.
 */

import { ConnectionError, CredentialError } from "./errors.js";
import { VeritasIdentity } from "./identity.js";
import { CredentialBuilder } from "./credentials.js";
import { ProofRequester } from "./proofs.js";
import { CredentialVerifier } from "./verification.js";
import { TrustManager } from "./trust.js";
import type {
  NodeStatus,
  PeerInfo,
  ProofRequest,
  TrustScore,
  VerifiableCredential,
  VerificationResult,
} from "./types.js";

/** Options for creating a VeritasClient. */
export interface VeritasClientOptions {
  /** The URL of the Veritas node to connect to. */
  url: string;
  /** An optional pre-existing identity (key pair) to use. */
  identity?: VeritasIdentity;
}

/**
 * The main Veritas client.
 *
 * Provides high-level methods for interacting with the Veritas network
 * including credential issuance, verification, proof requests, trust,
 * and identity management.
 */
export class VeritasClient {
  private readonly _url: string;
  private _identity: VeritasIdentity | undefined;
  private _connected: boolean = false;
  private readonly _proofRequester: ProofRequester;
  private readonly _credentialVerifier: CredentialVerifier;
  private readonly _trustManager: TrustManager;
  private readonly _credentials: Map<string, VerifiableCredential> = new Map();

  constructor(options: VeritasClientOptions) {
    this._url = options.url;
    this._identity = options.identity;
    this._proofRequester = new ProofRequester(options.url);
    this._credentialVerifier = new CredentialVerifier();
    this._trustManager = new TrustManager();
  }

  /** Whether the client is currently connected to the network. */
  get connected(): boolean {
    return this._connected;
  }

  /** The client's identity, if one has been set or created. */
  get identity(): VeritasIdentity | undefined {
    return this._identity;
  }

  /** The credential verifier instance for managing trusted issuers. */
  get verifier(): CredentialVerifier {
    return this._credentialVerifier;
  }

  /**
   * Connect to the Veritas network.
   * @throws ConnectionError if already connected.
   */
  async connect(): Promise<void> {
    if (this._connected) {
      throw new ConnectionError("Already connected");
    }
    this._connected = true;
  }

  /**
   * Disconnect from the Veritas network.
   * @throws ConnectionError if not connected.
   */
  async disconnect(): Promise<void> {
    if (!this._connected) {
      throw new ConnectionError("Not connected");
    }
    this._connected = false;
  }

  /**
   * Create a new identity for this client.
   * @returns The newly created VeritasIdentity.
   */
  async createIdentity(): Promise<VeritasIdentity> {
    this._identity = await VeritasIdentity.createIdentity();
    return this._identity;
  }

  /**
   * Issue a verifiable credential to a subject.
   *
   * @param subjectDid - DID of the subject to issue the credential to.
   * @param credentialType - The type(s) of credential.
   * @param claims - The claims to include in the credential.
   * @param expiresAt - Optional expiration timestamp (ISO 8601).
   * @returns The issued VerifiableCredential.
   * @throws ConnectionError if not connected.
   * @throws CredentialError if no identity is set.
   */
  async issueCredential(
    subjectDid: string,
    credentialType: string[],
    claims: Record<string, string | number | boolean>,
    expiresAt?: string
  ): Promise<VerifiableCredential> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to issue credentials");
    }
    if (!this._identity) {
      throw new CredentialError(
        "No identity set; call createIdentity() first"
      );
    }

    const builder = new CredentialBuilder()
      .issuer(this._identity.did)
      .subject(subjectDid)
      .claims(claims);

    for (const t of credentialType) {
      builder.type(t);
    }

    if (expiresAt) {
      builder.expires(expiresAt);
    }

    const credential = builder.build();
    this._credentials.set(credential.id, credential);

    return credential;
  }

  /**
   * Verify a verifiable credential.
   *
   * @param credential - The credential to verify.
   * @returns The verification result.
   */
  async verifyCredential(
    credential: VerifiableCredential
  ): Promise<VerificationResult> {
    return this._credentialVerifier.verify(credential);
  }

  /**
   * Get a stored credential by ID.
   *
   * @param credentialId - The ID of the credential.
   * @returns The credential, or undefined if not found.
   */
  async getCredential(
    credentialId: string
  ): Promise<VerifiableCredential | undefined> {
    return this._credentials.get(credentialId);
  }

  /**
   * List all stored credentials.
   * @returns An array of all stored credentials.
   */
  async listCredentials(): Promise<VerifiableCredential[]> {
    return Array.from(this._credentials.values());
  }

  /**
   * Create an age proof request.
   *
   * @param minAge - The minimum age to prove.
   * @returns A ProofRequest to send to the credential holder.
   * @throws ConnectionError if not connected.
   * @throws CredentialError if no identity is set.
   */
  async requestAgeProof(minAge: number): Promise<ProofRequest> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to request proofs");
    }
    if (!this._identity) {
      throw new CredentialError(
        "No identity set; call createIdentity() first"
      );
    }

    return this._proofRequester.createAgeProofRequest(
      this._identity.did,
      minAge
    );
  }

  /**
   * Create a residency proof request.
   *
   * @param allowedCountries - List of allowed country codes.
   * @returns A ProofRequest to send to the credential holder.
   * @throws ConnectionError if not connected.
   * @throws CredentialError if no identity is set.
   */
  async requestResidencyProof(
    allowedCountries: string[]
  ): Promise<ProofRequest> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to request proofs");
    }
    if (!this._identity) {
      throw new CredentialError(
        "No identity set; call createIdentity() first"
      );
    }

    return this._proofRequester.createResidencyProofRequest(
      this._identity.did,
      allowedCountries
    );
  }

  /**
   * Get the trust score for a peer.
   *
   * @param did - The DID of the peer.
   * @returns The trust score for the peer.
   */
  async getTrustScore(did: string): Promise<TrustScore> {
    return this._trustManager.getTrustScore(did);
  }

  /**
   * Update trust score after a verification interaction.
   *
   * @param did - The DID of the peer.
   * @param success - Whether the interaction was successful.
   * @returns The updated trust score.
   */
  async updateTrust(did: string, success: boolean): Promise<TrustScore> {
    return this._trustManager.updateScore(did, success);
  }

  /**
   * Get a list of known peers.
   *
   * Placeholder that returns an empty list.
   *
   * @returns An array of PeerInfo objects.
   * @throws ConnectionError if not connected.
   */
  async getPeers(): Promise<PeerInfo[]> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to get peers");
    }
    return [];
  }

  /**
   * Get the status of the connected node.
   *
   * Placeholder implementation.
   *
   * @returns The node status.
   * @throws ConnectionError if not connected.
   */
  async getNodeStatus(): Promise<NodeStatus> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to get node status");
    }

    return {
      did: this._identity?.did ?? "unknown",
      peerId: this._identity?.publicKeyHex ?? "unknown",
      connected: this._connected,
      peerCount: 0,
      version: "0.1.0",
      uptimeSeconds: 0,
    };
  }
}
