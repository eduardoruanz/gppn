//! Integration test: Zero-knowledge proof generation and verification.
//!
//! Tests age, residency, and KYC level proofs end-to-end using veritas-proof
//! and veritas-crypto together.

use chrono::{Datelike, NaiveDate};
use veritas_proof::{AgeProof, HumanityProofBundle, KycLevelProof, ResidencyProof};

// =========================================================================
// Age Proofs
// =========================================================================

#[test]
fn test_age_proof_adult() {
    let dob = NaiveDate::from_ymd_opt(1995, 3, 15).unwrap();
    let proof = AgeProof::create(dob, 18).expect("should create age proof");
    assert!(proof.verify().unwrap(), "adult should pass age >= 18 proof");
}

#[test]
fn test_age_proof_exact_threshold() {
    let today = chrono::Utc::now().date_naive();
    let dob =
        NaiveDate::from_ymd_opt(today.year() - 21, today.month(), today.day().min(28)).unwrap();
    let proof = AgeProof::create(dob, 21).expect("should create proof");
    assert!(
        proof.verify().unwrap(),
        "person exactly at threshold should pass"
    );
}

#[test]
fn test_age_proof_minor_fails() {
    let today = chrono::Utc::now().date_naive();
    let dob = NaiveDate::from_ymd_opt(today.year() - 15, 1, 1).unwrap();
    let result = AgeProof::create(dob, 18);
    // Proof creation should fail for a minor
    assert!(
        result.is_err(),
        "minor should not be able to create age >= 18 proof"
    );
}

#[test]
fn test_age_proof_senior() {
    let dob = NaiveDate::from_ymd_opt(1950, 1, 1).unwrap();
    let proof = AgeProof::create(dob, 65).expect("should create proof");
    assert!(
        proof.verify().unwrap(),
        "senior should pass age >= 65 proof"
    );
}

#[test]
fn test_age_proof_serialization_roundtrip() {
    let dob = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
    let proof = AgeProof::create(dob, 18).unwrap();

    let json = serde_json::to_string(&proof).expect("serialize");
    let deserialized: AgeProof = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.min_age, proof.min_age);
}

// =========================================================================
// Residency Proofs
// =========================================================================

#[test]
fn test_residency_proof_valid_country() {
    let allowed = ["BR", "AR", "CL", "CO", "PE"];
    let proof = ResidencyProof::create("BR", &allowed).expect("should create proof");
    assert!(proof.verify().unwrap(), "BR should be in allowed set");
}

#[test]
fn test_residency_proof_invalid_country() {
    let allowed = ["BR", "AR", "CL"];
    let result = ResidencyProof::create("US", &allowed);
    // Creation should fail for non-member
    assert!(result.is_err(), "US should not be provable in [BR, AR, CL]");
}

#[test]
fn test_residency_proof_single_country() {
    let allowed = ["BR"];
    let proof = ResidencyProof::create("BR", &allowed).expect("should create proof");
    assert!(proof.verify().unwrap());
}

#[test]
fn test_residency_proof_large_set() {
    let allowed = [
        "BR", "AR", "CL", "CO", "PE", "UY", "PY", "EC", "VE", "BO", "US", "CA", "MX", "GB", "FR",
        "DE", "IT", "ES", "PT", "NL",
    ];
    let proof = ResidencyProof::create("PT", &allowed).expect("should create proof");
    assert!(proof.verify().unwrap(), "PT should be in large allowed set");
}

#[test]
fn test_residency_proof_serialization_roundtrip() {
    let allowed = ["BR", "AR"];
    let proof = ResidencyProof::create("BR", &allowed).unwrap();

    let json = serde_json::to_string(&proof).expect("serialize");
    let _deserialized: ResidencyProof = serde_json::from_str(&json).expect("deserialize");
}

// =========================================================================
// KYC Level Proofs
// =========================================================================

#[test]
fn test_kyc_level_proof_sufficient() {
    let proof = KycLevelProof::create(3, 2).expect("should create proof");
    assert!(
        proof.verify().unwrap(),
        "level 3 should satisfy min level 2"
    );
}

#[test]
fn test_kyc_level_proof_exact_match() {
    let proof = KycLevelProof::create(2, 2).expect("should create proof");
    assert!(
        proof.verify().unwrap(),
        "level 2 should satisfy min level 2"
    );
}

#[test]
fn test_kyc_level_proof_insufficient() {
    let result = KycLevelProof::create(1, 3);
    assert!(result.is_err(), "level 1 should not satisfy min level 3");
}

#[test]
fn test_kyc_level_proof_max_level() {
    let proof = KycLevelProof::create(3, 1).expect("should create proof");
    assert!(proof.verify().unwrap(), "max level should satisfy any min");
}

#[test]
fn test_kyc_level_proof_serialization_roundtrip() {
    let proof = KycLevelProof::create(3, 2).unwrap();
    let json = serde_json::to_string(&proof).expect("serialize");
    let _deserialized: KycLevelProof = serde_json::from_str(&json).expect("deserialize");
}

// =========================================================================
// Humanity Proof Bundle
// =========================================================================

#[test]
fn test_humanity_proof_bundle_creation() {
    let dob = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
    let age_proof = AgeProof::create(dob, 18).unwrap();
    let residency_proof = ResidencyProof::create("BR", &["BR", "AR"]).unwrap();
    let kyc_proof = KycLevelProof::create(3, 2).unwrap();

    let bundle = HumanityProofBundle::new()
        .with_age_proof(age_proof)
        .with_residency_proof(residency_proof)
        .with_kyc_level_proof(kyc_proof);

    assert_eq!(bundle.proof_count(), 3);
    assert!(
        bundle.verify_all().unwrap(),
        "bundle with all valid proofs should verify"
    );
}

#[test]
fn test_humanity_proof_bundle_confidence_score() {
    let dob = NaiveDate::from_ymd_opt(1985, 6, 15).unwrap();
    let age_proof = AgeProof::create(dob, 18).unwrap();
    let residency_proof = ResidencyProof::create("BR", &["BR"]).unwrap();
    let kyc_proof = KycLevelProof::create(3, 1).unwrap();

    let bundle = HumanityProofBundle::new()
        .with_age_proof(age_proof)
        .with_residency_proof(residency_proof)
        .with_kyc_level_proof(kyc_proof);

    // age(0.20) + residency(0.20) + kyc(0.30) = 0.70
    assert!(
        (bundle.confidence_score - 0.70).abs() < f64::EPSILON,
        "confidence score should be 0.70, got {}",
        bundle.confidence_score
    );
}

#[test]
fn test_humanity_proof_bundle_full_confidence() {
    let dob = NaiveDate::from_ymd_opt(1985, 6, 15).unwrap();
    let age_proof = AgeProof::create(dob, 18).unwrap();
    let residency_proof = ResidencyProof::create("BR", &["BR"]).unwrap();
    let kyc_proof = KycLevelProof::create(3, 1).unwrap();

    let bundle = HumanityProofBundle::new()
        .with_age_proof(age_proof)
        .with_residency_proof(residency_proof)
        .with_kyc_level_proof(kyc_proof)
        .with_social_vouches(3);

    // age(0.20) + residency(0.20) + kyc(0.30) + vouches(0.30) = 1.0
    assert!(
        (bundle.confidence_score - 1.0).abs() < f64::EPSILON,
        "full bundle should have 1.0 confidence, got {}",
        bundle.confidence_score
    );
    assert!(bundle.verify_all().unwrap());
}

// =========================================================================
// Combined proof + credential flow
// =========================================================================

#[test]
fn test_proof_generation_from_credential_claims() {
    use std::sync::Arc;
    use veritas_credentials::{CredentialIssuer, SchemaRegistry};
    use veritas_crypto::KeyPair;

    let kp = KeyPair::generate();
    let registry = Arc::new(SchemaRegistry::new());
    let issuer = CredentialIssuer::new("did:veritas:key:issuer".into(), kp, registry);

    // Issue credential with rich claims
    let vc = issuer
        .issue(
            "did:veritas:key:holder",
            vec!["KycBasic".into()],
            serde_json::json!({
                "full_name": "Alice Santos",
                "date_of_birth": "1995-03-15",
                "country": "BR",
                "kyc_level": 3
            }),
        )
        .unwrap();

    // Extract claims and generate proofs
    let dob_str = vc.claims["date_of_birth"].as_str().unwrap();
    let dob = NaiveDate::parse_from_str(dob_str, "%Y-%m-%d").unwrap();
    let country = vc.claims["country"].as_str().unwrap();
    let kyc_level = vc.claims["kyc_level"].as_i64().unwrap();

    // Generate age proof from DOB claim
    let age_proof = AgeProof::create(dob, 18).unwrap();
    assert!(age_proof.verify().unwrap());

    // Generate residency proof from country claim
    let residency_proof = ResidencyProof::create(country, &["BR", "AR", "CL", "CO", "PE"]).unwrap();
    assert!(residency_proof.verify().unwrap());

    // Generate KYC level proof from kyc_level claim
    let kyc_proof = KycLevelProof::create(kyc_level, 2).unwrap();
    assert!(kyc_proof.verify().unwrap());

    // Bundle all proofs
    let bundle = HumanityProofBundle::new()
        .with_age_proof(age_proof)
        .with_residency_proof(residency_proof)
        .with_kyc_level_proof(kyc_proof);

    assert!(bundle.verify_all().unwrap());
    assert!(bundle.confidence_score > 0.0);
}
