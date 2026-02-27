use crate::error::CryptoError;

/// A zero-knowledge proof blob.
#[derive(Debug, Clone)]
pub struct ZkProof {
    /// Serialized proof data.
    pub proof: Vec<u8>,
    /// Public inputs to the circuit.
    pub public_inputs: Vec<u8>,
    /// Identifier of the ZK circuit used.
    pub circuit_id: String,
}

/// Trait for ZK compliance proof generation and verification.
///
/// This is currently a stub interface. Real ZKP implementations
/// (e.g., using arkworks, bellman, or halo2) will be added
/// as settlement adapters mature.
pub trait ComplianceProver: Send + Sync {
    /// Generate a proof for a compliance statement.
    fn prove(&self, statement: &ComplianceStatement) -> Result<ZkProof, CryptoError>;

    /// Verify a proof against a compliance statement.
    fn verify(&self, proof: &ZkProof) -> Result<bool, CryptoError>;
}

/// A compliance statement to prove in zero knowledge.
#[derive(Debug, Clone)]
pub struct ComplianceStatement {
    /// Type of compliance claim.
    pub claim_type: String,
    /// Private witness data (not revealed).
    pub witness: Vec<u8>,
    /// Public statement parameters.
    pub public_params: Vec<u8>,
}

/// Stub implementation that always succeeds.
/// Replace with real ZKP backend in production.
pub struct StubProver;

impl ComplianceProver for StubProver {
    fn prove(&self, statement: &ComplianceStatement) -> Result<ZkProof, CryptoError> {
        tracing::warn!("using stub ZKP prover — not suitable for production");
        Ok(ZkProof {
            proof: vec![0u8; 32], // Fake proof
            public_inputs: statement.public_params.clone(),
            circuit_id: format!("stub-{}", statement.claim_type),
        })
    }

    fn verify(&self, _proof: &ZkProof) -> Result<bool, CryptoError> {
        tracing::warn!("using stub ZKP verifier — not suitable for production");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_prover() {
        let prover = StubProver;
        let statement = ComplianceStatement {
            claim_type: "kyc_verified".into(),
            witness: vec![1, 2, 3],
            public_params: vec![4, 5, 6],
        };

        let proof = prover.prove(&statement).unwrap();
        assert!(!proof.proof.is_empty());
        assert_eq!(proof.circuit_id, "stub-kyc_verified");

        let valid = prover.verify(&proof).unwrap();
        assert!(valid);
    }
}
