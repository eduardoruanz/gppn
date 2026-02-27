import { describe, it, expect } from "vitest";
import { VeritasIdentity } from "../src/identity.js";

describe("VeritasIdentity", () => {
  it("should create a new identity with DID", async () => {
    const identity = await VeritasIdentity.createIdentity();

    expect(identity.privateKey).toHaveLength(32);
    expect(identity.publicKey).toHaveLength(32);
    expect(identity.did).toMatch(/^did:veritas:key:/);
    expect(identity.publicKeyHex).toHaveLength(64);
    expect(identity.did).toContain(identity.publicKeyHex);
  });

  it("should sign and verify a message", async () => {
    const identity = await VeritasIdentity.createIdentity();
    const message = new TextEncoder().encode("Hello Veritas");

    const signature = await identity.sign(message);

    expect(signature).toHaveLength(64);

    const valid = await VeritasIdentity.verify(
      signature,
      message,
      identity.publicKey
    );
    expect(valid).toBe(true);
  });

  it("should fail verification with wrong public key", async () => {
    const identity1 = await VeritasIdentity.createIdentity();
    const identity2 = await VeritasIdentity.createIdentity();
    const message = new TextEncoder().encode("Hello Veritas");

    const signature = await identity1.sign(message);

    const valid = await VeritasIdentity.verify(
      signature,
      message,
      identity2.publicKey
    );
    expect(valid).toBe(false);
  });

  it("should fail verification with tampered message", async () => {
    const identity = await VeritasIdentity.createIdentity();
    const message = new TextEncoder().encode("Hello Veritas");
    const tampered = new TextEncoder().encode("Hello Tampered");

    const signature = await identity.sign(message);

    const valid = await VeritasIdentity.verify(
      signature,
      tampered,
      identity.publicKey
    );
    expect(valid).toBe(false);
  });

  it("should generate unique identities", async () => {
    const id1 = await VeritasIdentity.createIdentity();
    const id2 = await VeritasIdentity.createIdentity();

    expect(id1.did).not.toBe(id2.did);
    expect(id1.publicKeyHex).not.toBe(id2.publicKeyHex);
  });
});
