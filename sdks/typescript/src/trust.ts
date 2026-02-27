/**
 * Trust score management for Veritas peers.
 */

import type { TrustScore } from "./types.js";

/** Manages trust scores for peers in the Veritas network. */
export class TrustManager {
  private readonly _scores: Map<string, TrustScore> = new Map();

  /**
   * Get the trust score for a peer.
   *
   * @param did - The DID of the peer.
   * @returns The trust score, or a default score if the peer is unknown.
   */
  getTrustScore(did: string): TrustScore {
    const existing = this._scores.get(did);
    if (existing) {
      return existing;
    }

    // Return a default neutral trust score for unknown peers
    const defaultScore: TrustScore = {
      did,
      score: 0.5,
      successCount: 0,
      failureCount: 0,
      lastUpdated: new Date().toISOString(),
    };

    this._scores.set(did, defaultScore);
    return defaultScore;
  }

  /**
   * Update the trust score for a peer based on a verification outcome.
   *
   * @param did - The DID of the peer.
   * @param success - Whether the verification was successful.
   * @returns The updated trust score.
   */
  updateScore(did: string, success: boolean): TrustScore {
    const current = this.getTrustScore(did);

    if (success) {
      current.successCount += 1;
    } else {
      current.failureCount += 1;
    }

    const total = current.successCount + current.failureCount;
    current.score = total > 0 ? current.successCount / total : 0.5;
    current.lastUpdated = new Date().toISOString();

    this._scores.set(did, current);
    return current;
  }
}
