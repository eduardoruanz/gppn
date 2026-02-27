//! Integration test: Trust attestation and scoring across the identity layer.
//!
//! Tests TrustGraph, HumanityVerifier from veritas-identity,
//! combined with credential issuance from veritas-credentials.

use std::sync::Arc;

use veritas_credentials::{CredentialIssuer, CredentialVerifier, SchemaRegistry};
use veritas_crypto::KeyPair;
use veritas_identity::{HumanityVerificationMethod, HumanityVerifier, TrustGraph};

// =========================================================================
// Trust Graph — cross-crate integration
// =========================================================================

#[test]
fn test_trust_graph_build_from_credential_verifications() {
    let trust_graph = TrustGraph::new();

    let issuer_did = "did:veritas:key:issuer";
    let holder_did = "did:veritas:key:holder";
    let verifier_did = "did:veritas:key:verifier";

    // Simulate: Verifier successfully verified holder's credential → attest trust
    trust_graph.add_edge(verifier_did, holder_did, 0.9).unwrap();

    // Simulate: Holder trusts the issuer (they issued a valid credential)
    trust_graph.add_edge(holder_did, issuer_did, 0.95).unwrap();

    // Verify edges exist
    assert!(trust_graph.get_edge(verifier_did, holder_did).is_some());
    assert!(trust_graph.get_edge(holder_did, issuer_did).is_some());
    assert!(trust_graph.get_edge(issuer_did, verifier_did).is_none()); // No reverse edge
}

#[test]
fn test_trust_graph_outgoing_edges() {
    let trust_graph = TrustGraph::new();

    // A trusts B and C
    trust_graph
        .add_edge("did:veritas:key:a", "did:veritas:key:b", 0.9)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:a", "did:veritas:key:c", 0.8)
        .unwrap();

    let outgoing = trust_graph.outgoing_edges("did:veritas:key:a");
    assert_eq!(outgoing.len(), 2);

    // B should have no outgoing edges
    let outgoing_b = trust_graph.outgoing_edges("did:veritas:key:b");
    assert_eq!(outgoing_b.len(), 0);
}

#[test]
fn test_trust_graph_score_accumulation() {
    let trust_graph = TrustGraph::new();

    let target = "did:veritas:key:target";

    // Multiple peers attest trust in the target
    trust_graph
        .add_edge("did:veritas:key:peer1", target, 0.9)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:peer2", target, 0.85)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:peer3", target, 0.95)
        .unwrap();

    // Target should have 3 incoming trust edges
    let incoming = trust_graph.incoming_edges(target);
    assert_eq!(incoming.len(), 3);
}

#[test]
fn test_trust_graph_update_existing_edge() {
    let trust_graph = TrustGraph::new();

    let from = "did:veritas:key:verifier";
    let to = "did:veritas:key:holder";

    // Initial trust attestation
    trust_graph.add_edge(from, to, 0.7).unwrap();
    let edge = trust_graph.get_edge(from, to).unwrap();
    assert_eq!(edge.interactions, 1);

    // Update trust (same edge, higher score)
    trust_graph.add_edge(from, to, 0.95).unwrap();
    let edge = trust_graph.get_edge(from, to).unwrap();
    // Interaction count should increase
    assert_eq!(edge.interactions, 2);
    // Score should be updated
    assert!((edge.weight - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_trust_graph_compute_global_scores() {
    let trust_graph = TrustGraph::new();

    // A → B → C (chain of trust)
    trust_graph
        .add_edge("did:veritas:key:a", "did:veritas:key:b", 0.9)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:b", "did:veritas:key:c", 0.8)
        .unwrap();

    let scores = trust_graph.compute_scores(100, 1e-6);
    assert!(!scores.is_empty());

    // All DIDs should have positive scores
    for (_did, score) in &scores {
        assert!(*score > 0.0);
    }
}

#[test]
fn test_trust_graph_remove_edge() {
    let trust_graph = TrustGraph::new();

    let from = "did:veritas:key:a";
    let to = "did:veritas:key:b";

    trust_graph.add_edge(from, to, 0.8).unwrap();
    assert!(trust_graph.get_edge(from, to).is_some());

    let removed = trust_graph.remove_edge(from, to);
    assert!(removed.is_some());
    assert!(trust_graph.get_edge(from, to).is_none());
}

// =========================================================================
// Humanity Verification — integrated with trust
// =========================================================================

#[test]
fn test_humanity_verifier_social_vouching() {
    let verifier = HumanityVerifier::new(0.3);
    let subject_did = "did:veritas:key:subject";

    let status = verifier
        .evaluate(
            subject_did,
            &[HumanityVerificationMethod::SocialVouching],
            None,
        )
        .unwrap();

    assert_eq!(status.did, subject_did);
    // SocialVouching has weight 0.25, min_confidence 0.3 → not verified
    assert!(!status.verified);
    assert!((status.confidence_score - 0.25).abs() < f64::EPSILON);
}

#[test]
fn test_humanity_verifier_multiple_methods() {
    let verifier = HumanityVerifier::new(0.5);
    let subject_did = "did:veritas:key:subject";

    let status = verifier
        .evaluate(
            subject_did,
            &[
                HumanityVerificationMethod::SocialVouching,
                HumanityVerificationMethod::TrustedIssuer,
            ],
            None,
        )
        .unwrap();

    // SocialVouching (0.25) + TrustedIssuer (0.35) = 0.60 ≥ 0.5 → verified
    assert!(status.verified);
    assert!((status.confidence_score - 0.60).abs() < f64::EPSILON);
}

#[test]
fn test_humanity_verifier_all_methods_max_confidence() {
    let verifier = HumanityVerifier::new(0.5);
    let subject_did = "did:veritas:key:subject";

    let status = verifier
        .evaluate(
            subject_did,
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

// =========================================================================
// Combined: Credential issuance → verification → trust update
// =========================================================================

#[test]
fn test_credential_verification_updates_trust() {
    // Setup parties
    let issuer_kp = KeyPair::generate();
    let issuer_pubkey = issuer_kp.public_key();
    let registry = Arc::new(SchemaRegistry::new());
    let issuer = CredentialIssuer::new(
        "did:veritas:key:issuer".into(),
        issuer_kp,
        Arc::clone(&registry),
    );

    let verifier_cred = CredentialVerifier::new(Arc::clone(&registry));
    verifier_cred.add_trusted_issuer("did:veritas:key:issuer".into(), issuer_pubkey);

    let trust_graph = TrustGraph::new();
    let holder_did = "did:veritas:key:holder";
    let verifier_did = "did:veritas:key:verifier";

    // Step 1: Issue credential
    let vc = issuer
        .issue(
            holder_did,
            vec!["KycBasic".into()],
            serde_json::json!({
                "full_name": "Alice",
                "date_of_birth": "1990-01-01",
                "country": "BR"
            }),
        )
        .unwrap();

    // Step 2: Verify credential
    let result = verifier_cred.verify_credential(&vc).unwrap();
    assert!(result.valid);

    // Step 3: Successful verification → update trust graph
    if result.valid {
        trust_graph.add_edge(verifier_did, holder_did, 0.9).unwrap();
        trust_graph
            .add_edge(verifier_did, "did:veritas:key:issuer", 0.95)
            .unwrap();
    }

    // Verify trust was updated
    assert!(trust_graph.get_edge(verifier_did, holder_did).is_some());
    assert!(trust_graph
        .get_edge(verifier_did, "did:veritas:key:issuer")
        .is_some());

    let holder_incoming = trust_graph.incoming_edges(holder_did);
    assert_eq!(holder_incoming.len(), 1);
}

#[test]
fn test_failed_verification_does_not_update_trust() {
    // Setup: verifier does NOT trust the issuer
    let registry = Arc::new(SchemaRegistry::new());
    let verifier_cred = CredentialVerifier::new(Arc::clone(&registry));
    // Not adding any trusted issuers!

    let trust_graph = TrustGraph::new();
    let holder_did = "did:veritas:key:holder";
    let verifier_did = "did:veritas:key:verifier";

    // Issue credential with untrusted issuer
    let kp = KeyPair::generate();
    let issuer = CredentialIssuer::new(
        "did:veritas:key:bad-issuer".into(),
        kp,
        Arc::clone(&registry),
    );
    let vc = issuer
        .issue(holder_did, vec!["Test".into()], serde_json::json!({}))
        .unwrap();

    // Verify fails
    let result = verifier_cred.verify_credential(&vc).unwrap();
    assert!(!result.valid);

    // Failed verification → do NOT update trust
    if result.valid {
        trust_graph.add_edge(verifier_did, holder_did, 0.9).unwrap();
    }

    // Trust graph should remain empty
    assert!(trust_graph.get_edge(verifier_did, holder_did).is_none());
}

// =========================================================================
// DID management integration
// =========================================================================

#[test]
fn test_did_manager_with_credentials() {
    use veritas_identity::DidManager;

    let did_manager = DidManager::new();

    // Create DIDs using keypairs
    let issuer_kp = KeyPair::generate();
    let holder_kp = KeyPair::generate();

    let issuer_did = did_manager
        .create_did("key", &issuer_kp)
        .expect("create issuer DID");
    let holder_did = did_manager
        .create_did("key", &holder_kp)
        .expect("create holder DID");

    // Verify DIDs are resolvable
    assert!(did_manager.resolve_did(issuer_did.uri()).is_some());
    assert!(did_manager.resolve_did(holder_did.uri()).is_some());

    // Issue credential between registered DIDs
    let registry = Arc::new(SchemaRegistry::new());
    let issuer = CredentialIssuer::new(issuer_did.uri().to_string(), issuer_kp, registry);

    let vc = issuer
        .issue(
            holder_did.uri(),
            vec!["KycBasic".into()],
            serde_json::json!({"test": true}),
        )
        .unwrap();

    assert_eq!(vc.issuer, issuer_did.uri());
    assert_eq!(vc.subject, holder_did.uri());
    assert!(vc.is_signed());
}

#[test]
fn test_eigentrust_scores_after_multiple_verifications() {
    let trust_graph = TrustGraph::new();

    // Simulate a network of trust attestations from credential verifications
    // Issuer A issued credentials, verified by V1, V2, V3
    // Issuer B issued credentials, verified by V1
    // V1 is a highly trusted verifier

    trust_graph
        .add_edge("did:veritas:key:v1", "did:veritas:key:issuer-a", 0.95)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:v2", "did:veritas:key:issuer-a", 0.9)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:v3", "did:veritas:key:issuer-a", 0.85)
        .unwrap();
    trust_graph
        .add_edge("did:veritas:key:v1", "did:veritas:key:issuer-b", 0.7)
        .unwrap();

    // Compute global trust scores
    let scores = trust_graph.compute_scores(100, 1e-6);

    // Issuer A should have a higher score than Issuer B (more attestations)
    let score_a = scores
        .get("did:veritas:key:issuer-a")
        .copied()
        .unwrap_or(0.0);
    let score_b = scores
        .get("did:veritas:key:issuer-b")
        .copied()
        .unwrap_or(0.0);
    assert!(
        score_a > score_b,
        "issuer A ({}) should have higher trust than issuer B ({})",
        score_a,
        score_b
    );
}
