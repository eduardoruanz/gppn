use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use veritas_crypto::Blake3ProofGenerator;

use crate::error::ProofError;

/// Proves that a person's age is >= threshold without revealing their DOB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeProof {
    /// The minimum age that was proven.
    pub min_age: i64,
    /// The range proof from the crypto layer.
    pub range_proof: veritas_crypto::RangeProof,
    /// When the proof was generated (ISO 8601).
    pub generated_at: String,
}

impl AgeProof {
    /// Create an age proof from a date of birth.
    ///
    /// Computes the age from DOB, then generates a range proof showing age >= min_age.
    pub fn create(dob: NaiveDate, min_age: i64) -> Result<Self, ProofError> {
        let today = chrono::Utc::now().date_naive();
        let age = compute_age(dob, today);

        let (range_proof, _nonce) = Blake3ProofGenerator::prove_range(age, min_age, 150)
            .map_err(|e| ProofError::GenerationFailed(format!("age range proof failed: {}", e)))?;

        Ok(Self {
            min_age,
            range_proof,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Verify that the age proof is valid.
    pub fn verify(&self) -> Result<bool, ProofError> {
        Blake3ProofGenerator::verify_range(&self.range_proof)
            .map_err(|e| ProofError::VerificationFailed(e.to_string()))
    }
}

/// Compute age in years from DOB and today's date.
fn compute_age(dob: NaiveDate, today: NaiveDate) -> i64 {
    let mut age = today.year() as i64 - dob.year() as i64;
    if today.month() < dob.month() || (today.month() == dob.month() && today.day() < dob.day()) {
        age -= 1;
    }
    age
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_age_proof_adult() {
        let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let proof = AgeProof::create(dob, 18).unwrap();
        assert_eq!(proof.min_age, 18);
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_age_proof_exact_threshold() {
        // Someone who is exactly 21 proving >= 21
        let today = chrono::Utc::now().date_naive();
        let dob = NaiveDate::from_ymd_opt(
            today.year() - 21,
            today.month(),
            today.day().min(28), // safe day
        )
        .unwrap();
        let proof = AgeProof::create(dob, 21).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_age_proof_too_young() {
        let today = chrono::Utc::now().date_naive();
        let dob = NaiveDate::from_ymd_opt(today.year() - 15, 1, 1).unwrap();
        let result = AgeProof::create(dob, 18);
        assert!(result.is_err());
    }

    #[test]
    fn test_age_proof_elderly() {
        let dob = NaiveDate::from_ymd_opt(1940, 3, 20).unwrap();
        let proof = AgeProof::create(dob, 65).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_compute_age() {
        let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_eq!(compute_age(dob, today), 34);

        let today2 = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        assert_eq!(compute_age(dob, today2), 35);

        let today3 = NaiveDate::from_ymd_opt(2025, 6, 14).unwrap();
        assert_eq!(compute_age(dob, today3), 34);
    }

    #[test]
    fn test_age_proof_serialization() {
        let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let proof = AgeProof::create(dob, 18).unwrap();
        let json = serde_json::to_string(&proof).unwrap();
        let back: AgeProof = serde_json::from_str(&json).unwrap();
        assert_eq!(back.min_age, 18);
    }
}
