use std::time::Duration;

use chrono::{DateTime, Utc};
use gppn_core::{Currency, Did};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::drt::{DistributedRoutingTable, RouteEntry};
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

/// A route advertisement message broadcast by a node to announce its
/// reachability and capabilities to peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAdvertisement {
    /// Unique identifier for this advertisement.
    pub id: Uuid,
    /// The DID of the node publishing this advertisement.
    pub advertiser_did: Did,
    /// The peer ID of the advertising node.
    pub advertiser_peer_id: String,
    /// Destinations reachable through this advertiser.
    pub reachable_destinations: Vec<DestinationAnnouncement>,
    /// Timestamp when this advertisement was created.
    pub created_at: DateTime<Utc>,
    /// Sequence number for ordering advertisements from the same node.
    pub sequence_number: u64,
    /// Time-to-live: how long recipients should keep these entries.
    #[serde(with = "duration_secs")]
    pub ttl: Duration,
}

/// A single destination announced in a RouteAdvertisement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationAnnouncement {
    /// The destination DID.
    pub destination_did: Did,
    /// Currencies supported on the path to this destination.
    pub supported_currencies: Vec<Currency>,
    /// Available liquidity to forward payments toward this destination.
    pub available_liquidity: u128,
    /// Fee rate charged for forwarding.
    pub fee_rate: f64,
    /// Average latency to the destination in milliseconds.
    pub avg_latency_ms: u64,
    /// Trust score for the path.
    pub trust_score: f64,
    /// Hop count to the destination.
    pub hop_count: u32,
}

impl RouteAdvertisement {
    /// Create a new route advertisement.
    pub fn new(
        advertiser_did: Did,
        advertiser_peer_id: String,
        reachable_destinations: Vec<DestinationAnnouncement>,
        ttl: Duration,
        sequence_number: u64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            advertiser_did,
            advertiser_peer_id,
            reachable_destinations,
            created_at: Utc::now(),
            sequence_number,
            ttl,
        }
    }

    /// Create a self-advertisement: the advertiser announces that it itself
    /// is directly reachable.
    pub fn self_advertisement(
        did: Did,
        peer_id: String,
        supported_currencies: Vec<Currency>,
        available_liquidity: u128,
        fee_rate: f64,
        ttl: Duration,
        sequence_number: u64,
    ) -> Self {
        let announcement = DestinationAnnouncement {
            destination_did: did.clone(),
            supported_currencies,
            available_liquidity,
            fee_rate,
            avg_latency_ms: 0,
            trust_score: 1.0,
            hop_count: 0,
        };
        Self::new(did, peer_id, vec![announcement], ttl, sequence_number)
    }

    /// Number of destinations announced in this advertisement.
    pub fn destination_count(&self) -> usize {
        self.reachable_destinations.len()
    }

    /// Validate the advertisement contents.
    pub fn validate(&self) -> Result<(), RoutingError> {
        if self.advertiser_peer_id.is_empty() {
            return Err(RoutingError::InvalidRouteEntry {
                reason: "advertiser_peer_id is empty".into(),
            });
        }
        if self.reachable_destinations.is_empty() {
            return Err(RoutingError::InvalidRouteEntry {
                reason: "no reachable destinations in advertisement".into(),
            });
        }
        for dest in &self.reachable_destinations {
            if dest.fee_rate < 0.0 || dest.fee_rate > 1.0 {
                return Err(RoutingError::InvalidRouteEntry {
                    reason: format!(
                        "fee_rate out of range [0, 1] for destination {}: {}",
                        dest.destination_did, dest.fee_rate
                    ),
                });
            }
            if dest.trust_score < 0.0 || dest.trust_score > 1.0 {
                return Err(RoutingError::InvalidRouteEntry {
                    reason: format!(
                        "trust_score out of range [0, 1] for destination {}: {}",
                        dest.destination_did, dest.trust_score
                    ),
                });
            }
        }
        Ok(())
    }
}

/// Process an incoming RouteAdvertisement and merge its entries into the DRT.
///
/// For each announced destination, a RouteEntry is created with the
/// advertiser as the next hop. If the advertiser's hop_count + 1 exceeds
/// `max_hops`, that destination is skipped.
///
/// Returns the number of entries inserted or updated.
pub fn process_advertisement(
    drt: &DistributedRoutingTable,
    advert: &RouteAdvertisement,
    max_hops: u32,
) -> Result<usize, RoutingError> {
    advert.validate()?;

    let mut count = 0;
    for dest in &advert.reachable_destinations {
        let effective_hop_count = dest.hop_count + 1;
        if effective_hop_count > max_hops {
            tracing::debug!(
                destination = %dest.destination_did,
                hop_count = effective_hop_count,
                max_hops,
                "skipping destination: exceeds max hops"
            );
            continue;
        }

        let entry = RouteEntry {
            next_hop_peer_id: advert.advertiser_peer_id.clone(),
            destination_did: dest.destination_did.clone(),
            supported_currencies: dest.supported_currencies.clone(),
            available_liquidity: dest.available_liquidity,
            fee_rate: dest.fee_rate,
            avg_latency_ms: dest.avg_latency_ms,
            trust_score: dest.trust_score,
            last_updated: advert.created_at,
            ttl: advert.ttl,
            hop_count: effective_hop_count,
        };

        drt.insert(entry);
        count += 1;
    }

    Ok(count)
}

/// Build a RouteAdvertisement from the local DRT entries, re-advertising
/// destinations this node can reach. The hop count is incremented by 1.
pub fn create_readvertisement(
    local_did: &Did,
    local_peer_id: &str,
    drt: &DistributedRoutingTable,
    ttl: Duration,
    sequence_number: u64,
    max_hops: u32,
) -> RouteAdvertisement {
    let announcements: Vec<DestinationAnnouncement> = drt
        .all_entries()
        .into_iter()
        .filter(|e| e.hop_count + 1 <= max_hops)
        .map(|e| DestinationAnnouncement {
            destination_did: e.destination_did,
            supported_currencies: e.supported_currencies,
            available_liquidity: e.available_liquidity,
            fee_rate: e.fee_rate,
            avg_latency_ms: e.avg_latency_ms,
            trust_score: e.trust_score,
            hop_count: e.hop_count + 1,
        })
        .collect();

    RouteAdvertisement::new(
        local_did.clone(),
        local_peer_id.to_string(),
        announcements,
        ttl,
        sequence_number,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gppn_core::types::FiatCurrency;

    fn usd() -> Currency {
        Currency::Fiat(FiatCurrency::USD)
    }

    #[test]
    fn test_self_advertisement() {
        let did = Did::from_parts("key", "alice");
        let ad = RouteAdvertisement::self_advertisement(
            did.clone(),
            "peer-alice".into(),
            vec![usd()],
            1_000_000,
            0.001,
            Duration::from_secs(300),
            1,
        );

        assert_eq!(ad.destination_count(), 1);
        assert!(ad.validate().is_ok());
        assert_eq!(ad.reachable_destinations[0].hop_count, 0);
    }

    #[test]
    fn test_process_advertisement() {
        let drt = DistributedRoutingTable::new();
        let did = Did::from_parts("key", "alice");
        let ad = RouteAdvertisement::self_advertisement(
            did.clone(),
            "peer-alice".into(),
            vec![usd()],
            1_000_000,
            0.001,
            Duration::from_secs(300),
            1,
        );

        let count = process_advertisement(&drt, &ad, 10).unwrap();
        assert_eq!(count, 1);
        assert_eq!(drt.size(), 1);

        let routes = drt.get_routes(&did);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].hop_count, 1); // 0 + 1
    }

    #[test]
    fn test_process_advertisement_max_hops_exceeded() {
        let drt = DistributedRoutingTable::new();
        let dest = DestinationAnnouncement {
            destination_did: Did::from_parts("key", "far-away"),
            supported_currencies: vec![usd()],
            available_liquidity: 1_000_000,
            fee_rate: 0.01,
            avg_latency_ms: 500,
            trust_score: 0.7,
            hop_count: 9, // will become 10 after +1
        };

        let ad = RouteAdvertisement::new(
            Did::from_parts("key", "relay"),
            "peer-relay".into(),
            vec![dest],
            Duration::from_secs(300),
            1,
        );

        // max_hops=10, so hop_count 10 is just within bounds.
        let count = process_advertisement(&drt, &ad, 10).unwrap();
        assert_eq!(count, 1);

        // Now try with max_hops=9, which should reject it.
        let drt2 = DistributedRoutingTable::new();
        let dest2 = DestinationAnnouncement {
            destination_did: Did::from_parts("key", "far-away"),
            supported_currencies: vec![usd()],
            available_liquidity: 1_000_000,
            fee_rate: 0.01,
            avg_latency_ms: 500,
            trust_score: 0.7,
            hop_count: 9,
        };
        let ad2 = RouteAdvertisement::new(
            Did::from_parts("key", "relay"),
            "peer-relay".into(),
            vec![dest2],
            Duration::from_secs(300),
            2,
        );
        let count2 = process_advertisement(&drt2, &ad2, 9).unwrap();
        assert_eq!(count2, 0);
    }

    #[test]
    fn test_create_readvertisement() {
        let drt = DistributedRoutingTable::new();
        // Populate the DRT with some entries.
        let entry = RouteEntry {
            next_hop_peer_id: "peer-bob".into(),
            destination_did: Did::from_parts("key", "bob"),
            supported_currencies: vec![usd()],
            available_liquidity: 2_000_000,
            fee_rate: 0.005,
            avg_latency_ms: 100,
            trust_score: 0.85,
            last_updated: Utc::now(),
            ttl: Duration::from_secs(300),
            hop_count: 2,
        };
        drt.insert(entry);

        let readvert = create_readvertisement(
            &Did::from_parts("key", "me"),
            "peer-me",
            &drt,
            Duration::from_secs(300),
            5,
            10,
        );

        assert_eq!(readvert.destination_count(), 1);
        assert_eq!(readvert.reachable_destinations[0].hop_count, 3); // 2 + 1
    }

    #[test]
    fn test_validate_bad_advertisement() {
        let ad = RouteAdvertisement::new(
            Did::from_parts("key", "alice"),
            String::new(), // empty peer ID
            vec![DestinationAnnouncement {
                destination_did: Did::from_parts("key", "bob"),
                supported_currencies: vec![usd()],
                available_liquidity: 100,
                fee_rate: 0.001,
                avg_latency_ms: 10,
                trust_score: 0.9,
                hop_count: 1,
            }],
            Duration::from_secs(300),
            1,
        );

        assert!(ad.validate().is_err());
    }
}
