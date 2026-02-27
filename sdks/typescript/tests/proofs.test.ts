import { describe, it, expect } from "vitest";
import { ProofRequester } from "../src/proofs.js";
import { ProofError } from "../src/errors.js";
import { ProofType } from "../src/types.js";

describe("ProofRequester", () => {
  const requester = new ProofRequester("http://localhost:9001");

  describe("createAgeProofRequest", () => {
    it("should create a valid age proof request", () => {
      const req = requester.createAgeProofRequest(
        "did:veritas:key:verifier123",
        18
      );

      expect(req.id).toMatch(/^pr_/);
      expect(req.verifier).toBe("did:veritas:key:verifier123");
      expect(req.proofType).toBe(ProofType.AgeProof);
      expect(req.params.min_age).toBe(18);
      expect(req.createdAt).toBeTruthy();
    });

    it("should reject empty verifier DID", () => {
      expect(() => requester.createAgeProofRequest("", 18)).toThrow(
        ProofError
      );
    });

    it("should reject invalid age values", () => {
      expect(() =>
        requester.createAgeProofRequest("did:veritas:key:v", 0)
      ).toThrow(ProofError);
      expect(() =>
        requester.createAgeProofRequest("did:veritas:key:v", 151)
      ).toThrow(ProofError);
    });
  });

  describe("createResidencyProofRequest", () => {
    it("should create a valid residency proof request", () => {
      const req = requester.createResidencyProofRequest(
        "did:veritas:key:verifier123",
        ["US", "BR", "DE"]
      );

      expect(req.id).toMatch(/^pr_/);
      expect(req.verifier).toBe("did:veritas:key:verifier123");
      expect(req.proofType).toBe(ProofType.ResidencyProof);
      expect(req.params.allowed_countries).toEqual(["US", "BR", "DE"]);
    });

    it("should reject empty verifier DID", () => {
      expect(() =>
        requester.createResidencyProofRequest("", ["US"])
      ).toThrow(ProofError);
    });

    it("should reject empty country list", () => {
      expect(() =>
        requester.createResidencyProofRequest("did:veritas:key:v", [])
      ).toThrow(ProofError);
    });
  });

  describe("createKycLevelProofRequest", () => {
    it("should create a valid KYC level proof request", () => {
      const req = requester.createKycLevelProofRequest(
        "did:veritas:key:verifier123",
        2
      );

      expect(req.id).toMatch(/^pr_/);
      expect(req.proofType).toBe(ProofType.KycLevelProof);
      expect(req.params.min_level).toBe(2);
    });

    it("should reject invalid KYC levels", () => {
      expect(() =>
        requester.createKycLevelProofRequest("did:veritas:key:v", 0)
      ).toThrow(ProofError);
      expect(() =>
        requester.createKycLevelProofRequest("did:veritas:key:v", 6)
      ).toThrow(ProofError);
    });
  });

  describe("verifyProof", () => {
    it("should accept a valid proof", () => {
      const valid = requester.verifyProof({
        proofType: "age",
        valid: true,
        commitment: "abc123",
        challenge: "def456",
        response: "ghi789",
        generatedAt: new Date().toISOString(),
      });

      expect(valid).toBe(true);
    });

    it("should reject a proof with missing commitment", () => {
      const valid = requester.verifyProof({
        proofType: "age",
        valid: true,
        commitment: "",
        challenge: "def456",
        response: "ghi789",
        generatedAt: new Date().toISOString(),
      });

      expect(valid).toBe(false);
    });

    it("should reject a proof marked as invalid", () => {
      const valid = requester.verifyProof({
        proofType: "age",
        valid: false,
        commitment: "abc123",
        challenge: "def456",
        response: "ghi789",
        generatedAt: new Date().toISOString(),
      });

      expect(valid).toBe(false);
    });
  });
});
