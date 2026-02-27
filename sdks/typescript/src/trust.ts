/**
 * Trust score management for GPPN peers.
 */

import type { TrustScore } from "./types.js";

/** Manages trust scores for peers in the GPPN network. */
export class TrustManager {
  private readonly _scores: Map<string, TrustScore> = new Map();

  /**
   * Get the trust score for a peer.
   *
   * @param peerId - The public key of the peer.
   * @returns The trust score, or a default score if the peer is unknown.
   */
  getTrustScore(peerId: string): TrustScore {
    const existing = this._scores.get(peerId);
    if (existing) {
      return existing;
    }

    // Return a default neutral trust score for unknown peers
    const defaultScore: TrustScore = {
      peerId,
      score: 0.5,
      successCount: 0,
      failureCount: 0,
      lastUpdated: new Date().toISOString(),
    };

    this._scores.set(peerId, defaultScore);
    return defaultScore;
  }

  /**
   * Update the trust score for a peer based on an interaction outcome.
   *
   * @param peerId - The public key of the peer.
   * @param success - Whether the interaction was successful.
   * @returns The updated trust score.
   */
  updateScore(peerId: string, success: boolean): TrustScore {
    const current = this.getTrustScore(peerId);

    if (success) {
      current.successCount += 1;
    } else {
      current.failureCount += 1;
    }

    const total = current.successCount + current.failureCount;
    // Weighted moving average: new score = successes / total
    // Applies a smoothing factor to avoid extreme swings
    current.score = total > 0 ? current.successCount / total : 0.5;
    current.lastUpdated = new Date().toISOString();

    this._scores.set(peerId, current);
    return current;
  }
}
