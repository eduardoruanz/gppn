use gppn_core::Currency;
use serde::{Deserialize, Serialize};

use crate::drt::RouteEntry;
use crate::scoring::{RouteScore, ScoringInput, ScoringWeights};

/// A complete route from source to destination, consisting of an ordered
/// list of hops (RouteEntry values).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Ordered hops from source toward destination.
    hops: Vec<RouteEntry>,
    /// The currency this route was evaluated for.
    pub currency: Currency,
    /// The amount being routed (atomic units).
    pub amount: u128,
}

impl Route {
    /// Create a new route from a list of hops.
    pub fn new(hops: Vec<RouteEntry>, currency: Currency, amount: u128) -> Self {
        Self {
            hops,
            currency,
            amount,
        }
    }

    /// The ordered hop entries.
    pub fn hops(&self) -> &[RouteEntry] {
        &self.hops
    }

    /// Number of hops in this route.
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }

    /// Total cost (sum of fees) along the route in atomic units.
    ///
    /// Each hop applies its fee_rate to the amount that enters it.
    /// The first hop sees the original `amount`, the next sees
    /// `amount + fee_0`, etc. For simplicity we compute the fee at each
    /// hop on the original amount (additive model).
    pub fn total_cost(&self) -> u128 {
        self.hops.iter().map(|h| h.fee_for(self.amount)).sum()
    }

    /// Estimated end-to-end latency in milliseconds (sum of per-hop latencies).
    pub fn estimated_latency(&self) -> u64 {
        self.hops.iter().map(|h| h.avg_latency_ms).sum()
    }

    /// Minimum trust score across all hops (the weakest link).
    pub fn min_trust_score(&self) -> f64 {
        self.hops
            .iter()
            .map(|h| h.trust_score)
            .fold(f64::INFINITY, f64::min)
    }

    /// Minimum available liquidity across all hops.
    pub fn min_liquidity(&self) -> u128 {
        self.hops
            .iter()
            .map(|h| h.available_liquidity)
            .min()
            .unwrap_or(0)
    }

    /// Compute a liquidity score in [0, 1] as `min(min_liquidity / amount, 1.0)`.
    pub fn liquidity_score(&self) -> f64 {
        if self.amount == 0 {
            return 1.0;
        }
        let min_liq = self.min_liquidity() as f64;
        (min_liq / self.amount as f64).min(1.0)
    }

    /// Score this route using the provided weights.
    pub fn score(&self, weights: &ScoringWeights) -> RouteScore {
        let input = ScoringInput {
            total_cost: self.total_cost(),
            total_latency_ms: self.estimated_latency(),
            trust_score: self.min_trust_score(),
            liquidity_score: self.liquidity_score(),
        };
        RouteScore::compute(&input, weights)
    }

    /// Returns true if all hops support the route's currency.
    pub fn all_hops_support_currency(&self) -> bool {
        self.hops.iter().all(|h| h.supports_currency(&self.currency))
    }

    /// Returns true if all hops have enough liquidity for the route amount.
    pub fn all_hops_have_liquidity(&self) -> bool {
        self.hops
            .iter()
            .all(|h| h.available_liquidity >= self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drt::RouteEntry;
    use chrono::Utc;
    use gppn_core::{Currency, Did};
    use gppn_core::types::FiatCurrency;
    use std::time::Duration;

    fn make_hop(peer: &str, dest: &str, fee_rate: f64, latency: u64, trust: f64, liquidity: u128) -> RouteEntry {
        RouteEntry {
            next_hop_peer_id: peer.to_string(),
            destination_did: Did::from_parts("key", dest),
            supported_currencies: vec![Currency::Fiat(FiatCurrency::USD)],
            available_liquidity: liquidity,
            fee_rate,
            avg_latency_ms: latency,
            trust_score: trust,
            last_updated: Utc::now(),
            ttl: Duration::from_secs(300),
            hop_count: 1,
        }
    }

    #[test]
    fn test_single_hop_route() {
        let route = Route::new(
            vec![make_hop("p1", "alice", 0.01, 50, 0.95, 1_000_000)],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );

        // fee = ceil(100_000 * 0.01) = 1000
        assert_eq!(route.total_cost(), 1_000);
        assert_eq!(route.estimated_latency(), 50);
        assert!((route.min_trust_score() - 0.95).abs() < f64::EPSILON);
        assert_eq!(route.hop_count(), 1);
    }

    #[test]
    fn test_multi_hop_route() {
        let route = Route::new(
            vec![
                make_hop("p1", "relay1", 0.01, 50, 0.95, 500_000),
                make_hop("p2", "relay2", 0.005, 30, 0.80, 1_000_000),
                make_hop("p3", "alice", 0.002, 20, 0.90, 2_000_000),
            ],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );

        // Fees: ceil(100_000*0.01)=1000, ceil(100_000*0.005)=500, ceil(100_000*0.002)=200
        assert_eq!(route.total_cost(), 1_700);
        assert_eq!(route.estimated_latency(), 100); // 50+30+20
        assert!((route.min_trust_score() - 0.80).abs() < f64::EPSILON); // min trust
        assert_eq!(route.min_liquidity(), 500_000);
        assert_eq!(route.hop_count(), 3);
    }

    #[test]
    fn test_liquidity_score() {
        let route = Route::new(
            vec![make_hop("p1", "alice", 0.01, 50, 0.9, 50_000)],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );
        // liquidity 50_000 / amount 100_000 = 0.5
        assert!((route.liquidity_score() - 0.5).abs() < f64::EPSILON);

        let route2 = Route::new(
            vec![make_hop("p1", "alice", 0.01, 50, 0.9, 200_000)],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );
        // liquidity 200_000 / amount 100_000 = 2.0 → clamped to 1.0
        assert!((route2.liquidity_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_route_score_ordering() {
        let weights = ScoringWeights::default();

        let good_route = Route::new(
            vec![make_hop("p1", "alice", 0.001, 20, 0.99, 5_000_000)],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );
        let bad_route = Route::new(
            vec![
                make_hop("p1", "relay", 0.05, 500, 0.3, 50_000),
                make_hop("p2", "alice", 0.05, 500, 0.3, 50_000),
            ],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );

        let good_score = good_route.score(&weights);
        let bad_score = bad_route.score(&weights);

        assert!(
            good_score.value > bad_score.value,
            "good route should outscore bad route: {} vs {}",
            good_score.value,
            bad_score.value
        );
    }

    #[test]
    fn test_currency_support_check() {
        let mut hop = make_hop("p1", "alice", 0.01, 50, 0.9, 1_000_000);
        hop.supported_currencies = vec![Currency::Fiat(FiatCurrency::EUR)];

        let route = Route::new(
            vec![hop],
            Currency::Fiat(FiatCurrency::USD),
            100_000,
        );
        assert!(!route.all_hops_support_currency());
    }
}
