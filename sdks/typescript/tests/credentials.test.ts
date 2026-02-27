import { describe, it, expect } from "vitest";
import { CredentialBuilder } from "../src/credentials.js";
import { CredentialState } from "../src/types.js";
import { CredentialError } from "../src/errors.js";

describe("CredentialBuilder", () => {
  it("should build a valid verifiable credential", () => {
    const vc = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .subject("did:veritas:key:subject456")
      .type("KycBasic")
      .claim("full_name", "Alice Smith")
      .claim("date_of_birth", "1990-01-15")
      .claim("country", "US")
      .build();

    expect(vc.id).toMatch(/^vc_/);
    expect(vc.issuer).toBe("did:veritas:key:issuer123");
    expect(vc.subject).toBe("did:veritas:key:subject456");
    expect(vc.credentialType).toEqual(["KycBasic"]);
    expect(vc.claims.full_name).toBe("Alice Smith");
    expect(vc.claims.date_of_birth).toBe("1990-01-15");
    expect(vc.claims.country).toBe("US");
    expect(vc.state).toBe(CredentialState.Issued);
    expect(vc.issuedAt).toBeTruthy();
  });

  it("should build a credential with multiple types", () => {
    const vc = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .subject("did:veritas:key:subject456")
      .type("KycBasic")
      .type("AgeVerification")
      .claim("age", 25)
      .build();

    expect(vc.credentialType).toEqual(["KycBasic", "AgeVerification"]);
  });

  it("should build a credential with expiration", () => {
    const expiresAt = "2027-12-31T23:59:59Z";
    const vc = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .subject("did:veritas:key:subject456")
      .type("Residency")
      .claim("country", "BR")
      .expires(expiresAt)
      .build();

    expect(vc.expiresAt).toBe(expiresAt);
  });

  it("should set multiple claims at once", () => {
    const vc = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .subject("did:veritas:key:subject456")
      .type("KycEnhanced")
      .claims({
        full_name: "Bob Jones",
        kyc_level: 3,
        verified: true,
      })
      .build();

    expect(vc.claims.full_name).toBe("Bob Jones");
    expect(vc.claims.kyc_level).toBe(3);
    expect(vc.claims.verified).toBe(true);
  });

  it("should return validation errors for missing issuer", () => {
    const builder = new CredentialBuilder()
      .subject("did:veritas:key:subject456")
      .type("KycBasic")
      .claim("name", "Alice");

    const errors = builder.validate();
    expect(errors).toContain("issuer is required");
  });

  it("should return validation errors for missing subject", () => {
    const builder = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .type("KycBasic")
      .claim("name", "Alice");

    const errors = builder.validate();
    expect(errors).toContain("subject is required");
  });

  it("should return validation errors for missing credential type", () => {
    const builder = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .subject("did:veritas:key:subject456")
      .claim("name", "Alice");

    const errors = builder.validate();
    expect(errors).toContain("at least one credential type is required");
  });

  it("should return validation errors for missing claims", () => {
    const builder = new CredentialBuilder()
      .issuer("did:veritas:key:issuer123")
      .subject("did:veritas:key:subject456")
      .type("KycBasic");

    const errors = builder.validate();
    expect(errors).toContain("at least one claim is required");
  });

  it("should return validation error when issuer equals subject", () => {
    const builder = new CredentialBuilder()
      .issuer("did:veritas:key:same")
      .subject("did:veritas:key:same")
      .type("KycBasic")
      .claim("name", "Alice");

    const errors = builder.validate();
    expect(errors).toContain("issuer and subject must be different");
  });

  it("should throw CredentialError when building with missing fields", () => {
    const builder = new CredentialBuilder();

    expect(() => builder.build()).toThrow(CredentialError);
    expect(() => builder.build()).toThrow("Invalid credential");
  });

  it("should throw CredentialError with all missing field messages", () => {
    const builder = new CredentialBuilder();

    try {
      builder.build();
      expect.unreachable("Should have thrown");
    } catch (err) {
      expect(err).toBeInstanceOf(CredentialError);
      const msg = (err as CredentialError).message;
      expect(msg).toContain("issuer is required");
      expect(msg).toContain("subject is required");
      expect(msg).toContain("at least one credential type is required");
      expect(msg).toContain("at least one claim is required");
    }
  });
});
