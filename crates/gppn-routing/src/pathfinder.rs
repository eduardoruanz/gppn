use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use gppn_core::{Currency, Did};

use crate::drt::{DistributedRoutingTable, RouteEntry};
use crate::error::RoutingError;
use crate::route::Route;
use crate::scoring::{RouteScore, ScoringInput, ScoringWeights};

/// Configuration for the PathFinder algorithm.
#[derive(Debug, Clone)]
pub struct PathFinderConfig {
    /// Maximum number of hops allowed in a single route.
    pub max_hops: u32,
    /// Minimum trust score to consider a hop viable.
    pub min_trust_score: f64,
    /// Scoring weights for ranking discovered routes.
    pub weights: ScoringWeights,
}

impl Default for PathFinderConfig {
    fn default() -> Self {
        Self {
            max_hops: 10,
            min_trust_score: 0.0,
            weights: ScoringWeights::default(),
        }
    }
}

/// The PathFinder discovers optimal payment routes through the Distributed
/// Routing Table using a modified Dijkstra's algorithm with score-based
/// edge weights.
pub struct PathFinder {
    config: PathFinderConfig,
}

/// Internal node representation for the priority queue.
#[derive(Debug, Clone)]
struct SearchNode {
    /// The identifier of the current node in the graph.
    /// Nodes are identified by DID URI strings (e.g., "did:gppn:key:B").
    node_id: String,
    /// Accumulated score so far (higher is better).
    score: f64,
    /// The path of RouteEntry hops taken to reach this node.
    path: Vec<RouteEntry>,
    /// Total cost accumulated along the path.
    total_cost: u128,
    /// Total latency accumulated along the path.
    total_latency_ms: u64,
    /// Minimum trust score along the path.
    min_trust: f64,
    /// Minimum liquidity along the path.
    min_liquidity: u128,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.score.to_bits() == other.score.to_bits()
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher score = better. BinaryHeap is a max-heap, so this is correct.
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PathFinder {
    /// Create a new PathFinder with the given configuration.
    pub fn new(config: PathFinderConfig) -> Self {
        Self { config }
    }

    /// Create a PathFinder with default configuration.
    pub fn with_defaults() -> Self {
        Self {
            config: PathFinderConfig::default(),
        }
    }

    /// Find up to `max_routes` routes from `from` to `to` for the given
    /// `amount` and `currency`.
    ///
    /// Uses a modified Dijkstra's algorithm where edge weights are derived
    /// from the composite scoring function. Since we want highest-scoring
    /// paths and use a max-heap, we explore the best partial paths first.
    ///
    /// Graph model:
    /// - Each node is identified by a DID URI (e.g., "did:gppn:key:B").
    /// - An edge from node X to node Y exists when the DRT contains an
    ///   entry whose `next_hop_peer_id` corresponds to X and whose
    ///   `destination_did` is Y.
    /// - The correspondence between peer IDs and DID URIs is established
    ///   by matching the `next_hop_peer_id` against the DID identifier
    ///   (last component of the DID URI).
    pub fn find_routes(
        &self,
        drt: &DistributedRoutingTable,
        from: &Did,
        to: &Did,
        amount: u128,
        currency: &Currency,
        max_routes: usize,
    ) -> Result<Vec<Route>, RoutingError> {
        if drt.is_empty() {
            return Err(RoutingError::EmptyRoutingTable);
        }

        let dest_uri = to.uri().to_string();
        let source_uri = from.uri().to_string();

        // Build the adjacency graph using DID URIs as node identifiers.
        let adjacency = self.build_adjacency(drt, amount, currency);

        let mut found_routes: Vec<Route> = Vec::new();
        let mut heap: BinaryHeap<SearchNode> = BinaryHeap::new();

        // Visit count: node_id -> number of times popped from the heap.
        // We allow popping a node up to `max_routes` times to find k-shortest paths.
        let mut visit_counts: HashMap<String, usize> = HashMap::new();

        // Start node.
        heap.push(SearchNode {
            node_id: source_uri.clone(),
            score: f64::MAX,
            path: Vec::new(),
            total_cost: 0,
            total_latency_ms: 0,
            min_trust: 1.0,
            min_liquidity: u128::MAX,
        });

        while let Some(current) = heap.pop() {
            if found_routes.len() >= max_routes {
                break;
            }

            let count = visit_counts.entry(current.node_id.clone()).or_insert(0);
            *count += 1;

            if *count > max_routes {
                continue;
            }

            // Reached the destination.
            if current.node_id == dest_uri {
                if !current.path.is_empty() {
                    let route = Route::new(
                        current.path.clone(),
                        currency.clone(),
                        amount,
                    );
                    found_routes.push(route);
                }
                continue;
            }

            // Max hops exceeded.
            if current.path.len() as u32 >= self.config.max_hops {
                continue;
            }

            // Expand neighbours.
            if let Some(edges) = adjacency.get(&current.node_id) {
                for (entry, next_node) in edges {
                    // Loop detection: don't revisit a node already on this path.
                    let already_in_path = current.path.iter().any(|h| {
                        h.destination_did.uri() == next_node.as_str()
                    });
                    if already_in_path || *next_node == source_uri {
                        continue;
                    }

                    // Trust threshold check.
                    let new_min_trust = current.min_trust.min(entry.trust_score);
                    if new_min_trust < self.config.min_trust_score {
                        continue;
                    }

                    let hop_cost = entry.fee_for(amount);
                    let new_total_cost = current.total_cost + hop_cost;
                    let new_total_latency = current.total_latency_ms + entry.avg_latency_ms;
                    let new_min_liquidity = current.min_liquidity.min(entry.available_liquidity);

                    let liquidity_score = if amount == 0 {
                        1.0
                    } else {
                        (new_min_liquidity as f64 / amount as f64).min(1.0)
                    };

                    let input = ScoringInput {
                        total_cost: new_total_cost,
                        total_latency_ms: new_total_latency,
                        trust_score: new_min_trust,
                        liquidity_score,
                    };
                    let score = RouteScore::compute(&input, &self.config.weights);

                    let mut new_path = current.path.clone();
                    new_path.push(entry.clone());

                    heap.push(SearchNode {
                        node_id: next_node.clone(),
                        score: score.value,
                        path: new_path,
                        total_cost: new_total_cost,
                        total_latency_ms: new_total_latency,
                        min_trust: new_min_trust,
                        min_liquidity: new_min_liquidity,
                    });
                }
            }
        }

        if found_routes.is_empty() {
            return Err(RoutingError::NoRouteFound {
                from: from.clone(),
                to: to.clone(),
            });
        }

        // Sort routes by score descending (best first).
        found_routes.sort_by(|a, b| {
            let sa = a.score(&self.config.weights);
            let sb = b.score(&self.config.weights);
            sb.value
                .partial_cmp(&sa.value)
                .unwrap_or(Ordering::Equal)
        });

        Ok(found_routes)
    }

    /// Build an adjacency list from the DRT, filtering by currency and liquidity.
    ///
    /// Returns a map: source_DID_URI -> Vec<(RouteEntry, target_DID_URI)>
    ///
    /// Each DRT entry `{next_hop_peer_id=P, destination_did=D}` creates
    /// a directed edge. The source node is identified by finding the DID
    /// URI that corresponds to peer P. The correspondence is established
    /// by matching `next_hop_peer_id` against DID identifiers present in
    /// the routing table.
    ///
    /// For the source node (which may not appear as any destination_did
    /// in the DRT), we also match using the DID URI directly: if the
    /// source DID URI's identifier matches a `next_hop_peer_id`, we
    /// create the mapping.
    fn build_adjacency(
        &self,
        drt: &DistributedRoutingTable,
        amount: u128,
        currency: &Currency,
    ) -> HashMap<String, Vec<(RouteEntry, String)>> {
        let mut adjacency: HashMap<String, Vec<(RouteEntry, String)>> = HashMap::new();

        // Collect all entries, filtering by currency and liquidity.
        let entries: Vec<RouteEntry> = drt
            .all_entries()
            .into_iter()
            .filter(|e| e.supports_currency(currency) && e.available_liquidity >= amount)
            .collect();

        // Build a mapping: peer_id -> DID URI.
        // We derive this from destination_did values in the DRT: every
        // destination DID has an identifier (its last ':'-separated component)
        // that serves as the peer's shorthand.
        let mut peer_to_did_uri: HashMap<String, String> = HashMap::new();
        for entry in &entries {
            let uri = entry.destination_did.uri().to_string();
            if let Some(id) = entry.destination_did.identifier() {
                peer_to_did_uri.insert(id.to_string(), uri);
            }
        }

        // Also include peer IDs that only appear as next_hop_peer_id (like
        // the source node). For these, we construct a DID URI from the
        // peer ID using the same DID method as other nodes in the table.
        //
        // Determine the DID method from existing entries.
        let did_method = entries
            .first()
            .and_then(|e| e.destination_did.method().map(|m| m.to_string()))
            .unwrap_or_else(|| "key".to_string());

        for entry in &entries {
            let peer_id = &entry.next_hop_peer_id;
            if !peer_to_did_uri.contains_key(peer_id) {
                let synthetic_did = Did::from_parts(&did_method, peer_id);
                peer_to_did_uri.insert(peer_id.clone(), synthetic_did.uri().to_string());
            }
        }

        // Build the adjacency list.
        for entry in entries {
            let target_uri = entry.destination_did.uri().to_string();
            let source_uri = peer_to_did_uri
                .get(&entry.next_hop_peer_id)
                .cloned()
                .unwrap_or_else(|| entry.next_hop_peer_id.clone());

            adjacency
                .entry(source_uri)
                .or_default()
                .push((entry, target_uri));
        }

        adjacency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drt::RouteEntry;
    use chrono::Utc;
    use gppn_core::types::FiatCurrency;
    use std::time::Duration;

    fn usd() -> Currency {
        Currency::Fiat(FiatCurrency::USD)
    }

    /// Helper to insert a directed edge into the DRT.
    /// "from_peer can forward to dest_did, with the given parameters."
    fn insert_edge(
        drt: &DistributedRoutingTable,
        from_peer: &str,
        dest_did_id: &str,
        fee_rate: f64,
        latency_ms: u64,
        trust: f64,
        liquidity: u128,
    ) {
        let entry = RouteEntry {
            next_hop_peer_id: from_peer.to_string(),
            destination_did: Did::from_parts("key", dest_did_id),
            supported_currencies: vec![usd()],
            available_liquidity: liquidity,
            fee_rate,
            avg_latency_ms: latency_ms,
            trust_score: trust,
            last_updated: Utc::now(),
            ttl: Duration::from_secs(300),
            hop_count: 1,
        };
        drt.insert(entry);
    }

    /// Build a synthetic 5-node network:
    ///
    /// ```text
    ///          B ----> D
    ///         / \       \
    ///   A --+    +-----> E (destination)
    ///         \         /
    ///          C ------/
    /// ```
    ///
    /// A = source (did:gppn:key:A, peer "A")
    /// E = destination (did:gppn:key:E)
    ///
    /// Edges (from_peer -> destination):
    ///   A -> B  (cheap, fast, trusted)
    ///   A -> C  (medium cost, medium speed)
    ///   B -> D  (cheap, fast)
    ///   B -> E  (direct but expensive)
    ///   C -> E  (direct, cheap, but higher latency)
    ///   D -> E  (cheap, fast)
    fn build_five_node_network() -> DistributedRoutingTable {
        let drt = DistributedRoutingTable::new();

        // A -> B
        insert_edge(&drt, "A", "B", 0.001, 20, 0.95, 5_000_000);
        // A -> C
        insert_edge(&drt, "A", "C", 0.005, 50, 0.85, 3_000_000);
        // B -> D
        insert_edge(&drt, "B", "D", 0.001, 15, 0.90, 4_000_000);
        // B -> E (direct, expensive)
        insert_edge(&drt, "B", "E", 0.05, 30, 0.80, 2_000_000);
        // C -> E (direct, cheap, slow)
        insert_edge(&drt, "C", "E", 0.002, 200, 0.88, 3_000_000);
        // D -> E
        insert_edge(&drt, "D", "E", 0.001, 10, 0.92, 4_000_000);

        drt
    }

    #[test]
    fn test_find_routes_basic() {
        let drt = build_five_node_network();
        let pf = PathFinder::with_defaults();

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let routes = pf
            .find_routes(&drt, &from, &to, 100_000, &usd(), 5)
            .unwrap();

        assert!(!routes.is_empty(), "should find at least one route");

        // Verify all routes end at E.
        for route in &routes {
            let last = route.hops().last().unwrap();
            assert_eq!(last.destination_did.uri(), "did:gppn:key:E");
        }
    }

    #[test]
    fn test_find_routes_returns_multiple() {
        let drt = build_five_node_network();
        let pf = PathFinder::with_defaults();

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let routes = pf
            .find_routes(&drt, &from, &to, 100_000, &usd(), 10)
            .unwrap();

        // There are 3 possible paths: A->B->E, A->B->D->E, A->C->E
        assert!(
            routes.len() >= 2,
            "expected at least 2 routes, got {}",
            routes.len()
        );
    }

    #[test]
    fn test_find_routes_sorted_by_score() {
        let drt = build_five_node_network();
        let weights = ScoringWeights::default();
        let config = PathFinderConfig {
            max_hops: 10,
            min_trust_score: 0.0,
            weights: weights.clone(),
        };
        let pf = PathFinder::new(config);

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let routes = pf
            .find_routes(&drt, &from, &to, 100_000, &usd(), 10)
            .unwrap();

        // Verify scores are in descending order.
        for window in routes.windows(2) {
            let s0 = window[0].score(&weights);
            let s1 = window[1].score(&weights);
            assert!(
                s0.value >= s1.value,
                "routes should be sorted descending: {} >= {}",
                s0.value,
                s1.value
            );
        }
    }

    #[test]
    fn test_find_routes_no_route() {
        let drt = DistributedRoutingTable::new();
        // Add a single edge that doesn't connect to destination.
        insert_edge(&drt, "A", "B", 0.001, 20, 0.95, 5_000_000);

        let pf = PathFinder::with_defaults();
        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "Z");

        let result = pf.find_routes(&drt, &from, &to, 100_000, &usd(), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_routes_empty_table() {
        let drt = DistributedRoutingTable::new();
        let pf = PathFinder::with_defaults();

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let result = pf.find_routes(&drt, &from, &to, 100_000, &usd(), 5);
        assert!(matches!(result, Err(RoutingError::EmptyRoutingTable)));
    }

    #[test]
    fn test_find_routes_insufficient_liquidity() {
        let drt = DistributedRoutingTable::new();
        // Edge with low liquidity.
        insert_edge(&drt, "A", "E", 0.001, 20, 0.95, 100); // only 100 units

        let pf = PathFinder::with_defaults();
        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        // Request 1_000_000 units, but only 100 available.
        let result = pf.find_routes(&drt, &from, &to, 1_000_000, &usd(), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_routes_currency_filter() {
        let drt = DistributedRoutingTable::new();
        // Insert edge that only supports EUR.
        let entry = RouteEntry {
            next_hop_peer_id: "A".to_string(),
            destination_did: Did::from_parts("key", "E"),
            supported_currencies: vec![Currency::Fiat(FiatCurrency::EUR)],
            available_liquidity: 5_000_000,
            fee_rate: 0.001,
            avg_latency_ms: 20,
            trust_score: 0.95,
            last_updated: Utc::now(),
            ttl: Duration::from_secs(300),
            hop_count: 1,
        };
        drt.insert(entry);

        let pf = PathFinder::with_defaults();
        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        // Request in USD, but only EUR available.
        let result = pf.find_routes(&drt, &from, &to, 100_000, &usd(), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_routes_trust_threshold() {
        let drt = DistributedRoutingTable::new();
        insert_edge(&drt, "A", "E", 0.001, 20, 0.3, 5_000_000); // low trust

        let config = PathFinderConfig {
            max_hops: 10,
            min_trust_score: 0.5, // require at least 0.5
            weights: ScoringWeights::default(),
        };
        let pf = PathFinder::new(config);

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let result = pf.find_routes(&drt, &from, &to, 100_000, &usd(), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_routes_max_hops_limit() {
        let drt = DistributedRoutingTable::new();
        // Create a chain: A -> B -> C -> D -> E (4 hops).
        insert_edge(&drt, "A", "B", 0.001, 10, 0.9, 5_000_000);
        insert_edge(&drt, "B", "C", 0.001, 10, 0.9, 5_000_000);
        insert_edge(&drt, "C", "D", 0.001, 10, 0.9, 5_000_000);
        insert_edge(&drt, "D", "E", 0.001, 10, 0.9, 5_000_000);

        // With max_hops=2, the 4-hop chain should not be found.
        let config = PathFinderConfig {
            max_hops: 2,
            min_trust_score: 0.0,
            weights: ScoringWeights::default(),
        };
        let pf = PathFinder::new(config);

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let result = pf.find_routes(&drt, &from, &to, 100_000, &usd(), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_five_node_route_properties() {
        let drt = build_five_node_network();
        let pf = PathFinder::with_defaults();

        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let routes = pf
            .find_routes(&drt, &from, &to, 100_000, &usd(), 10)
            .unwrap();

        for route in &routes {
            // All hops should support USD.
            assert!(route.all_hops_support_currency());
            // All hops should have enough liquidity.
            assert!(route.all_hops_have_liquidity());
            // Total cost should be positive.
            assert!(route.total_cost() > 0);
            // Latency should be positive.
            assert!(route.estimated_latency() > 0);
            // Trust should be in [0, 1].
            let trust = route.min_trust_score();
            assert!((0.0..=1.0).contains(&trust));
        }
    }

    #[test]
    fn test_direct_single_hop_preferred() {
        let drt = DistributedRoutingTable::new();
        // Direct A->E: cheap, fast.
        insert_edge(&drt, "A", "E", 0.001, 10, 0.99, 10_000_000);
        // Indirect A->B->E: also cheap, but 2 hops.
        insert_edge(&drt, "A", "B", 0.001, 10, 0.99, 10_000_000);
        insert_edge(&drt, "B", "E", 0.001, 10, 0.99, 10_000_000);

        let pf = PathFinder::with_defaults();
        let from = Did::from_parts("key", "A");
        let to = Did::from_parts("key", "E");

        let routes = pf
            .find_routes(&drt, &from, &to, 100_000, &usd(), 5)
            .unwrap();

        // The best (first) route should be the direct single-hop route
        // because it has lower cost and latency.
        assert_eq!(
            routes[0].hop_count(),
            1,
            "direct route should be ranked first"
        );
    }
}
