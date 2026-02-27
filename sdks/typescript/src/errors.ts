/**
 * Error classes for the GPPN SDK.
 */

/** Base error class for all GPPN errors. */
export class GppnError extends Error {
  /** A machine-readable error code. */
  public readonly code: string;

  constructor(message: string, code: string = "GPPN_ERROR") {
    super(message);
    this.name = "GppnError";
    this.code = code;
    // Restore prototype chain for proper instanceof checks
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Error thrown when a network connection fails. */
export class ConnectionError extends GppnError {
  constructor(message: string) {
    super(message, "CONNECTION_ERROR");
    this.name = "ConnectionError";
  }
}

/** Error thrown when a payment operation fails. */
export class PaymentError extends GppnError {
  /** The payment ID associated with the error, if available. */
  public readonly paymentId?: string;

  constructor(message: string, paymentId?: string) {
    super(message, "PAYMENT_ERROR");
    this.name = "PaymentError";
    this.paymentId = paymentId;
  }
}

/** Error thrown when route finding or selection fails. */
export class RoutingError extends GppnError {
  constructor(message: string) {
    super(message, "ROUTING_ERROR");
    this.name = "RoutingError";
  }
}

/** Error thrown when identity operations fail. */
export class IdentityError extends GppnError {
  constructor(message: string) {
    super(message, "IDENTITY_ERROR");
    this.name = "IdentityError";
  }
}
