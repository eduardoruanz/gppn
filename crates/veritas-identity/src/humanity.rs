use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::IdentityError;

/// Methods for verifying humanity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HumanityVerificationMethod {
    /// Social vouching by trusted peers.
    SocialVouching,
    /// Credential issued by a trusted authority.
    TrustedIssuer,
    /// Biometric liveness detection.
    BiometricLiveness,
    /// Cross-platform identity correlation.
    CrossPlatform,
}

impl std::fmt::Display for HumanityVerificationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocialVouching => write!(f, "SocialVouching"),
            Self::TrustedIssuer => write!(f, "TrustedIssuer"),
            Self::BiometricLiveness => write!(f, "BiometricLiveness"),
            Self::CrossPlatform => write!(f, "CrossPlatform"),
        }
    }
}

/// Status of a humanity verification for a DID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanityStatus {
    /// The DID being verified.
    pub did: String,
    /// Whether the DID is verified as human.
    pub verified: bool,
    /// The method(s) used for verification.
    pub verification_methods: Vec<HumanityVerificationMethod>,
    /// When the verification was performed.
    pub verified_at: Option<DateTime<Utc>>,
    /// When the verification expires.
    pub expires_at: Option<DateTime<Utc>>,
    /// Confidence score (0.0 - 1.0).
    pub confidence_score: f64,
}

impl HumanityStatus {
    /// Create a new unverified status.
    pub fn unverified(did: String) -> Self {
        Self {
            did,
            verified: false,
            verification_methods: Vec::new(),
            verified_at: None,
            expires_at: None,
            confidence_score: 0.0,
        }
    }

    /// Check if the verification has expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Check if currently valid (verified and not expired).
    pub fn is_valid(&self) -> bool {
        self.verified && !self.is_expired()
    }
}

/// Verifier for humanity proofs.
///
/// Combines multiple verification signals to produce a confidence score.
pub struct HumanityVerifier {
    /// Minimum confidence score to consider verified.
    min_confidence: f64,
    /// Weight for each verification method.
    method_weights: std::collections::HashMap<HumanityVerificationMethod, f64>,
}

impl HumanityVerifier {
    /// Create a new humanity verifier with default weights.
    pub fn new(min_confidence: f64) -> Self {
        let mut weights = std::collections::HashMap::new();
        weights.insert(HumanityVerificationMethod::SocialVouching, 0.25);
        weights.insert(HumanityVerificationMethod::TrustedIssuer, 0.35);
        weights.insert(HumanityVerificationMethod::BiometricLiveness, 0.25);
        weights.insert(HumanityVerificationMethod::CrossPlatform, 0.15);

        Self {
            min_confidence: min_confidence.clamp(0.0, 1.0),
            method_weights: weights,
        }
    }

    /// Evaluate humanity based on completed verification methods.
    ///
    /// Each method contributes its weight to the confidence score.
    /// The DID is considered verified if the total confidence exceeds min_confidence.
    pub fn evaluate(
        &self,
        did: &str,
        completed_methods: &[HumanityVerificationMethod],
        expiry: Option<DateTime<Utc>>,
    ) -> Result<HumanityStatus, IdentityError> {
        if completed_methods.is_empty() {
            return Ok(HumanityStatus::unverified(did.to_string()));
        }

        let confidence: f64 = completed_methods
            .iter()
            .filter_map(|method| self.method_weights.get(method))
            .sum();

        let confidence = confidence.clamp(0.0, 1.0);
        let verified = confidence >= self.min_confidence;

        Ok(HumanityStatus {
            did: did.to_string(),
            verified,
            verification_methods: completed_methods.to_vec(),
            verified_at: if verified { Some(Utc::now()) } else { None },
            expires_at: expiry,
            confidence_score: confidence,
        })
    }

    /// Get the minimum confidence threshold.
    pub fn min_confidence(&self) -> f64 {
        self.min_confidence
    }

    /// Set a custom weight for a verification method.
    pub fn set_weight(&mut self, method: HumanityVerificationMethod, weight: f64) {
        self.method_weights.insert(method, weight.clamp(0.0, 1.0));
    }
}

impl Default for HumanityVerifier {
    fn default() -> Self {
        Self::new(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_unverified_status() {
        let status = HumanityStatus::unverified("did:veritas:key:abc".into());
        assert!(!status.verified);
        assert!(!status.is_valid());
        assert_eq!(status.confidence_score, 0.0);
    }

    #[test]
    fn test_evaluate_single_method() {
        let verifier = HumanityVerifier::new(0.3);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[HumanityVerificationMethod::TrustedIssuer],
                None,
            )
            .unwrap();
        assert!(status.verified);
        assert!((status.confidence_score - 0.35).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evaluate_multiple_methods() {
        let verifier = HumanityVerifier::new(0.5);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[
                    HumanityVerificationMethod::SocialVouching,
                    HumanityVerificationMethod::TrustedIssuer,
                ],
                None,
            )
            .unwrap();
        assert!(status.verified);
        assert!((status.confidence_score - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evaluate_insufficient_confidence() {
        let verifier = HumanityVerifier::new(0.5);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[HumanityVerificationMethod::CrossPlatform],
                None,
            )
            .unwrap();
        assert!(!status.verified);
        assert!((status.confidence_score - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evaluate_all_methods() {
        let verifier = HumanityVerifier::new(0.5);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[
                    HumanityVerificationMethod::SocialVouching,
                    HumanityVerificationMethod::TrustedIssuer,
                    HumanityVerificationMethod::BiometricLiveness,
                    HumanityVerificationMethod::CrossPlatform,
                ],
                None,
            )
            .unwrap();
        assert!(status.verified);
        assert!((status.confidence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evaluate_empty_methods() {
        let verifier = HumanityVerifier::new(0.5);
        let status = verifier.evaluate("did:veritas:key:abc", &[], None).unwrap();
        assert!(!status.verified);
    }

    #[test]
    fn test_expired_status() {
        let verifier = HumanityVerifier::new(0.3);
        let past = Utc::now() - Duration::hours(1);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[HumanityVerificationMethod::TrustedIssuer],
                Some(past),
            )
            .unwrap();
        assert!(status.verified);
        assert!(status.is_expired());
        assert!(!status.is_valid());
    }

    #[test]
    fn test_valid_status_with_future_expiry() {
        let verifier = HumanityVerifier::new(0.3);
        let future = Utc::now() + Duration::hours(24);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[HumanityVerificationMethod::TrustedIssuer],
                Some(future),
            )
            .unwrap();
        assert!(status.is_valid());
    }

    #[test]
    fn test_custom_weight() {
        let mut verifier = HumanityVerifier::new(0.8);
        verifier.set_weight(HumanityVerificationMethod::SocialVouching, 0.9);
        let status = verifier
            .evaluate(
                "did:veritas:key:abc",
                &[HumanityVerificationMethod::SocialVouching],
                None,
            )
            .unwrap();
        assert!(status.verified);
        assert!((status.confidence_score - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_verifier() {
        let verifier = HumanityVerifier::default();
        assert!((verifier.min_confidence() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_verification_method_display() {
        assert_eq!(
            format!("{}", HumanityVerificationMethod::SocialVouching),
            "SocialVouching"
        );
        assert_eq!(
            format!("{}", HumanityVerificationMethod::BiometricLiveness),
            "BiometricLiveness"
        );
    }
}
