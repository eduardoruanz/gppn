use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::error::IdentityError;

/// A directed trust edge between two DIDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEdge {
    /// Source DID (the truster).
    pub from_did: String,
    /// Target DID (the trusted).
    pub to_did: String,
    /// Trust weight in range [-1.0, 1.0].
    /// Positive = trust, negative = distrust, 0 = neutral.
    pub weight: f64,
    /// Number of interactions that informed this edge.
    pub interactions: u64,
    /// When this edge was last updated.
    pub last_updated: DateTime<Utc>,
}

/// Directed weighted trust graph using DashMap for concurrent access.
///
/// Edges are keyed by `(from_did, to_did)`.
pub struct TrustGraph {
    /// Edges keyed by (from_did, to_did).
    edges: DashMap<(String, String), TrustEdge>,
}

impl TrustGraph {
    /// Create a new, empty trust graph.
    pub fn new() -> Self {
        Self {
            edges: DashMap::new(),
        }
    }

    /// Add or update a trust edge.
    ///
    /// If the edge already exists, the weight is updated and the interaction
    /// count is incremented.
    pub fn add_edge(&self, from_did: &str, to_did: &str, weight: f64) -> Result<(), IdentityError> {
        if !(-1.0..=1.0).contains(&weight) {
            return Err(IdentityError::InvalidTrustWeight(weight));
        }

        let key = (from_did.to_string(), to_did.to_string());
        self.edges
            .entry(key)
            .and_modify(|edge| {
                edge.weight = weight;
                edge.interactions += 1;
                edge.last_updated = Utc::now();
            })
            .or_insert_with(|| TrustEdge {
                from_did: from_did.to_string(),
                to_did: to_did.to_string(),
                weight,
                interactions: 1,
                last_updated: Utc::now(),
            });

        Ok(())
    }

    /// Get the direct trust score from one DID to another.
    pub fn get_score(&self, from_did: &str, to_did: &str) -> Option<f64> {
        let key = (from_did.to_string(), to_did.to_string());
        self.edges.get(&key).map(|edge| edge.weight)
    }

    /// Get the trust edge between two DIDs.
    pub fn get_edge(&self, from_did: &str, to_did: &str) -> Option<TrustEdge> {
        let key = (from_did.to_string(), to_did.to_string());
        self.edges.get(&key).map(|e| e.clone())
    }

    /// Remove a trust edge.
    pub fn remove_edge(&self, from_did: &str, to_did: &str) -> Option<TrustEdge> {
        let key = (from_did.to_string(), to_did.to_string());
        self.edges.remove(&key).map(|(_, edge)| edge)
    }

    /// Get all outgoing edges from a DID.
    pub fn outgoing_edges(&self, from_did: &str) -> Vec<TrustEdge> {
        self.edges
            .iter()
            .filter(|entry| entry.key().0 == from_did)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all incoming edges to a DID.
    pub fn incoming_edges(&self, to_did: &str) -> Vec<TrustEdge> {
        self.edges
            .iter()
            .filter(|entry| entry.key().1 == to_did)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Collect all unique DIDs in the graph.
    fn all_dids(&self) -> Vec<String> {
        let mut dids = HashSet::new();
        for entry in self.edges.iter() {
            dids.insert(entry.key().0.clone());
            dids.insert(entry.key().1.clone());
        }
        dids.into_iter().collect()
    }

    /// Compute global trust scores using an iterative power method
    /// (EigenTrust-like algorithm).
    ///
    /// Returns a map from DID to its computed global trust score.
    ///
    /// Parameters:
    /// - `max_iterations`: Maximum number of power iterations.
    /// - `convergence_threshold`: Stop when the max score change is below this.
    pub fn compute_scores(
        &self,
        max_iterations: usize,
        convergence_threshold: f64,
    ) -> HashMap<String, f64> {
        let dids = self.all_dids();
        if dids.is_empty() {
            return HashMap::new();
        }

        let n = dids.len();
        let did_to_idx: HashMap<&str, usize> = dids
            .iter()
            .enumerate()
            .map(|(i, d)| (d.as_str(), i))
            .collect();

        // Build normalized adjacency structure.
        // For each node, collect positive outgoing weights and normalize them.
        let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        for entry in self.edges.iter() {
            let (from, to) = entry.key();
            let weight = entry.value().weight;
            if weight > 0.0 {
                if let (Some(&from_idx), Some(&to_idx)) =
                    (did_to_idx.get(from.as_str()), did_to_idx.get(to.as_str()))
                {
                    adj[from_idx].push((to_idx, weight));
                }
            }
        }

        // Normalize outgoing weights per node.
        for neighbors in &mut adj {
            let sum: f64 = neighbors.iter().map(|(_, w)| *w).sum();
            if sum > 0.0 {
                for (_, w) in neighbors.iter_mut() {
                    *w /= sum;
                }
            }
        }

        // Initialize scores uniformly.
        let initial = 1.0 / n as f64;
        let mut scores = vec![initial; n];

        // Pre-trust vector: uniform for now.
        let pre_trust = vec![initial; n];
        let alpha = 0.15; // damping factor toward pre-trust

        for _iter in 0..max_iterations {
            let mut new_scores = vec![0.0; n];

            // Compute trust propagation.
            for i in 0..n {
                for &(j, w) in &adj[i] {
                    new_scores[j] += scores[i] * w;
                }
            }

            // Apply damping.
            for i in 0..n {
                new_scores[i] = alpha * pre_trust[i] + (1.0 - alpha) * new_scores[i];
            }

            // Normalize.
            let sum: f64 = new_scores.iter().sum();
            if sum > 0.0 {
                for s in &mut new_scores {
                    *s /= sum;
                }
            }

            // Check convergence.
            let max_diff = scores
                .iter()
                .zip(new_scores.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0_f64, f64::max);

            scores = new_scores;

            if max_diff < convergence_threshold {
                break;
            }
        }

        // Map back to DIDs.
        dids.into_iter()
            .enumerate()
            .map(|(i, did)| (did, scores[i]))
            .collect()
    }

    /// Get the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

impl Default for TrustGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 0.8).unwrap();

        let score = graph.get_score("did:a", "did:b").unwrap();
        assert!((score - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_add_edge_invalid_weight() {
        let graph = TrustGraph::new();
        let result = graph.add_edge("did:a", "did:b", 1.5);
        assert!(matches!(result, Err(IdentityError::InvalidTrustWeight(_))));

        let result = graph.add_edge("did:a", "did:b", -1.5);
        assert!(matches!(result, Err(IdentityError::InvalidTrustWeight(_))));
    }

    #[test]
    fn test_add_edge_boundary_weights() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 1.0).unwrap();
        graph.add_edge("did:c", "did:d", -1.0).unwrap();
        graph.add_edge("did:e", "did:f", 0.0).unwrap();
    }

    #[test]
    fn test_update_edge() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 0.5).unwrap();
        graph.add_edge("did:a", "did:b", 0.9).unwrap();

        let edge = graph.get_edge("did:a", "did:b").unwrap();
        assert!((edge.weight - 0.9).abs() < f64::EPSILON);
        assert_eq!(edge.interactions, 2);
    }

    #[test]
    fn test_get_score_nonexistent() {
        let graph = TrustGraph::new();
        assert!(graph.get_score("did:a", "did:b").is_none());
    }

    #[test]
    fn test_remove_edge() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 0.7).unwrap();
        let removed = graph.remove_edge("did:a", "did:b");
        assert!(removed.is_some());
        assert!(graph.get_score("did:a", "did:b").is_none());
    }

    #[test]
    fn test_outgoing_edges() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 0.8).unwrap();
        graph.add_edge("did:a", "did:c", 0.6).unwrap();
        graph.add_edge("did:b", "did:c", 0.5).unwrap();

        let outgoing = graph.outgoing_edges("did:a");
        assert_eq!(outgoing.len(), 2);
    }

    #[test]
    fn test_incoming_edges() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:c", 0.8).unwrap();
        graph.add_edge("did:b", "did:c", 0.6).unwrap();

        let incoming = graph.incoming_edges("did:c");
        assert_eq!(incoming.len(), 2);
    }

    #[test]
    fn test_compute_scores_simple() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 1.0).unwrap();
        graph.add_edge("did:b", "did:c", 1.0).unwrap();
        graph.add_edge("did:c", "did:a", 1.0).unwrap();

        let scores = graph.compute_scores(100, 1e-6);
        assert_eq!(scores.len(), 3);

        // In a symmetric 3-cycle, all scores should be equal.
        let a = scores["did:a"];
        let b = scores["did:b"];
        let c = scores["did:c"];
        assert!((a - b).abs() < 0.01);
        assert!((b - c).abs() < 0.01);
    }

    #[test]
    fn test_compute_scores_empty() {
        let graph = TrustGraph::new();
        let scores = graph.compute_scores(100, 1e-6);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_compute_scores_star_topology() {
        let graph = TrustGraph::new();
        // Everyone trusts "did:hub"
        graph.add_edge("did:a", "did:hub", 1.0).unwrap();
        graph.add_edge("did:b", "did:hub", 1.0).unwrap();
        graph.add_edge("did:c", "did:hub", 1.0).unwrap();
        // Hub trusts everyone back
        graph.add_edge("did:hub", "did:a", 0.5).unwrap();
        graph.add_edge("did:hub", "did:b", 0.5).unwrap();
        graph.add_edge("did:hub", "did:c", 0.5).unwrap();

        let scores = graph.compute_scores(100, 1e-6);
        // Hub should have a higher score because more nodes trust it.
        let hub_score = scores["did:hub"];
        let a_score = scores["did:a"];
        assert!(hub_score > a_score);
    }

    #[test]
    fn test_compute_scores_negative_edges_ignored() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 1.0).unwrap();
        graph.add_edge("did:b", "did:a", -1.0).unwrap(); // distrust

        let scores = graph.compute_scores(100, 1e-6);
        // Both should still have scores (damping guarantees non-zero).
        assert!(scores["did:a"] > 0.0);
        assert!(scores["did:b"] > 0.0);
    }

    #[test]
    fn test_scores_sum_to_one() {
        let graph = TrustGraph::new();
        graph.add_edge("did:a", "did:b", 0.8).unwrap();
        graph.add_edge("did:b", "did:c", 0.6).unwrap();
        graph.add_edge("did:c", "did:a", 0.9).unwrap();
        graph.add_edge("did:a", "did:c", 0.3).unwrap();

        let scores = graph.compute_scores(100, 1e-6);
        let sum: f64 = scores.values().sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_edge_count() {
        let graph = TrustGraph::new();
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.is_empty());

        graph.add_edge("did:a", "did:b", 0.5).unwrap();
        assert_eq!(graph.edge_count(), 1);
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_graph_default() {
        let graph = TrustGraph::default();
        assert!(graph.is_empty());
    }
}
