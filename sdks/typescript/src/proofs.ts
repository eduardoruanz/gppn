/**
 * Zero-knowledge proof request and generation for the Veritas network.
 */

import { ProofError } from "./errors.js";
import type { ProofRequest, ZkProof } from "./types.js";
import { ProofType } from "./types.js";

/** Creates and manages zero-knowledge proof requests in the Veritas network. */
export class ProofRequester {
  private readonly _baseUrl: string;

  constructor(baseUrl: string) {
    this._baseUrl = baseUrl;
  }

  /**
   * Create an age proof request.
   *
   * @param verifierDid - The DID of the verifier requesting the proof.
   * @param minAge - The minimum age to prove.
   * @returns A ProofRequest to send to the credential holder.
   */
  createAgeProofRequest(verifierDid: string, minAge: number): ProofRequest {
    if (!verifierDid) {
      throw new ProofError("Verifier DID is required");
    }
    if (minAge <= 0 || minAge > 150) {
      throw new ProofError("Minimum age must be between 1 and 150");
    }

    return {
      id: generateRequestId(),
      verifier: verifierDid,
      proofType: ProofType.AgeProof,
      params: { min_age: minAge },
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Create a residency proof request.
   *
   * @param verifierDid - The DID of the verifier requesting the proof.
   * @param allowedCountries - List of allowed country codes.
   * @returns A ProofRequest to send to the credential holder.
   */
  createResidencyProofRequest(
    verifierDid: string,
    allowedCountries: string[]
  ): ProofRequest {
    if (!verifierDid) {
      throw new ProofError("Verifier DID is required");
    }
    if (!allowedCountries || allowedCountries.length === 0) {
      throw new ProofError("At least one allowed country is required");
    }

    return {
      id: generateRequestId(),
      verifier: verifierDid,
      proofType: ProofType.ResidencyProof,
      params: { allowed_countries: allowedCountries },
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Create a KYC level proof request.
   *
   * @param verifierDid - The DID of the verifier requesting the proof.
   * @param minLevel - The minimum KYC level to prove.
   * @returns A ProofRequest to send to the credential holder.
   */
  createKycLevelProofRequest(
    verifierDid: string,
    minLevel: number
  ): ProofRequest {
    if (!verifierDid) {
      throw new ProofError("Verifier DID is required");
    }
    if (minLevel < 1 || minLevel > 5) {
      throw new ProofError("KYC level must be between 1 and 5");
    }

    return {
      id: generateRequestId(),
      verifier: verifierDid,
      proofType: ProofType.KycLevelProof,
      params: { min_level: minLevel },
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Verify a zero-knowledge proof.
   *
   * This is a placeholder that performs basic structural validation.
   * In a real implementation this would verify the cryptographic proof.
   *
   * @param proof - The ZK proof to verify.
   * @returns true if the proof structure is valid.
   */
  verifyProof(proof: ZkProof): boolean {
    if (!proof.commitment || !proof.challenge || !proof.response) {
      return false;
    }
    return proof.valid;
  }
}

function generateRequestId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 10);
  return `pr_${timestamp}_${random}`;
}
