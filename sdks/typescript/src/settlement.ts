/**
 * Settlement tracking for GPPN payments.
 */

import { PaymentError } from "./errors.js";
import type { SettlementStatus } from "./types.js";
import { PaymentState } from "./types.js";

/** Tracks settlement status of payments in the GPPN network. */
export class SettlementTracker {
  private readonly _settlements: Map<string, SettlementStatus> = new Map();

  /**
   * Begin tracking a payment settlement.
   *
   * @param paymentId - The ID of the payment to track.
   * @param requiredConfirmations - Number of confirmations needed (default: 3).
   * @returns The initial settlement status.
   */
  track(paymentId: string, requiredConfirmations: number = 3): SettlementStatus {
    if (!paymentId) {
      throw new PaymentError("Payment ID is required for settlement tracking");
    }

    const status: SettlementStatus = {
      paymentId,
      state: PaymentState.Pending,
      confirmations: 0,
      requiredConfirmations,
      initiatedAt: new Date().toISOString(),
    };

    this._settlements.set(paymentId, status);
    return status;
  }

  /**
   * Get the current settlement status for a payment.
   *
   * @param paymentId - The ID of the payment.
   * @returns The settlement status, or undefined if not tracked.
   */
  getStatus(paymentId: string): SettlementStatus | undefined {
    return this._settlements.get(paymentId);
  }

  /**
   * Wait for a payment settlement to reach the required number of confirmations.
   *
   * This is a placeholder implementation that simulates confirmation progress.
   *
   * @param paymentId - The ID of the payment to wait for.
   * @param timeoutMs - Maximum time to wait in milliseconds (default: 30000).
   * @returns The final settlement status.
   * @throws PaymentError if the payment is not being tracked or times out.
   */
  async waitForConfirmation(
    paymentId: string,
    timeoutMs: number = 30000
  ): Promise<SettlementStatus> {
    const status = this._settlements.get(paymentId);
    if (!status) {
      throw new PaymentError(
        `Payment ${paymentId} is not being tracked`,
        paymentId
      );
    }

    // Placeholder: simulate confirmations arriving
    const intervalMs = 100;
    const maxIterations = Math.ceil(timeoutMs / intervalMs);

    for (let i = 0; i < maxIterations; i++) {
      if (status.confirmations >= status.requiredConfirmations) {
        status.state = PaymentState.Settled;
        status.completedAt = new Date().toISOString();
        return status;
      }

      // Simulate a confirmation arriving
      status.confirmations += 1;
      status.state = PaymentState.Pending;

      await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }

    throw new PaymentError(
      `Settlement timed out for payment ${paymentId}`,
      paymentId
    );
  }
}
