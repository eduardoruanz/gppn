use serde::{Deserialize, Serialize};

use crate::age_proof::AgeProof;
use crate::error::ProofError;
use crate::kyc_level_proof::KycLevelProof;
use crate::residency_proof::ResidencyProof;

/// A bundle of proofs demonstrating humanity across multiple dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanityProofBundle {
    /// Optional age proof.
    pub age_proof: Option<AgeProof>,
    /// Optional residency proof.
    pub residency_proof: Option<ResidencyProof>,
    /// Optional KYC level proof.
    pub kyc_level_proof: Option<KycLevelProof>,
    /// Number of social vouches received.
    pub social_vouch_count: u32,
    /// Overall confidence score (0.0 - 1.0).
    pub confidence_score: f64,
    /// When the bundle was created.
    pub generated_at: String,
}

impl HumanityProofBundle {
    /// Create a new empty bundle.
    pub fn new() -> Self {
        Self {
            age_proof: None,
            residency_proof: None,
            kyc_level_proof: None,
            social_vouch_count: 0,
            confidence_score: 0.0,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Set the age proof.
    pub fn with_age_proof(mut self, proof: AgeProof) -> Self {
        self.age_proof = Some(proof);
        self.recalculate_confidence();
        self
    }

    /// Set the residency proof.
    pub fn with_residency_proof(mut self, proof: ResidencyProof) -> Self {
        self.residency_proof = Some(proof);
        self.recalculate_confidence();
        self
    }

    /// Set the KYC level proof.
    pub fn with_kyc_level_proof(mut self, proof: KycLevelProof) -> Self {
        self.kyc_level_proof = Some(proof);
        self.recalculate_confidence();
        self
    }

    /// Set the social vouch count.
    pub fn with_social_vouches(mut self, count: u32) -> Self {
        self.social_vouch_count = count;
        self.recalculate_confidence();
        self
    }

    /// Recalculate the confidence score based on available proofs.
    fn recalculate_confidence(&mut self) {
        let mut score = 0.0;

        // Age proof: 0.20
        if self.age_proof.is_some() {
            score += 0.20;
        }

        // Residency proof: 0.20
        if self.residency_proof.is_some() {
            score += 0.20;
        }

        // KYC level proof: 0.30
        if self.kyc_level_proof.is_some() {
            score += 0.30;
        }

        // Social vouches: up to 0.30 (capped at 3 vouches)
        let vouch_score = (self.social_vouch_count.min(3) as f64) / 3.0 * 0.30;
        score += vouch_score;

        self.confidence_score = score.min(1.0);
    }

    /// Verify all proofs in the bundle.
    pub fn verify_all(&self) -> Result<bool, ProofError> {
        if let Some(ref proof) = self.age_proof {
            if !proof.verify()? {
                return Ok(false);
            }
        }

        if let Some(ref proof) = self.residency_proof {
            if !proof.verify()? {
                return Ok(false);
            }
        }

        if let Some(ref proof) = self.kyc_level_proof {
            if !proof.verify()? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Number of proofs in the bundle.
    pub fn proof_count(&self) -> usize {
        let mut count = 0;
        if self.age_proof.is_some() {
            count += 1;
        }
        if self.residency_proof.is_some() {
            count += 1;
        }
        if self.kyc_level_proof.is_some() {
            count += 1;
        }
        count
    }
}

impl Default for HumanityProofBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_empty_bundle() {
        let bundle = HumanityProofBundle::new();
        assert_eq!(bundle.proof_count(), 0);
        assert!((bundle.confidence_score - 0.0).abs() < f64::EPSILON);
        assert!(bundle.verify_all().unwrap());
    }

    #[test]
    fn test_bundle_with_age_proof() {
        let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let age_proof = AgeProof::create(dob, 18).unwrap();
        let bundle = HumanityProofBundle::new().with_age_proof(age_proof);
        assert_eq!(bundle.proof_count(), 1);
        assert!((bundle.confidence_score - 0.20).abs() < f64::EPSILON);
        assert!(bundle.verify_all().unwrap());
    }

    #[test]
    fn test_bundle_with_residency() {
        let residency = ResidencyProof::create("BR", &["AR", "BR", "PY", "UY"]).unwrap();
        let bundle = HumanityProofBundle::new().with_residency_proof(residency);
        assert_eq!(bundle.proof_count(), 1);
        assert!((bundle.confidence_score - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bundle_with_kyc() {
        let kyc = KycLevelProof::create(3, 2).unwrap();
        let bundle = HumanityProofBundle::new().with_kyc_level_proof(kyc);
        assert_eq!(bundle.proof_count(), 1);
        assert!((bundle.confidence_score - 0.30).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bundle_with_social_vouches() {
        let bundle = HumanityProofBundle::new().with_social_vouches(3);
        assert_eq!(bundle.proof_count(), 0);
        assert!((bundle.confidence_score - 0.30).abs() < f64::EPSILON);
    }

    #[test]
    fn test_full_bundle() {
        let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let age_proof = AgeProof::create(dob, 18).unwrap();
        let residency = ResidencyProof::create("BR", &["AR", "BR", "PY", "UY"]).unwrap();
        let kyc = KycLevelProof::create(3, 1).unwrap();

        let bundle = HumanityProofBundle::new()
            .with_age_proof(age_proof)
            .with_residency_proof(residency)
            .with_kyc_level_proof(kyc)
            .with_social_vouches(3);

        assert_eq!(bundle.proof_count(), 3);
        assert!((bundle.confidence_score - 1.0).abs() < f64::EPSILON);
        assert!(bundle.verify_all().unwrap());
    }

    #[test]
    fn test_social_vouches_capped() {
        let bundle = HumanityProofBundle::new().with_social_vouches(100);
        // Should be capped at 3 vouches worth
        assert!((bundle.confidence_score - 0.30).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bundle_serialization() {
        let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let age_proof = AgeProof::create(dob, 18).unwrap();
        let bundle = HumanityProofBundle::new().with_age_proof(age_proof);

        let json = serde_json::to_string(&bundle).unwrap();
        let back: HumanityProofBundle = serde_json::from_str(&json).unwrap();
        assert!(back.age_proof.is_some());
        assert_eq!(back.proof_count(), 1);
    }
}
