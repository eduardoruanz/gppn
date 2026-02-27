/**
 * Credential building and issuance for Veritas verifiable credentials.
 */

import { CredentialError } from "./errors.js";
import type { VerifiableCredential } from "./types.js";
import { CredentialState } from "./types.js";

/** Builder class for constructing Veritas verifiable credentials with a fluent API. */
export class CredentialBuilder {
  private _issuer?: string;
  private _subject?: string;
  private _credentialType: string[] = [];
  private _claims: Record<string, string | number | boolean> = {};
  private _expiresAt?: string;

  /**
   * Set the issuer DID.
   * @param issuer - The DID of the credential issuer.
   */
  issuer(issuer: string): this {
    this._issuer = issuer;
    return this;
  }

  /**
   * Set the subject DID.
   * @param subject - The DID of the credential subject (holder).
   */
  subject(subject: string): this {
    this._subject = subject;
    return this;
  }

  /**
   * Add a credential type.
   * @param type - The credential type to add.
   */
  type(type: string): this {
    this._credentialType.push(type);
    return this;
  }

  /**
   * Add a claim to the credential.
   * @param name - The claim name.
   * @param value - The claim value.
   */
  claim(name: string, value: string | number | boolean): this {
    this._claims[name] = value;
    return this;
  }

  /**
   * Set multiple claims at once.
   * @param claims - A record of claim name-value pairs.
   */
  claims(claims: Record<string, string | number | boolean>): this {
    Object.assign(this._claims, claims);
    return this;
  }

  /**
   * Set the expiration date.
   * @param expiresAt - ISO 8601 timestamp for credential expiration.
   */
  expires(expiresAt: string): this {
    this._expiresAt = expiresAt;
    return this;
  }

  /**
   * Validate the current builder state.
   * @returns An array of validation error messages. Empty if valid.
   */
  validate(): string[] {
    const errors: string[] = [];

    if (!this._issuer) {
      errors.push("issuer is required");
    }
    if (!this._subject) {
      errors.push("subject is required");
    }
    if (this._credentialType.length === 0) {
      errors.push("at least one credential type is required");
    }
    if (Object.keys(this._claims).length === 0) {
      errors.push("at least one claim is required");
    }
    if (this._issuer && this._subject && this._issuer === this._subject) {
      errors.push("issuer and subject must be different");
    }

    return errors;
  }

  /**
   * Build the verifiable credential.
   * @returns A fully constructed VerifiableCredential.
   * @throws CredentialError if validation fails.
   */
  build(): VerifiableCredential {
    const errors = this.validate();
    if (errors.length > 0) {
      throw new CredentialError(`Invalid credential: ${errors.join(", ")}`);
    }

    const now = new Date().toISOString();

    return {
      id: generateCredentialId(),
      issuer: this._issuer!,
      subject: this._subject!,
      credentialType: [...this._credentialType],
      claims: { ...this._claims },
      state: CredentialState.Issued,
      issuedAt: now,
      expiresAt: this._expiresAt,
    };
  }
}

/**
 * Generate a unique credential ID.
 * Uses a combination of timestamp and random bytes.
 */
function generateCredentialId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 10);
  return `vc_${timestamp}_${random}`;
}
