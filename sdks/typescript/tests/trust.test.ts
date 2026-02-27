import { describe, it, expect } from "vitest";
import { TrustManager } from "../src/trust.js";

describe("TrustManager", () => {
  it("should return a default trust score for unknown peers", () => {
    const manager = new TrustManager();
    const score = manager.getTrustScore("did:veritas:key:unknown");

    expect(score.did).toBe("did:veritas:key:unknown");
    expect(score.score).toBe(0.5);
    expect(score.successCount).toBe(0);
    expect(score.failureCount).toBe(0);
  });

  it("should increase trust score on successful verification", () => {
    const manager = new TrustManager();
    const did = "did:veritas:key:peer1";

    manager.updateScore(did, true);
    manager.updateScore(did, true);
    manager.updateScore(did, true);

    const score = manager.getTrustScore(did);
    expect(score.score).toBe(1.0);
    expect(score.successCount).toBe(3);
    expect(score.failureCount).toBe(0);
  });

  it("should decrease trust score on failed verification", () => {
    const manager = new TrustManager();
    const did = "did:veritas:key:peer2";

    manager.updateScore(did, false);
    manager.updateScore(did, false);

    const score = manager.getTrustScore(did);
    expect(score.score).toBe(0);
    expect(score.successCount).toBe(0);
    expect(score.failureCount).toBe(2);
  });

  it("should compute a weighted score from mixed outcomes", () => {
    const manager = new TrustManager();
    const did = "did:veritas:key:peer3";

    manager.updateScore(did, true);
    manager.updateScore(did, true);
    manager.updateScore(did, false);

    const score = manager.getTrustScore(did);
    expect(score.score).toBeCloseTo(2 / 3, 5);
    expect(score.successCount).toBe(2);
    expect(score.failureCount).toBe(1);
  });

  it("should update the lastUpdated timestamp", () => {
    const manager = new TrustManager();
    const did = "did:veritas:key:peer4";

    const initial = manager.getTrustScore(did);
    const initialTime = initial.lastUpdated;

    // Small delay to ensure timestamp differs
    manager.updateScore(did, true);

    const updated = manager.getTrustScore(did);
    expect(updated.lastUpdated).toBeTruthy();
  });
});
