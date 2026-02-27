/**
 * GPPN TypeScript SDK
 *
 * A complete SDK for interacting with the Global Peer-to-Peer Payment Network.
 */

// Core types
export {
  PaymentState,
  type Currency,
  type Amount,
  type PaymentMessage,
  type RouteEntry,
  type Route,
  type TrustScore,
  type PeerInfo,
  type NodeStatus,
  type SettlementStatus,
} from "./types.js";

// Error classes
export {
  GppnError,
  ConnectionError,
  PaymentError,
  RoutingError,
  IdentityError,
} from "./errors.js";

// Identity management
export { GppnIdentity } from "./identity.js";

// Payment building
export { PaymentBuilder } from "./payments.js";

// Route finding
export { RouteFinder } from "./routing.js";

// Settlement tracking
export { SettlementTracker } from "./settlement.js";

// Trust management
export { TrustManager } from "./trust.js";

// Main client
export { GppnClient, type GppnClientOptions } from "./client.js";
