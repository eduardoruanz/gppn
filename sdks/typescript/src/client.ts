/**
 * Main GPPN client -- the primary entry point for the SDK.
 */

import { ConnectionError, PaymentError } from "./errors.js";
import { GppnIdentity } from "./identity.js";
import { PaymentBuilder } from "./payments.js";
import { RouteFinder } from "./routing.js";
import { SettlementTracker } from "./settlement.js";
import { TrustManager } from "./trust.js";
import type {
  Amount,
  Currency,
  NodeStatus,
  PaymentMessage,
  PeerInfo,
  Route,
  TrustScore,
} from "./types.js";
import { PaymentState } from "./types.js";

/** Options for creating a GppnClient. */
export interface GppnClientOptions {
  /** The URL of the GPPN node to connect to. */
  url: string;
  /** An optional pre-existing identity (key pair) to use. */
  keypair?: GppnIdentity;
}

/**
 * The main GPPN client.
 *
 * Provides high-level methods for interacting with the GPPN network
 * including payments, routing, trust, and identity management.
 */
export class GppnClient {
  private readonly _url: string;
  private _identity: GppnIdentity | undefined;
  private _connected: boolean = false;
  private readonly _routeFinder: RouteFinder;
  private readonly _settlementTracker: SettlementTracker;
  private readonly _trustManager: TrustManager;
  private readonly _payments: Map<string, PaymentMessage> = new Map();

  constructor(options: GppnClientOptions) {
    this._url = options.url;
    this._identity = options.keypair;
    this._routeFinder = new RouteFinder(options.url);
    this._settlementTracker = new SettlementTracker();
    this._trustManager = new TrustManager();
  }

  /** Whether the client is currently connected to the network. */
  get connected(): boolean {
    return this._connected;
  }

  /** The client's identity, if one has been set or created. */
  get identity(): GppnIdentity | undefined {
    return this._identity;
  }

  /**
   * Connect to the GPPN network.
   *
   * This is a placeholder that simulates establishing a connection.
   *
   * @throws ConnectionError if already connected.
   */
  async connect(): Promise<void> {
    if (this._connected) {
      throw new ConnectionError("Already connected");
    }

    // Placeholder: simulate connection handshake
    this._connected = true;
  }

  /**
   * Disconnect from the GPPN network.
   *
   * @throws ConnectionError if not connected.
   */
  async disconnect(): Promise<void> {
    if (!this._connected) {
      throw new ConnectionError("Not connected");
    }

    this._connected = false;
  }

  /**
   * Create a new identity for this client.
   *
   * @returns The newly created GppnIdentity.
   */
  async createIdentity(): Promise<GppnIdentity> {
    this._identity = await GppnIdentity.createIdentity();
    return this._identity;
  }

  /**
   * Send a payment to a recipient.
   *
   * @param recipient - Hex-encoded public key of the recipient.
   * @param value - The amount value as a string.
   * @param currency - The currency to use.
   * @param memo - Optional payment memo.
   * @returns The created PaymentMessage.
   * @throws ConnectionError if not connected.
   * @throws PaymentError if no identity is set.
   */
  async sendPayment(
    recipient: string,
    value: string,
    currency: Currency,
    memo?: string
  ): Promise<PaymentMessage> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to send payments");
    }
    if (!this._identity) {
      throw new PaymentError("No identity set; call createIdentity() first");
    }

    const builder = new PaymentBuilder()
      .sender(this._identity.publicKeyHex)
      .recipient(recipient)
      .amount(value, currency);

    if (memo) {
      builder.memo(memo);
    }

    const payment = builder.build();
    this._payments.set(payment.id, payment);

    // Start tracking settlement
    this._settlementTracker.track(payment.id);

    return payment;
  }

  /**
   * Get the status of a previously sent payment.
   *
   * @param paymentId - The ID of the payment to check.
   * @returns The PaymentMessage, or undefined if not found.
   */
  async getPaymentStatus(paymentId: string): Promise<PaymentMessage | undefined> {
    return this._payments.get(paymentId);
  }

  /**
   * Find available routes to a recipient.
   *
   * @param recipient - Hex-encoded public key of the recipient.
   * @param amount - The amount to route.
   * @returns An array of possible routes.
   * @throws ConnectionError if not connected.
   * @throws PaymentError if no identity is set.
   */
  async findRoutes(recipient: string, amount: Amount): Promise<Route[]> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to find routes");
    }
    if (!this._identity) {
      throw new PaymentError("No identity set; call createIdentity() first");
    }

    return this._routeFinder.findRoutes(
      this._identity.publicKeyHex,
      recipient,
      amount
    );
  }

  /**
   * Get the trust score for a peer.
   *
   * @param peerId - The public key of the peer.
   * @returns The trust score for the peer.
   */
  async getTrustScore(peerId: string): Promise<TrustScore> {
    return this._trustManager.getTrustScore(peerId);
  }

  /**
   * Get a list of known peers.
   *
   * This is a placeholder that returns an empty list.
   *
   * @returns An array of PeerInfo objects.
   * @throws ConnectionError if not connected.
   */
  async getPeers(): Promise<PeerInfo[]> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to get peers");
    }

    // Placeholder: return empty list
    return [];
  }

  /**
   * Get the status of the connected node.
   *
   * This is a placeholder implementation.
   *
   * @returns The node status.
   * @throws ConnectionError if not connected.
   */
  async getNodeStatus(): Promise<NodeStatus> {
    if (!this._connected) {
      throw new ConnectionError("Must be connected to get node status");
    }

    return {
      nodeId: this._identity?.publicKeyHex ?? "unknown",
      connected: this._connected,
      peerCount: 0,
      version: "0.1.0",
      uptimeSeconds: 0,
    };
  }
}
