//! GPPN Routing — Smart routing layer for the Global Payment Protocol Network.
//!
//! This crate provides:
//! - [`DistributedRoutingTable`] — a concurrent, lock-free routing table backed by DashMap.
//! - [`PathFinder`] — a modified Dijkstra's algorithm that discovers optimal payment routes.
//! - [`Route`] — an ordered sequence of hops with cost, latency, and trust aggregation.
//! - [`ScoringWeights`] and [`RouteScore`] — configurable multi-factor route scoring.
//! - [`RouteAdvertisement`] — creation and processing of route advertisements.

pub mod advertisement;
pub mod drt;
pub mod error;
pub mod pathfinder;
pub mod route;
pub mod scoring;

// Re-exports for convenience.
pub use advertisement::{
    create_readvertisement, process_advertisement, DestinationAnnouncement, RouteAdvertisement,
};
pub use drt::{DistributedRoutingTable, RouteEntry};
pub use error::RoutingError;
pub use pathfinder::{PathFinder, PathFinderConfig};
pub use route::Route;
pub use scoring::{RouteScore, ScoringInput, ScoringWeights};
