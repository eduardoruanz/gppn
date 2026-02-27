/**
 * Core types for the GPPN SDK.
 */

/** States a payment can be in during its lifecycle. */
export enum PaymentState {
  /** Payment has been created but not yet submitted. */
  Created = "created",
  /** Payment is pending processing. */
  Pending = "pending",
  /** Payment is being routed through the network. */
  Routing = "routing",
  /** Payment has been settled successfully. */
  Settled = "settled",
  /** Payment has failed. */
  Failed = "failed",
  /** Payment has been cancelled. */
  Cancelled = "cancelled",
}

/** Represents a currency in the GPPN network. */
export interface Currency {
  /** ISO 4217 code or token symbol (e.g. "USD", "BTC"). */
  code: string;
  /** Number of decimal places for the currency. */
  decimals: number;
}

/** Represents a monetary amount with its currency. */
export interface Amount {
  /** The numeric value as a string to avoid floating-point issues. */
  value: string;
  /** The currency of this amount. */
  currency: Currency;
}

/** A payment message exchanged between nodes. */
export interface PaymentMessage {
  /** Unique identifier for the payment. */
  id: string;
  /** Public key of the sender. */
  sender: string;
  /** Public key of the recipient. */
  recipient: string;
  /** The amount being transferred. */
  amount: Amount;
  /** Current state of the payment. */
  state: PaymentState;
  /** Optional memo or description. */
  memo?: string;
  /** Timestamp of payment creation (ISO 8601). */
  createdAt: string;
  /** Timestamp of the last update (ISO 8601). */
  updatedAt: string;
}

/** A single hop in a payment route. */
export interface RouteEntry {
  /** Public key of the node at this hop. */
  nodeId: string;
  /** Fee charged by this node for forwarding. */
  fee: Amount;
  /** Estimated latency in milliseconds for this hop. */
  latencyMs: number;
}

/** A complete route through the network for a payment. */
export interface Route {
  /** Ordered list of hops from sender to recipient. */
  entries: RouteEntry[];
  /** Total fee for the entire route. */
  totalFee: Amount;
  /** Estimated total latency in milliseconds. */
  totalLatencyMs: number;
  /** A score indicating route quality (higher is better). */
  score: number;
}

/** Trust score for a peer in the network. */
export interface TrustScore {
  /** Public key of the peer. */
  peerId: string;
  /** Numeric trust score between 0.0 and 1.0. */
  score: number;
  /** Number of successful interactions. */
  successCount: number;
  /** Number of failed interactions. */
  failureCount: number;
  /** Timestamp of last update (ISO 8601). */
  lastUpdated: string;
}

/** Information about a peer node in the network. */
export interface PeerInfo {
  /** Public key of the peer. */
  peerId: string;
  /** Network address of the peer. */
  address: string;
  /** Whether the peer is currently connected. */
  connected: boolean;
  /** Trust score for this peer. */
  trustScore: TrustScore;
  /** Timestamp of last seen activity (ISO 8601). */
  lastSeen: string;
}

/** Status of the local GPPN node. */
export interface NodeStatus {
  /** Public key of this node. */
  nodeId: string;
  /** Whether the node is currently connected to the network. */
  connected: boolean;
  /** Number of connected peers. */
  peerCount: number;
  /** Node version string. */
  version: string;
  /** Node uptime in seconds. */
  uptimeSeconds: number;
}

/** Status of a settlement operation. */
export interface SettlementStatus {
  /** The payment ID being settled. */
  paymentId: string;
  /** Current state of the settlement. */
  state: PaymentState;
  /** Number of confirmations received. */
  confirmations: number;
  /** Number of confirmations required. */
  requiredConfirmations: number;
  /** Timestamp of settlement initiation (ISO 8601). */
  initiatedAt: string;
  /** Timestamp of settlement completion, if completed (ISO 8601). */
  completedAt?: string;
}
