/**
 * Credential verification for the Veritas network.
 */

import { VerificationError } from "./errors.js";
import type {
  VerifiableCredential,
  VerificationCheck,
  VerificationResult,
} from "./types.js";
import { CredentialState } from "./types.js";

/** Verifies verifiable credentials and presentations in the Veritas network. */
export class CredentialVerifier {
  private readonly _trustedIssuers: Set<string> = new Set();

  /**
   * Add a trusted issuer DID.
   * @param issuerDid - The DID of the trusted issuer.
   */
  addTrustedIssuer(issuerDid: string): void {
    this._trustedIssuers.add(issuerDid);
  }

  /**
   * Remove a trusted issuer DID.
   * @param issuerDid - The DID of the issuer to remove.
   */
  removeTrustedIssuer(issuerDid: string): void {
    this._trustedIssuers.delete(issuerDid);
  }

  /**
   * Check if an issuer DID is trusted.
   * @param issuerDid - The DID to check.
   */
  isTrustedIssuer(issuerDid: string): boolean {
    return this._trustedIssuers.has(issuerDid);
  }

  /**
   * Verify a verifiable credential.
   *
   * Performs structural validation, state checks, expiration checks,
   * and issuer trust checks. In a real implementation this would also
   * verify the cryptographic signature.
   *
   * @param credential - The credential to verify.
   * @returns The verification result with individual check details.
   */
  verify(credential: VerifiableCredential): VerificationResult {
    if (!credential) {
      throw new VerificationError("Credential is required for verification");
    }

    const checks: VerificationCheck[] = [];

    // Check: credential has required fields
    const structureValid =
      !!credential.id &&
      !!credential.issuer &&
      !!credential.subject &&
      credential.credentialType.length > 0;
    checks.push({
      name: "structure",
      passed: structureValid,
      detail: structureValid
        ? "Credential has all required fields"
        : "Credential is missing required fields",
    });

    // Check: credential state is active/issued
    const stateValid =
      credential.state === CredentialState.Issued ||
      credential.state === CredentialState.Active;
    checks.push({
      name: "state",
      passed: stateValid,
      detail: stateValid
        ? `Credential state is ${credential.state}`
        : `Credential state ${credential.state} is not valid for verification`,
    });

    // Check: credential is not expired
    let expirationValid = true;
    if (credential.expiresAt) {
      expirationValid = new Date(credential.expiresAt) > new Date();
    }
    checks.push({
      name: "expiration",
      passed: expirationValid,
      detail: expirationValid
        ? "Credential has not expired"
        : "Credential has expired",
    });

    // Check: issuer is trusted
    const issuerTrusted = this._trustedIssuers.has(credential.issuer);
    checks.push({
      name: "issuer_trust",
      passed: issuerTrusted,
      detail: issuerTrusted
        ? `Issuer ${credential.issuer} is trusted`
        : `Issuer ${credential.issuer} is not in the trusted issuers list`,
    });

    // Check: has proof (signature)
    const hasProof = !!credential.proof;
    checks.push({
      name: "signature",
      passed: hasProof,
      detail: hasProof
        ? "Credential has cryptographic proof"
        : "Credential is missing cryptographic proof",
    });

    const valid = checks.every((c) => c.passed);

    return { valid, checks };
  }
}
