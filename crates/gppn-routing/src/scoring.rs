use serde::{Deserialize, Serialize};

use crate::error::RoutingError;

/// Configurable weights for the route scoring formula.
///
/// The score is computed as:
///   `score = alpha * (1/cost) + beta * (1/latency) + gamma * trust_score + delta * liquidity_score`
///
/// All weights must be in [0, 1] and sum to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// Weight for cost efficiency (inverse of total fee).
    pub alpha: f64,
    /// Weight for latency efficiency (inverse of total latency).
    pub beta: f64,
    /// Weight for trust score.
    pub gamma: f64,
    /// Weight for liquidity availability.
    pub delta: f64,
}

impl ScoringWeights {
    /// Create new scoring weights, validating that they sum to 1.0
    /// (within floating-point tolerance).
    pub fn new(alpha: f64, beta: f64, gamma: f64, delta: f64) -> Result<Self, RoutingError> {
        let sum = alpha + beta + gamma + delta;
        if (sum - 1.0).abs() > 1e-6 {
            return Err(RoutingError::InvalidScoringWeights { sum });
        }
        Ok(Self {
            alpha,
            beta,
            gamma,
            delta,
        })
    }

    /// Validate that weights sum to 1.0.
    pub fn validate(&self) -> Result<(), RoutingError> {
        let sum = self.alpha + self.beta + self.gamma + self.delta;
        if (sum - 1.0).abs() > 1e-6 {
            return Err(RoutingError::InvalidScoringWeights { sum });
        }
        Ok(())
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            alpha: 0.25,
            beta: 0.25,
            gamma: 0.30,
            delta: 0.20,
        }
    }
}

/// The raw inputs used to compute a route score.
#[derive(Debug, Clone)]
pub struct ScoringInput {
    /// Total cost (fee) in atomic units along the route.
    pub total_cost: u128,
    /// Total estimated latency in milliseconds along the route.
    pub total_latency_ms: u64,
    /// Minimum trust score across all hops (worst-case trust).
    pub trust_score: f64,
    /// Normalized liquidity score in [0, 1]. This is typically
    /// `min(available_liquidity / required_amount, 1.0)` across hops.
    pub liquidity_score: f64,
}

/// The result of scoring a route.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteScore {
    /// The composite score value. Higher is better.
    pub value: f64,
    /// The individual component values for transparency.
    pub cost_component: f64,
    pub latency_component: f64,
    pub trust_component: f64,
    pub liquidity_component: f64,
}

impl RouteScore {
    /// Compute a route score from the given inputs and weights.
    ///
    /// Formula:
    ///   `score = alpha * (1/cost) + beta * (1/latency) + gamma * trust_score + delta * liquidity_score`
    ///
    /// Cost and latency are inverted so that lower cost/latency yields a higher score.
    /// A normalisation factor is applied to keep the inverse terms in a useful range:
    /// - cost is normalised to `1 / (1 + cost_in_units)` to keep it in (0, 1].
    /// - latency is normalised to `1 / (1 + latency_ms)` to keep it in (0, 1].
    pub fn compute(input: &ScoringInput, weights: &ScoringWeights) -> Self {
        let cost_component = 1.0 / (1.0 + input.total_cost as f64);
        let latency_component = 1.0 / (1.0 + input.total_latency_ms as f64);
        let trust_component = input.trust_score;
        let liquidity_component = input.liquidity_score;

        let value = weights.alpha * cost_component
            + weights.beta * latency_component
            + weights.gamma * trust_component
            + weights.delta * liquidity_component;

        Self {
            value,
            cost_component,
            latency_component,
            trust_component,
            liquidity_component,
        }
    }
}

impl PartialOrd for RouteScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights_sum_to_one() {
        let weights = ScoringWeights::default();
        assert!(weights.validate().is_ok());
        let sum = weights.alpha + weights.beta + weights.gamma + weights.delta;
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_invalid_weights() {
        let result = ScoringWeights::new(0.5, 0.5, 0.5, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_weights() {
        let weights = ScoringWeights::new(0.1, 0.2, 0.3, 0.4).unwrap();
        assert!((weights.alpha - 0.1).abs() < f64::EPSILON);
        assert!((weights.delta - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scoring_with_known_inputs() {
        let weights = ScoringWeights::default();
        let input = ScoringInput {
            total_cost: 0, // free route
            total_latency_ms: 0, // instant
            trust_score: 1.0, // perfect trust
            liquidity_score: 1.0, // full liquidity
        };

        let score = RouteScore::compute(&input, &weights);
        // cost_component = 1/(1+0) = 1.0
        // latency_component = 1/(1+0) = 1.0
        // trust = 1.0, liquidity = 1.0
        // score = 0.25*1 + 0.25*1 + 0.30*1 + 0.20*1 = 1.0
        assert!((score.value - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_scoring_cost_sensitivity() {
        let weights = ScoringWeights::default();

        let cheap = ScoringInput {
            total_cost: 100,
            total_latency_ms: 100,
            trust_score: 0.8,
            liquidity_score: 0.8,
        };
        let expensive = ScoringInput {
            total_cost: 10_000,
            total_latency_ms: 100,
            trust_score: 0.8,
            liquidity_score: 0.8,
        };

        let cheap_score = RouteScore::compute(&cheap, &weights);
        let expensive_score = RouteScore::compute(&expensive, &weights);

        assert!(
            cheap_score.value > expensive_score.value,
            "cheaper route should score higher: {} vs {}",
            cheap_score.value,
            expensive_score.value
        );
    }

    #[test]
    fn test_scoring_latency_sensitivity() {
        let weights = ScoringWeights::default();

        let fast = ScoringInput {
            total_cost: 1000,
            total_latency_ms: 10,
            trust_score: 0.8,
            liquidity_score: 0.8,
        };
        let slow = ScoringInput {
            total_cost: 1000,
            total_latency_ms: 5000,
            trust_score: 0.8,
            liquidity_score: 0.8,
        };

        let fast_score = RouteScore::compute(&fast, &weights);
        let slow_score = RouteScore::compute(&slow, &weights);

        assert!(
            fast_score.value > slow_score.value,
            "faster route should score higher: {} vs {}",
            fast_score.value,
            slow_score.value
        );
    }

    #[test]
    fn test_scoring_trust_sensitivity() {
        let weights = ScoringWeights::default();

        let trusted = ScoringInput {
            total_cost: 1000,
            total_latency_ms: 100,
            trust_score: 0.99,
            liquidity_score: 0.8,
        };
        let untrusted = ScoringInput {
            total_cost: 1000,
            total_latency_ms: 100,
            trust_score: 0.1,
            liquidity_score: 0.8,
        };

        let trusted_score = RouteScore::compute(&trusted, &weights);
        let untrusted_score = RouteScore::compute(&untrusted, &weights);

        assert!(
            trusted_score.value > untrusted_score.value,
            "more trusted route should score higher"
        );
    }

    #[test]
    fn test_scoring_deterministic() {
        let weights = ScoringWeights::default();
        let input = ScoringInput {
            total_cost: 5000,
            total_latency_ms: 200,
            trust_score: 0.75,
            liquidity_score: 0.6,
        };

        let s1 = RouteScore::compute(&input, &weights);
        let s2 = RouteScore::compute(&input, &weights);
        assert!((s1.value - s2.value).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scoring_with_custom_weights_trust_heavy() {
        // Trust-heavy weighting: gamma = 0.70
        let weights = ScoringWeights::new(0.10, 0.10, 0.70, 0.10).unwrap();

        let high_trust = ScoringInput {
            total_cost: 10_000,
            total_latency_ms: 1000,
            trust_score: 0.99,
            liquidity_score: 0.5,
        };
        let low_trust = ScoringInput {
            total_cost: 100,
            total_latency_ms: 10,
            trust_score: 0.1,
            liquidity_score: 0.9,
        };

        let high_trust_score = RouteScore::compute(&high_trust, &weights);
        let low_trust_score = RouteScore::compute(&low_trust, &weights);

        // With 70% weight on trust, the high-trust route should win
        // even though it is more expensive and slower.
        assert!(
            high_trust_score.value > low_trust_score.value,
            "trust-heavy: {} vs {}",
            high_trust_score.value,
            low_trust_score.value
        );
    }
}
