/**
 * Route finding and selection for GPPN payments.
 */

import { RoutingError } from "./errors.js";
import type { Amount, Route } from "./types.js";

/** Finds and selects optimal payment routes through the GPPN network. */
export class RouteFinder {
  private readonly _baseUrl: string;

  constructor(baseUrl: string) {
    this._baseUrl = baseUrl;
  }

  /**
   * Find available routes from sender to recipient.
   *
   * This is a placeholder implementation that returns mock routes.
   * In a real implementation this would query the GPPN network.
   *
   * @param sender - Hex-encoded public key of the sender.
   * @param recipient - Hex-encoded public key of the recipient.
   * @param amount - The amount to route.
   * @returns An array of possible routes, sorted by score descending.
   */
  async findRoutes(
    sender: string,
    recipient: string,
    amount: Amount
  ): Promise<Route[]> {
    if (!sender || !recipient) {
      throw new RoutingError("Both sender and recipient are required");
    }
    if (!amount || !amount.value || Number(amount.value) <= 0) {
      throw new RoutingError("A valid positive amount is required");
    }

    // Placeholder: return mock routes
    const routes: Route[] = [
      {
        entries: [
          {
            nodeId: sender,
            fee: { value: "0", currency: amount.currency },
            latencyMs: 0,
          },
          {
            nodeId: "relay_node_1",
            fee: { value: "0.01", currency: amount.currency },
            latencyMs: 50,
          },
          {
            nodeId: recipient,
            fee: { value: "0", currency: amount.currency },
            latencyMs: 10,
          },
        ],
        totalFee: { value: "0.01", currency: amount.currency },
        totalLatencyMs: 60,
        score: 0.95,
      },
      {
        entries: [
          {
            nodeId: sender,
            fee: { value: "0", currency: amount.currency },
            latencyMs: 0,
          },
          {
            nodeId: "relay_node_2",
            fee: { value: "0.005", currency: amount.currency },
            latencyMs: 120,
          },
          {
            nodeId: "relay_node_3",
            fee: { value: "0.005", currency: amount.currency },
            latencyMs: 80,
          },
          {
            nodeId: recipient,
            fee: { value: "0", currency: amount.currency },
            latencyMs: 10,
          },
        ],
        totalFee: { value: "0.01", currency: amount.currency },
        totalLatencyMs: 210,
        score: 0.8,
      },
    ];

    return routes.sort((a, b) => b.score - a.score);
  }

  /**
   * Select the best route from a list of candidates.
   * Uses the route score for selection (higher is better).
   *
   * @param routes - Array of candidate routes.
   * @returns The highest-scoring route.
   * @throws RoutingError if no routes are available.
   */
  selectBestRoute(routes: Route[]): Route {
    if (!routes || routes.length === 0) {
      throw new RoutingError("No routes available");
    }

    return routes.reduce((best, current) =>
      current.score > best.score ? current : best
    );
  }
}
