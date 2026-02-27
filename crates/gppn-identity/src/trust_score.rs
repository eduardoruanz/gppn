use serde::{Deserialize, Serialize};

/// Weighted composite trust score for a GPPN network participant.
///
/// Each component is normalized to `[0.0, 1.0]` before weighting.
///
/// Weights:
/// - uptime:       0.20
/// - success_rate: 0.25
/// - avg_latency:  0.15  (inverted: lower latency = higher score)
/// - volume:       0.15
/// - age:          0.10
/// - attestations: 0.15
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    /// Fraction of time the node was reachable (0.0 - 1.0).
    pub uptime: f64,
    /// Fraction of successful payment operations (0.0 - 1.0).
    pub success_rate: f64,
    /// Average latency in milliseconds.  Lower is better.
    pub avg_latency_ms: f64,
    /// Normalised transaction volume (0.0 - 1.0).
    pub volume: f64,
    /// Normalised account age (0.0 - 1.0).
    pub age: f64,
    /// Normalised attestation count (0.0 - 1.0).
    pub attestations: f64,
}

/// Component weights for the trust score formula.
const WEIGHT_UPTIME: f64 = 0.20;
const WEIGHT_SUCCESS_RATE: f64 = 0.25;
const WEIGHT_AVG_LATENCY: f64 = 0.15;
const WEIGHT_VOLUME: f64 = 0.15;
const WEIGHT_AGE: f64 = 0.10;
const WEIGHT_ATTESTATIONS: f64 = 0.15;

/// Reference latency in milliseconds used to normalize the latency component.
/// A latency at or above this value yields a score of 0.
const REFERENCE_LATENCY_MS: f64 = 10_000.0;

impl TrustScore {
    /// Create a new trust score with all components.
    pub fn new(
        uptime: f64,
        success_rate: f64,
        avg_latency_ms: f64,
        volume: f64,
        age: f64,
        attestations: f64,
    ) -> Self {
        Self {
            uptime: uptime.clamp(0.0, 1.0),
            success_rate: success_rate.clamp(0.0, 1.0),
            avg_latency_ms: avg_latency_ms.max(0.0),
            volume: volume.clamp(0.0, 1.0),
            age: age.clamp(0.0, 1.0),
            attestations: attestations.clamp(0.0, 1.0),
        }
    }

    /// Calculate the composite trust score.
    ///
    /// Returns a value in `[0.0, 1.0]`.
    pub fn calculate(&self) -> f64 {
        // Invert latency: 0ms = 1.0, REFERENCE_LATENCY_MS = 0.0
        let latency_score = (1.0 - (self.avg_latency_ms / REFERENCE_LATENCY_MS)).clamp(0.0, 1.0);

        let score = WEIGHT_UPTIME * self.uptime
            + WEIGHT_SUCCESS_RATE * self.success_rate
            + WEIGHT_AVG_LATENCY * latency_score
            + WEIGHT_VOLUME * self.volume
            + WEIGHT_AGE * self.age
            + WEIGHT_ATTESTATIONS * self.attestations;

        score.clamp(0.0, 1.0)
    }

    /// Create a perfect trust score (all components = 1.0, latency = 0ms).
    pub fn perfect() -> Self {
        Self {
            uptime: 1.0,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            volume: 1.0,
            age: 1.0,
            attestations: 1.0,
        }
    }

    /// Create a zero trust score.
    pub fn zero() -> Self {
        Self {
            uptime: 0.0,
            success_rate: 0.0,
            avg_latency_ms: REFERENCE_LATENCY_MS,
            volume: 0.0,
            age: 0.0,
            attestations: 0.0,
        }
    }
}

impl Default for TrustScore {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_score() {
        let score = TrustScore::perfect();
        let value = score.calculate();
        assert!((value - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_score() {
        let score = TrustScore::zero();
        let value = score.calculate();
        assert!((value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_intermediate() {
        let score = TrustScore::new(
            0.95,  // uptime
            0.90,  // success_rate
            200.0, // avg_latency_ms
            0.50,  // volume
            0.70,  // age
            0.80,  // attestations
        );
        let value = score.calculate();
        assert!(value > 0.0 && value < 1.0);
    }

    #[test]
    fn test_calculate_weights_sum_to_one() {
        let sum = WEIGHT_UPTIME
            + WEIGHT_SUCCESS_RATE
            + WEIGHT_AVG_LATENCY
            + WEIGHT_VOLUME
            + WEIGHT_AGE
            + WEIGHT_ATTESTATIONS;
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_latency_inversion() {
        // Low latency should give higher score.
        let low_latency = TrustScore::new(0.5, 0.5, 100.0, 0.5, 0.5, 0.5);
        let high_latency = TrustScore::new(0.5, 0.5, 5000.0, 0.5, 0.5, 0.5);
        assert!(low_latency.calculate() > high_latency.calculate());
    }

    #[test]
    fn test_clamping() {
        // Values outside [0, 1] should be clamped.
        let score = TrustScore::new(1.5, -0.1, -100.0, 2.0, -0.5, 1.1);
        assert!((score.uptime - 1.0).abs() < f64::EPSILON);
        assert!((score.success_rate - 0.0).abs() < f64::EPSILON);
        assert!((score.avg_latency_ms - 0.0).abs() < f64::EPSILON);
        assert!((score.volume - 1.0).abs() < f64::EPSILON);
        assert!((score.age - 0.0).abs() < f64::EPSILON);
        assert!((score.attestations - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_range() {
        // Score should always be in [0, 1].
        let score = TrustScore::new(0.0, 0.0, 999_999.0, 0.0, 0.0, 0.0);
        let value = score.calculate();
        assert!(value >= 0.0 && value <= 1.0);
    }

    #[test]
    fn test_success_rate_has_highest_weight() {
        // Increase success_rate by 0.1 should have more impact than
        // increasing uptime by 0.1 (weight 0.25 vs 0.20).
        let base = TrustScore::new(0.5, 0.5, 500.0, 0.5, 0.5, 0.5);
        let more_uptime = TrustScore::new(0.6, 0.5, 500.0, 0.5, 0.5, 0.5);
        let more_success = TrustScore::new(0.5, 0.6, 500.0, 0.5, 0.5, 0.5);

        let base_val = base.calculate();
        let uptime_delta = more_uptime.calculate() - base_val;
        let success_delta = more_success.calculate() - base_val;

        assert!(success_delta > uptime_delta);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let score = TrustScore::new(0.95, 0.88, 150.0, 0.7, 0.5, 0.9);
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: TrustScore = serde_json::from_str(&json).unwrap();
        assert!((deserialized.uptime - score.uptime).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_is_zero() {
        let score = TrustScore::default();
        assert!((score.calculate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_very_high_latency() {
        // Latency above reference should still yield 0 for the latency component.
        let score = TrustScore::new(1.0, 1.0, 50_000.0, 1.0, 1.0, 1.0);
        let value = score.calculate();
        // Should be 1.0 minus the latency weight (since latency score = 0).
        let expected = 1.0 - WEIGHT_AVG_LATENCY;
        assert!((value - expected).abs() < 0.01);
    }
}
