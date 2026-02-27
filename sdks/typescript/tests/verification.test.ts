import { describe, it, expect } from "vitest";
import { CredentialVerifier } from "../src/verification.js";
import { CredentialState } from "../src/types.js";
import { VerificationError } from "../src/errors.js";
import type { VerifiableCredential } from "../src/types.js";

function makeCredential(
  overrides: Partial<VerifiableCredential> = {}
): VerifiableCredential {
  return {
    id: "vc_test_123",
    issuer: "did:veritas:key:issuer123",
    subject: "did:veritas:key:subject456",
    credentialType: ["KycBasic"],
    claims: { full_name: "Alice Smith" },
    state: CredentialState.Issued,
    issuedAt: new Date().toISOString(),
    proof: {
      type: "Ed25519Signature2020",
      created: new Date().toISOString(),
      verificationMethod: "did:veritas:key:issuer123#key-1",
      proofValue: "abcdef1234567890",
    },
    ...overrides,
  };
}

describe("CredentialVerifier", () => {
  it("should verify a valid credential with trusted issuer", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(makeCredential());

    expect(result.valid).toBe(true);
    expect(result.checks.length).toBe(5);
    expect(result.checks.every((c) => c.passed)).toBe(true);
  });

  it("should fail verification for untrusted issuer", () => {
    const verifier = new CredentialVerifier();
    // Do not add issuer as trusted

    const result = verifier.verify(makeCredential());

    expect(result.valid).toBe(false);
    const issuerCheck = result.checks.find((c) => c.name === "issuer_trust");
    expect(issuerCheck?.passed).toBe(false);
  });

  it("should fail verification for revoked credential", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(
      makeCredential({ state: CredentialState.Revoked })
    );

    expect(result.valid).toBe(false);
    const stateCheck = result.checks.find((c) => c.name === "state");
    expect(stateCheck?.passed).toBe(false);
  });

  it("should fail verification for expired credential", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(
      makeCredential({ expiresAt: "2020-01-01T00:00:00Z" })
    );

    expect(result.valid).toBe(false);
    const expirationCheck = result.checks.find(
      (c) => c.name === "expiration"
    );
    expect(expirationCheck?.passed).toBe(false);
  });

  it("should fail verification for credential without proof", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(makeCredential({ proof: undefined }));

    expect(result.valid).toBe(false);
    const sigCheck = result.checks.find((c) => c.name === "signature");
    expect(sigCheck?.passed).toBe(false);
  });

  it("should fail verification for credential missing required fields", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(
      makeCredential({ id: "", credentialType: [] })
    );

    expect(result.valid).toBe(false);
    const structureCheck = result.checks.find((c) => c.name === "structure");
    expect(structureCheck?.passed).toBe(false);
  });

  it("should manage trusted issuers", () => {
    const verifier = new CredentialVerifier();

    expect(verifier.isTrustedIssuer("did:veritas:key:issuer123")).toBe(false);

    verifier.addTrustedIssuer("did:veritas:key:issuer123");
    expect(verifier.isTrustedIssuer("did:veritas:key:issuer123")).toBe(true);

    verifier.removeTrustedIssuer("did:veritas:key:issuer123");
    expect(verifier.isTrustedIssuer("did:veritas:key:issuer123")).toBe(false);
  });

  it("should throw VerificationError for null credential", () => {
    const verifier = new CredentialVerifier();

    expect(() => verifier.verify(null as any)).toThrow(VerificationError);
  });

  it("should pass expiration check when no expiresAt is set", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(makeCredential({ expiresAt: undefined }));

    const expirationCheck = result.checks.find(
      (c) => c.name === "expiration"
    );
    expect(expirationCheck?.passed).toBe(true);
  });

  it("should accept Active state credentials", () => {
    const verifier = new CredentialVerifier();
    verifier.addTrustedIssuer("did:veritas:key:issuer123");

    const result = verifier.verify(
      makeCredential({ state: CredentialState.Active })
    );

    const stateCheck = result.checks.find((c) => c.name === "state");
    expect(stateCheck?.passed).toBe(true);
  });
});
