import { describe, it, expect } from "vitest";
import { VeritasClient } from "../src/client.js";
import { ConnectionError, CredentialError } from "../src/errors.js";
import { CredentialState } from "../src/types.js";

describe("VeritasClient", () => {
  it("should connect and disconnect", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });

    expect(client.connected).toBe(false);

    await client.connect();
    expect(client.connected).toBe(true);

    await client.disconnect();
    expect(client.connected).toBe(false);
  });

  it("should throw when connecting twice", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();

    await expect(client.connect()).rejects.toThrow(ConnectionError);
  });

  it("should throw when disconnecting while not connected", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });

    await expect(client.disconnect()).rejects.toThrow(ConnectionError);
  });

  it("should create an identity with a DID", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    const identity = await client.createIdentity();

    expect(identity.did).toMatch(/^did:veritas:key:/);
    expect(client.identity).toBe(identity);
  });

  it("should issue a credential", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();
    await client.createIdentity();

    const vc = await client.issueCredential(
      "did:veritas:key:subject456",
      ["KycBasic"],
      { full_name: "Alice Smith", country: "US" }
    );

    expect(vc.id).toMatch(/^vc_/);
    expect(vc.issuer).toBe(client.identity!.did);
    expect(vc.subject).toBe("did:veritas:key:subject456");
    expect(vc.state).toBe(CredentialState.Issued);
    expect(vc.claims.full_name).toBe("Alice Smith");
  });

  it("should require connection to issue credentials", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.createIdentity();

    await expect(
      client.issueCredential("did:veritas:key:sub", ["KycBasic"], {
        name: "Alice",
      })
    ).rejects.toThrow(ConnectionError);
  });

  it("should require identity to issue credentials", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();

    await expect(
      client.issueCredential("did:veritas:key:sub", ["KycBasic"], {
        name: "Alice",
      })
    ).rejects.toThrow(CredentialError);
  });

  it("should store and retrieve credentials", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();
    await client.createIdentity();

    const vc = await client.issueCredential(
      "did:veritas:key:subject456",
      ["KycBasic"],
      { name: "Alice" }
    );

    const retrieved = await client.getCredential(vc.id);
    expect(retrieved).toEqual(vc);

    const all = await client.listCredentials();
    expect(all).toHaveLength(1);
    expect(all[0].id).toBe(vc.id);
  });

  it("should request age proofs", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();
    await client.createIdentity();

    const req = await client.requestAgeProof(18);

    expect(req.id).toMatch(/^pr_/);
    expect(req.verifier).toBe(client.identity!.did);
    expect(req.params.min_age).toBe(18);
  });

  it("should request residency proofs", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();
    await client.createIdentity();

    const req = await client.requestResidencyProof(["US", "BR"]);

    expect(req.id).toMatch(/^pr_/);
    expect(req.params.allowed_countries).toEqual(["US", "BR"]);
  });

  it("should manage trust scores", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    const did = "did:veritas:key:peer1";

    const initial = await client.getTrustScore(did);
    expect(initial.score).toBe(0.5);

    await client.updateTrust(did, true);
    const updated = await client.getTrustScore(did);
    expect(updated.score).toBe(1.0);
  });

  it("should get node status", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });
    await client.connect();
    await client.createIdentity();

    const status = await client.getNodeStatus();
    expect(status.did).toMatch(/^did:veritas:key:/);
    expect(status.connected).toBe(true);
    expect(status.version).toBe("0.1.0");
  });

  it("should require connection for getPeers", async () => {
    const client = new VeritasClient({ url: "http://localhost:9001" });

    await expect(client.getPeers()).rejects.toThrow(ConnectionError);
  });
});
