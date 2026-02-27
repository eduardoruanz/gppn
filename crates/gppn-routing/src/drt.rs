use std::time::Duration;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use gppn_core::{Currency, Did};
use serde::{Deserialize, Serialize};

use crate::error::RoutingError;

/// Serde helper to serialize/deserialize `std::time::Duration` as seconds (u64).
mod duration_secs {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// A single entry in the Distributed Routing Table representing a reachable
/// destination through a specific next-hop peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    /// The peer ID of the next hop toward the destination.
    pub next_hop_peer_id: String,
    /// The destination DID this entry can reach.
    pub destination_did: Did,
    /// Currencies supported along this path.
    pub supported_currencies: Vec<Currency>,
    /// Available liquidity in atomic units.
    pub available_liquidity: u128,
    /// Fee rate as a fraction (e.g., 0.001 = 0.1%).
    pub fee_rate: f64,
    /// Average observed latency in milliseconds.
    pub avg_latency_ms: u64,
    /// Trust score in [0.0, 1.0] derived from historical behaviour.
    pub trust_score: f64,
    /// Timestamp of last successful update.
    pub last_updated: DateTime<Utc>,
    /// Time-to-live for this entry.
    #[serde(with = "duration_secs")]
    pub ttl: Duration,
    /// Number of hops to the destination through this path.
    pub hop_count: u32,
}

impl RouteEntry {
    /// Returns true if this entry has expired relative to `now`.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        let elapsed = now.signed_duration_since(self.last_updated);
        elapsed.to_std().map_or(false, |e| e >= self.ttl)
    }

    /// Returns true if this entry supports the given currency.
    pub fn supports_currency(&self, currency: &Currency) -> bool {
        self.supported_currencies.contains(currency)
    }

    /// Compute the fee for a given amount using this entry's fee_rate.
    pub fn fee_for(&self, amount: u128) -> u128 {
        ((amount as f64) * self.fee_rate).ceil() as u128
    }

    /// Validate that all fields are within acceptable ranges.
    pub fn validate(&self) -> Result<(), RoutingError> {
        if self.next_hop_peer_id.is_empty() {
            return Err(RoutingError::InvalidRouteEntry {
                reason: "next_hop_peer_id is empty".into(),
            });
        }
        if self.fee_rate < 0.0 || self.fee_rate > 1.0 {
            return Err(RoutingError::InvalidRouteEntry {
                reason: format!("fee_rate out of range [0, 1]: {}", self.fee_rate),
            });
        }
        if self.trust_score < 0.0 || self.trust_score > 1.0 {
            return Err(RoutingError::InvalidRouteEntry {
                reason: format!("trust_score out of range [0, 1]: {}", self.trust_score),
            });
        }
        if self.supported_currencies.is_empty() {
            return Err(RoutingError::InvalidRouteEntry {
                reason: "supported_currencies is empty".into(),
            });
        }
        Ok(())
    }
}

/// Composite key for looking up route entries: (destination DID uri, next-hop peer ID).
type RouteKey = (String, String);

/// A concurrent, lock-free Distributed Routing Table backed by DashMap.
///
/// Each key is `(destination_did_uri, next_hop_peer_id)` ensuring we can store
/// multiple paths to the same destination through different next-hops.
pub struct DistributedRoutingTable {
    table: DashMap<RouteKey, RouteEntry>,
}

impl DistributedRoutingTable {
    /// Create a new, empty routing table.
    pub fn new() -> Self {
        Self {
            table: DashMap::new(),
        }
    }

    /// Insert or overwrite a route entry. Returns any previous entry for the
    /// same (destination, next_hop) pair.
    pub fn insert(&self, entry: RouteEntry) -> Option<RouteEntry> {
        let key = (
            entry.destination_did.uri().to_string(),
            entry.next_hop_peer_id.clone(),
        );
        self.table.insert(key, entry)
    }

    /// Retrieve all route entries for a given destination DID.
    pub fn get_routes(&self, destination: &Did) -> Vec<RouteEntry> {
        let dest_uri = destination.uri().to_string();
        self.table
            .iter()
            .filter(|ref_multi| ref_multi.key().0 == dest_uri)
            .map(|ref_multi| ref_multi.value().clone())
            .collect()
    }

    /// Remove all entries whose TTL has expired relative to `now`.
    /// Returns the number of entries removed.
    pub fn remove_expired(&self, now: DateTime<Utc>) -> usize {
        let before = self.table.len();
        self.table.retain(|_key, entry| !entry.is_expired(now));
        before - self.table.len()
    }

    /// Update an existing entry. If the entry exists, the updater closure is
    /// called with a mutable reference to the entry. Returns `true` if the
    /// entry was found and updated.
    pub fn update<F>(&self, destination: &Did, next_hop: &str, updater: F) -> bool
    where
        F: FnOnce(&mut RouteEntry),
    {
        let key = (destination.uri().to_string(), next_hop.to_string());
        if let Some(mut entry) = self.table.get_mut(&key) {
            updater(entry.value_mut());
            true
        } else {
            false
        }
    }

    /// Remove a specific route entry. Returns the removed entry if it existed.
    pub fn remove(&self, destination: &Did, next_hop: &str) -> Option<RouteEntry> {
        let key = (destination.uri().to_string(), next_hop.to_string());
        self.table.remove(&key).map(|(_k, v)| v)
    }

    /// Total number of entries in the routing table.
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Returns `true` if the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Return all unique destination DIDs present in the table.
    pub fn destinations(&self) -> Vec<String> {
        let mut dests: Vec<String> = self
            .table
            .iter()
            .map(|r| r.key().0.clone())
            .collect();
        dests.sort();
        dests.dedup();
        dests
    }

    /// Return all entries in the routing table.
    pub fn all_entries(&self) -> Vec<RouteEntry> {
        self.table.iter().map(|r| r.value().clone()).collect()
    }
}

impl Default for DistributedRoutingTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    fn make_entry(dest: &str, next_hop: &str, liquidity: u128, fee_rate: f64) -> RouteEntry {
        RouteEntry {
            next_hop_peer_id: next_hop.to_string(),
            destination_did: Did::from_parts("key", dest),
            supported_currencies: vec![Currency::Fiat(gppn_core::types::FiatCurrency::USD)],
            available_liquidity: liquidity,
            fee_rate,
            avg_latency_ms: 50,
            trust_score: 0.9,
            last_updated: Utc::now(),
            ttl: Duration::from_secs(300),
            hop_count: 1,
        }
    }

    #[test]
    fn test_insert_and_get_routes() {
        let drt = DistributedRoutingTable::new();
        let entry = make_entry("alice", "peer-1", 1_000_000, 0.001);
        let dest = entry.destination_did.clone();

        assert!(drt.insert(entry).is_none());
        assert_eq!(drt.size(), 1);

        let routes = drt.get_routes(&dest);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].next_hop_peer_id, "peer-1");
    }

    #[test]
    fn test_multiple_routes_to_same_destination() {
        let drt = DistributedRoutingTable::new();
        let dest = Did::from_parts("key", "alice");

        drt.insert(make_entry("alice", "peer-1", 1_000_000, 0.001));
        drt.insert(make_entry("alice", "peer-2", 2_000_000, 0.002));
        drt.insert(make_entry("alice", "peer-3", 500_000, 0.0005));

        assert_eq!(drt.size(), 3);
        let routes = drt.get_routes(&dest);
        assert_eq!(routes.len(), 3);
    }

    #[test]
    fn test_update_entry() {
        let drt = DistributedRoutingTable::new();
        let entry = make_entry("alice", "peer-1", 1_000_000, 0.001);
        let dest = entry.destination_did.clone();
        drt.insert(entry);

        let updated = drt.update(&dest, "peer-1", |e| {
            e.available_liquidity = 5_000_000;
            e.trust_score = 0.95;
        });
        assert!(updated);

        let routes = drt.get_routes(&dest);
        assert_eq!(routes[0].available_liquidity, 5_000_000);
        assert!((routes[0].trust_score - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remove_expired() {
        let drt = DistributedRoutingTable::new();

        // Insert an entry with a very short TTL that is already expired.
        let mut expired_entry = make_entry("alice", "peer-1", 1_000_000, 0.001);
        expired_entry.ttl = Duration::from_secs(0);
        expired_entry.last_updated = Utc::now() - chrono::Duration::seconds(10);
        drt.insert(expired_entry);

        // Insert a fresh entry.
        drt.insert(make_entry("bob", "peer-2", 2_000_000, 0.002));

        assert_eq!(drt.size(), 2);
        let removed = drt.remove_expired(Utc::now());
        assert_eq!(removed, 1);
        assert_eq!(drt.size(), 1);
    }

    #[test]
    fn test_remove_specific_entry() {
        let drt = DistributedRoutingTable::new();
        let entry = make_entry("alice", "peer-1", 1_000_000, 0.001);
        let dest = entry.destination_did.clone();
        drt.insert(entry);

        let removed = drt.remove(&dest, "peer-1");
        assert!(removed.is_some());
        assert_eq!(drt.size(), 0);
    }

    #[test]
    fn test_concurrent_access() {
        let drt = Arc::new(DistributedRoutingTable::new());
        let mut handles = Vec::new();

        // Spawn 10 threads, each inserting 100 entries.
        for thread_id in 0..10u32 {
            let drt = Arc::clone(&drt);
            let handle = std::thread::spawn(move || {
                for i in 0..100u32 {
                    let dest_id = format!("dest-{}-{}", thread_id, i);
                    let peer_id = format!("peer-{}", thread_id);
                    let entry = RouteEntry {
                        next_hop_peer_id: peer_id,
                        destination_did: Did::from_parts("key", &dest_id),
                        supported_currencies: vec![Currency::Fiat(gppn_core::types::FiatCurrency::USD)],
                        available_liquidity: 1_000_000,
                        fee_rate: 0.001,
                        avg_latency_ms: 50,
                        trust_score: 0.9,
                        last_updated: Utc::now(),
                        ttl: Duration::from_secs(300),
                        hop_count: 1,
                    };
                    drt.insert(entry);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // All 1000 entries should be present.
        assert_eq!(drt.size(), 1000);
    }

    #[test]
    fn test_entry_validation() {
        let mut entry = make_entry("alice", "peer-1", 1_000_000, 0.001);
        assert!(entry.validate().is_ok());

        entry.fee_rate = 1.5;
        assert!(entry.validate().is_err());

        entry.fee_rate = 0.001;
        entry.trust_score = -0.1;
        assert!(entry.validate().is_err());

        entry.trust_score = 0.9;
        entry.next_hop_peer_id = String::new();
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_entry_is_expired() {
        let mut entry = make_entry("alice", "peer-1", 1_000_000, 0.001);
        let now = Utc::now();

        // Not expired yet.
        assert!(!entry.is_expired(now));

        // Make it expired.
        entry.last_updated = now - chrono::Duration::seconds(600);
        entry.ttl = Duration::from_secs(300);
        assert!(entry.is_expired(now));
    }

    #[test]
    fn test_fee_calculation() {
        let entry = make_entry("alice", "peer-1", 1_000_000, 0.01);
        // 1% of 1_000_000 = 10_000
        assert_eq!(entry.fee_for(1_000_000), 10_000);

        // Small amounts: ceil behaviour
        let entry2 = make_entry("alice", "peer-1", 1_000_000, 0.001);
        // 0.1% of 999 = 0.999 → ceil = 1
        assert_eq!(entry2.fee_for(999), 1);
    }
}
