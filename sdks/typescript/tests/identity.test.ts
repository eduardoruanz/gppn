import { describe, it, expect } from "vitest";
import { GppnIdentity } from "../src/identity.js";

describe("GppnIdentity", () => {
  it("should create a new identity with valid key pair", async () => {
    const identity = await GppnIdentity.createIdentity();

    expect(identity.privateKey).toBeInstanceOf(Uint8Array);
    expect(identity.publicKey).toBeInstanceOf(Uint8Array);
    expect(identity.privateKey.length).toBe(32);
    expect(identity.publicKey.length).toBe(32);
    expect(identity.publicKeyHex).toHaveLength(64);
  });

  it("should create unique identities each time", async () => {
    const id1 = await GppnIdentity.createIdentity();
    const id2 = await GppnIdentity.createIdentity();

    expect(id1.publicKeyHex).not.toBe(id2.publicKeyHex);
  });

  it("should sign and verify a message successfully", async () => {
    const identity = await GppnIdentity.createIdentity();
    const message = new TextEncoder().encode("Hello, GPPN!");

    const signature = await identity.sign(message);

    expect(signature).toBeInstanceOf(Uint8Array);
    expect(signature.length).toBe(64);

    const valid = await GppnIdentity.verify(signature, message, identity.publicKey);
    expect(valid).toBe(true);
  });

  it("should fail verification with a wrong public key", async () => {
    const identity = await GppnIdentity.createIdentity();
    const otherIdentity = await GppnIdentity.createIdentity();
    const message = new TextEncoder().encode("Hello, GPPN!");

    const signature = await identity.sign(message);

    const valid = await GppnIdentity.verify(
      signature,
      message,
      otherIdentity.publicKey
    );
    expect(valid).toBe(false);
  });

  it("should fail verification with a tampered message", async () => {
    const identity = await GppnIdentity.createIdentity();
    const message = new TextEncoder().encode("Hello, GPPN!");
    const tamperedMessage = new TextEncoder().encode("Tampered message!");

    const signature = await identity.sign(message);

    const valid = await GppnIdentity.verify(
      signature,
      tamperedMessage,
      identity.publicKey
    );
    expect(valid).toBe(false);
  });
});
