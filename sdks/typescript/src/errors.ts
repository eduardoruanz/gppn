/**
 * Error classes for the Veritas SDK.
 */

/** Base error class for all Veritas errors. */
export class VeritasError extends Error {
  /** A machine-readable error code. */
  public readonly code: string;

  constructor(message: string, code: string = "VERITAS_ERROR") {
    super(message);
    this.name = "VeritasError";
    this.code = code;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Error thrown when a network connection fails. */
export class ConnectionError extends VeritasError {
  constructor(message: string) {
    super(message, "CONNECTION_ERROR");
    this.name = "ConnectionError";
  }
}

/** Error thrown when a credential operation fails. */
export class CredentialError extends VeritasError {
  /** The credential ID associated with the error, if available. */
  public readonly credentialId?: string;

  constructor(message: string, credentialId?: string) {
    super(message, "CREDENTIAL_ERROR");
    this.name = "CredentialError";
    this.credentialId = credentialId;
  }
}

/** Error thrown when a zero-knowledge proof operation fails. */
export class ProofError extends VeritasError {
  constructor(message: string) {
    super(message, "PROOF_ERROR");
    this.name = "ProofError";
  }
}

/** Error thrown when identity operations fail. */
export class IdentityError extends VeritasError {
  constructor(message: string) {
    super(message, "IDENTITY_ERROR");
    this.name = "IdentityError";
  }
}

/** Error thrown when credential verification fails. */
export class VerificationError extends VeritasError {
  constructor(message: string) {
    super(message, "VERIFICATION_ERROR");
    this.name = "VerificationError";
  }
}
