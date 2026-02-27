//! Integration test: Full credential lifecycle across crates.
//!
//! Tests the issuer → holder → verifier flow using veritas-credentials,
//! veritas-identity, and veritas-crypto together.

use std::sync::Arc;

use chrono::Duration;
use veritas_credentials::{CredentialIssuer, CredentialVerifier, CredentialWallet, SchemaRegistry};
use veritas_crypto::KeyPair;
use veritas_identity::VerifiableCredential;

/// Helper: create an issuer with a fresh keypair and schema registry.
/// Returns the issuer, the public key (for verification), and the registry.
fn create_issuer(
    did: &str,
) -> (
    CredentialIssuer,
    veritas_crypto::PublicKey,
    Arc<SchemaRegistry>,
) {
    let kp = KeyPair::generate();
    let pubkey = kp.public_key();
    let registry = Arc::new(SchemaRegistry::new());
    let issuer = CredentialIssuer::new(did.to_string(), kp, Arc::clone(&registry));
    (issuer, pubkey, registry)
}

// =========================================================================
// Two-party credential flow: Issuer → Holder
// =========================================================================

#[test]
fn test_issue_store_and_retrieve() {
    let (issuer, _kp, _reg) = create_issuer("did:veritas:key:issuer-a");
    let holder_did = "did:veritas:key:holder-b";
    let wallet = CredentialWallet::new(holder_did.to_string());

    // Issuer issues credential
    let vc = issuer
        .issue(
            holder_did,
            vec!["VerifiableCredential".into(), "KycBasic".into()],
            serde_json::json!({
                "full_name": "Alice Santos",
                "date_of_birth": "1995-03-15",
                "country": "BR"
            }),
        )
        .expect("issuance should succeed");

    assert!(vc.is_signed());
    assert_eq!(vc.issuer, "did:veritas:key:issuer-a");
    assert_eq!(vc.subject, holder_did);

    // Holder stores credential
    wallet.store(vc.clone()).expect("store should succeed");
    assert_eq!(wallet.count(), 1);

    // Holder retrieves credential
    let retrieved = wallet.get(&vc.id).expect("should find credential");
    assert_eq!(retrieved.id, vc.id);
    assert_eq!(retrieved.issuer, vc.issuer);
}

#[test]
fn test_issue_multiple_credentials_to_wallet() {
    let (issuer, _kp, _reg) = create_issuer("did:veritas:key:issuer");
    let holder_did = "did:veritas:key:holder";
    let wallet = CredentialWallet::new(holder_did.to_string());

    // Issue KYC credential
    let vc1 = issuer
        .issue(
            holder_did,
            vec!["KycBasic".into()],
            serde_json::json!({"full_name": "Alice", "date_of_birth": "1995-01-01", "country": "BR"}),
        )
        .unwrap();
    wallet.store(vc1).unwrap();

    // Issue Age Verification credential
    let vc2 = issuer
        .issue(
            holder_did,
            vec!["AgeVerification".into()],
            serde_json::json!({"over_18": true}),
        )
        .unwrap();
    wallet.store(vc2).unwrap();

    // Issue Residency credential
    let vc3 = issuer
        .issue(
            holder_did,
            vec!["Residency".into()],
            serde_json::json!({"country": "BR", "state": "SP"}),
        )
        .unwrap();
    wallet.store(vc3).unwrap();

    assert_eq!(wallet.count(), 3);
    assert_eq!(wallet.list_by_type("KycBasic").len(), 1);
    assert_eq!(wallet.list_by_type("AgeVerification").len(), 1);
    assert_eq!(wallet.list_by_type("Residency").len(), 1);
}

// =========================================================================
// Three-party flow: Issuer → Holder → Verifier
// =========================================================================

#[test]
fn test_full_issuance_and_verification_flow() {
    // Setup: Issuer with keypair, Verifier that trusts the issuer
    let (issuer, issuer_pubkey, registry) = create_issuer("did:veritas:key:issuer");
    let verifier = CredentialVerifier::new(Arc::clone(&registry));
    verifier.add_trusted_issuer("did:veritas:key:issuer".into(), issuer_pubkey);

    let holder_did = "did:veritas:key:holder";
    let wallet = CredentialWallet::new(holder_did.to_string());

    // Step 1: Issuer issues credential to holder
    let vc = issuer
        .issue(
            holder_did,
            vec!["VerifiableCredential".into(), "KycBasic".into()],
            serde_json::json!({
                "full_name": "Bob Silva",
                "date_of_birth": "1990-07-20",
                "country": "BR",
                "kyc_level": 3
            }),
        )
        .unwrap();

    // Step 2: Holder stores credential
    wallet.store(vc.clone()).unwrap();

    // Step 3: Holder presents credential to verifier
    let credential = wallet.get(&vc.id).unwrap();

    // Step 4: Verifier verifies the credential
    let result = verifier.verify_credential(&credential).unwrap();
    assert!(result.valid, "credential should be valid");
    assert!(
        result.checks.iter().all(|c| c.passed),
        "all checks should pass"
    );
}

#[test]
fn test_verification_fails_for_untrusted_issuer() {
    let (issuer, _kp, registry) = create_issuer("did:veritas:key:untrusted-issuer");
    let verifier = CredentialVerifier::new(Arc::clone(&registry));
    // Note: NOT adding the issuer to trusted list

    let holder_did = "did:veritas:key:holder";
    let vc = issuer
        .issue(holder_did, vec!["Test".into()], serde_json::json!({}))
        .unwrap();

    let result = verifier.verify_credential(&vc).unwrap();
    assert!(
        !result.valid,
        "credential from untrusted issuer should fail"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "issuer_trusted" && !c.passed),
        "issuer_trusted check should fail"
    );
}

#[test]
fn test_verification_fails_for_expired_credential() {
    let (issuer, issuer_pubkey, registry) = create_issuer("did:veritas:key:issuer");
    let verifier = CredentialVerifier::new(Arc::clone(&registry));
    verifier.add_trusted_issuer("did:veritas:key:issuer".into(), issuer_pubkey);

    let holder_did = "did:veritas:key:holder";
    let vc = issuer
        .issue_with_expiry(
            holder_did,
            vec!["Test".into()],
            serde_json::json!({}),
            Duration::hours(-1), // Already expired
        )
        .unwrap();

    let result = verifier.verify_credential(&vc).unwrap();
    assert!(!result.valid);
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "not_expired" && !c.passed),
        "expiration check should fail"
    );
}

#[test]
fn test_verification_fails_for_tampered_signature() {
    let (issuer, _issuer_pubkey, registry) = create_issuer("did:veritas:key:issuer");
    let verifier = CredentialVerifier::new(Arc::clone(&registry));

    // Trust a DIFFERENT key than the one that signed
    let wrong_kp = KeyPair::generate();
    verifier.add_trusted_issuer("did:veritas:key:issuer".into(), wrong_kp.public_key());

    let holder_did = "did:veritas:key:holder";
    let vc = issuer
        .issue(holder_did, vec!["Test".into()], serde_json::json!({}))
        .unwrap();

    let result = verifier.verify_credential(&vc).unwrap();
    assert!(!result.valid);
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "signature_valid" && !c.passed),
        "signature check should fail with wrong key"
    );
}

// =========================================================================
// Schema-validated credentials
// =========================================================================

#[test]
fn test_issue_with_schema_validation() {
    let (issuer, issuer_pubkey, registry) = create_issuer("did:veritas:key:issuer");
    let verifier = CredentialVerifier::new(Arc::clone(&registry));
    verifier.add_trusted_issuer("did:veritas:key:issuer".into(), issuer_pubkey);

    let holder_did = "did:veritas:key:holder";

    // Issue with schema validation (kyc-basic-v1 requires full_name, date_of_birth, country)
    let vc = issuer
        .issue_with_schema(
            holder_did,
            "kyc-basic-v1",
            vec!["KycBasic".into()],
            serde_json::json!({
                "full_name": "Carlos Mendes",
                "date_of_birth": "1988-11-03",
                "country": "BR"
            }),
        )
        .unwrap();

    let result = verifier.verify_credential(&vc).unwrap();
    assert!(result.valid);
}

#[test]
fn test_schema_validation_rejects_incomplete_claims() {
    let (issuer, _kp, _reg) = create_issuer("did:veritas:key:issuer");
    let holder_did = "did:veritas:key:holder";

    // kyc-basic-v1 requires full_name, date_of_birth, country — only providing full_name
    let result = issuer.issue_with_schema(
        holder_did,
        "kyc-basic-v1",
        vec!["KycBasic".into()],
        serde_json::json!({"full_name": "Alice"}),
    );

    assert!(
        result.is_err(),
        "schema validation should reject incomplete claims"
    );
}

// =========================================================================
// Credential serialization round-trip
// =========================================================================

#[test]
fn test_credential_json_roundtrip() {
    let (issuer, issuer_pubkey, registry) = create_issuer("did:veritas:key:issuer");
    let verifier = CredentialVerifier::new(Arc::clone(&registry));
    verifier.add_trusted_issuer("did:veritas:key:issuer".into(), issuer_pubkey);

    let vc = issuer
        .issue(
            "did:veritas:key:holder",
            vec!["KycBasic".into()],
            serde_json::json!({"name": "Alice"}),
        )
        .unwrap();

    // Serialize to JSON
    let json_str = serde_json::to_string(&vc).expect("serialize should work");

    // Deserialize back
    let deserialized: VerifiableCredential =
        serde_json::from_str(&json_str).expect("deserialize should work");

    // Verify deserialized credential
    assert_eq!(deserialized.id, vc.id);
    assert_eq!(deserialized.issuer, vc.issuer);
    assert_eq!(deserialized.subject, vc.subject);
    assert!(deserialized.is_signed());

    let result = verifier.verify_credential(&deserialized).unwrap();
    assert!(result.valid, "deserialized credential should verify");
}

// =========================================================================
// Wallet operations
// =========================================================================

#[test]
fn test_wallet_rejects_credential_for_wrong_subject() {
    let (issuer, _kp, _reg) = create_issuer("did:veritas:key:issuer");
    let wallet = CredentialWallet::new("did:veritas:key:alice".to_string());

    let vc = issuer
        .issue(
            "did:veritas:key:bob",
            vec!["Test".into()],
            serde_json::json!({}),
        )
        .unwrap();

    let result = wallet.store(vc);
    assert!(
        result.is_err(),
        "wallet should reject credential for wrong subject"
    );
}

#[test]
fn test_wallet_active_credentials_filters_expired() {
    let (issuer, _kp, _reg) = create_issuer("did:veritas:key:issuer");
    let holder_did = "did:veritas:key:holder";
    let wallet = CredentialWallet::new(holder_did.to_string());

    // Active credential
    let vc_active = issuer
        .issue_with_expiry(
            holder_did,
            vec!["Active".into()],
            serde_json::json!({}),
            Duration::hours(24),
        )
        .unwrap();
    wallet.store(vc_active).unwrap();

    // Expired credential
    let vc_expired = issuer
        .issue_with_expiry(
            holder_did,
            vec!["Expired".into()],
            serde_json::json!({}),
            Duration::hours(-1),
        )
        .unwrap();
    wallet.store(vc_expired).unwrap();

    assert_eq!(wallet.count(), 2);
    assert_eq!(wallet.active_credentials().len(), 1);
}
