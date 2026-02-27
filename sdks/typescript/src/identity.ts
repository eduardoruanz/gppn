/**
 * Identity management using Ed25519 cryptography.
 */

import * as ed from "@noble/ed25519";
import { sha512 } from "@noble/hashes/sha512";
import { IdentityError } from "./errors.js";

// Configure ed25519 to use sha512 from @noble/hashes
ed.etc.sha512Sync = (...msgs: Uint8Array[]): Uint8Array => {
  const h = sha512.create();
  for (const msg of msgs) h.update(msg);
  return h.digest();
};

/** Represents a GPPN identity backed by an Ed25519 key pair. */
export class GppnIdentity {
  /** The Ed25519 private key (32 bytes). */
  public readonly privateKey: Uint8Array;
  /** The Ed25519 public key (32 bytes). */
  public readonly publicKey: Uint8Array;

  constructor(privateKey: Uint8Array, publicKey: Uint8Array) {
    this.privateKey = privateKey;
    this.publicKey = publicKey;
  }

  /**
   * Create a new random GPPN identity.
   * @returns A promise that resolves to a new GppnIdentity.
   */
  static async createIdentity(): Promise<GppnIdentity> {
    try {
      const privateKey = ed.utils.randomPrivateKey();
      const publicKey = await ed.getPublicKeyAsync(privateKey);
      return new GppnIdentity(privateKey, publicKey);
    } catch (err) {
      throw new IdentityError(
        `Failed to create identity: ${err instanceof Error ? err.message : String(err)}`
      );
    }
  }

  /**
   * Sign a message with this identity's private key.
   * @param message - The message bytes to sign.
   * @returns The Ed25519 signature (64 bytes).
   */
  async sign(message: Uint8Array): Promise<Uint8Array> {
    try {
      return await ed.signAsync(message, this.privateKey);
    } catch (err) {
      throw new IdentityError(
        `Failed to sign message: ${err instanceof Error ? err.message : String(err)}`
      );
    }
  }

  /**
   * Verify a signature against a message using a given public key.
   * @param signature - The signature to verify (64 bytes).
   * @param message - The original message bytes.
   * @param publicKey - The public key to verify against.
   * @returns true if the signature is valid, false otherwise.
   */
  static async verify(
    signature: Uint8Array,
    message: Uint8Array,
    publicKey: Uint8Array
  ): Promise<boolean> {
    try {
      return await ed.verifyAsync(signature, message, publicKey);
    } catch {
      return false;
    }
  }

  /**
   * Get the public key as a hex string, useful as a node/peer identifier.
   */
  get publicKeyHex(): string {
    return Buffer.from(this.publicKey).toString("hex");
  }
}
