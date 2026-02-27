/**
 * Payment building and validation.
 */

import { PaymentError } from "./errors.js";
import type { Amount, Currency, PaymentMessage } from "./types.js";
import { PaymentState } from "./types.js";

/** Builder class for constructing GPPN payments with a fluent API. */
export class PaymentBuilder {
  private _sender?: string;
  private _recipient?: string;
  private _amount?: Amount;
  private _memo?: string;

  /**
   * Set the sender public key.
   * @param sender - Hex-encoded public key of the sender.
   */
  sender(sender: string): this {
    this._sender = sender;
    return this;
  }

  /**
   * Set the recipient public key.
   * @param recipient - Hex-encoded public key of the recipient.
   */
  recipient(recipient: string): this {
    this._recipient = recipient;
    return this;
  }

  /**
   * Set the payment amount.
   * @param value - The numeric value as a string.
   * @param currency - The currency for this payment.
   */
  amount(value: string, currency: Currency): this {
    this._amount = { value, currency };
    return this;
  }

  /**
   * Set an optional memo / description.
   * @param memo - A human-readable note attached to the payment.
   */
  memo(memo: string): this {
    this._memo = memo;
    return this;
  }

  /**
   * Validate the current builder state.
   * @returns An array of validation error messages. Empty if valid.
   */
  validate(): string[] {
    const errors: string[] = [];

    if (!this._sender) {
      errors.push("sender is required");
    }
    if (!this._recipient) {
      errors.push("recipient is required");
    }
    if (!this._amount) {
      errors.push("amount is required");
    } else {
      const numValue = Number(this._amount.value);
      if (isNaN(numValue) || numValue <= 0) {
        errors.push("amount must be a positive number");
      }
      if (!this._amount.currency.code) {
        errors.push("currency code is required");
      }
    }
    if (this._sender && this._recipient && this._sender === this._recipient) {
      errors.push("sender and recipient must be different");
    }

    return errors;
  }

  /**
   * Build the payment message.
   * @returns A fully constructed PaymentMessage.
   * @throws PaymentError if validation fails.
   */
  build(): PaymentMessage {
    const errors = this.validate();
    if (errors.length > 0) {
      throw new PaymentError(`Invalid payment: ${errors.join(", ")}`);
    }

    const now = new Date().toISOString();

    return {
      id: generatePaymentId(),
      sender: this._sender!,
      recipient: this._recipient!,
      amount: this._amount!,
      state: PaymentState.Created,
      memo: this._memo,
      createdAt: now,
      updatedAt: now,
    };
  }
}

/**
 * Generate a unique payment ID.
 * Uses a combination of timestamp and random bytes.
 */
function generatePaymentId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 10);
  return `pay_${timestamp}_${random}`;
}
