use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use veritas_core::types::SchemaId;

use crate::error::CredentialError;

/// Definition of a claim within a credential schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDefinition {
    /// Claim name.
    pub name: String,
    /// Expected value type (e.g., "string", "integer", "date", "boolean").
    pub value_type: String,
    /// Whether this claim is required.
    pub required: bool,
    /// Optional description.
    pub description: Option<String>,
}

/// A credential schema defining the structure of a verifiable credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSchema {
    /// Unique schema identifier.
    pub id: SchemaId,
    /// Human-readable name.
    pub name: String,
    /// Schema version.
    pub version: String,
    /// Claim definitions.
    pub claims: Vec<ClaimDefinition>,
    /// Description of the schema.
    pub description: String,
}

/// Registry of credential schemas.
pub struct SchemaRegistry {
    schemas: DashMap<String, CredentialSchema>,
}

impl SchemaRegistry {
    /// Create a new registry with built-in schemas.
    pub fn new() -> Self {
        let registry = Self {
            schemas: DashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    /// Register built-in credential schemas.
    fn register_builtins(&self) {
        self.schemas.insert(
            "kyc-basic-v1".into(),
            CredentialSchema {
                id: SchemaId::new("kyc-basic-v1"),
                name: "Basic KYC".into(),
                version: "1.0.0".into(),
                description: "Basic KYC verification with name, DOB, and country".into(),
                claims: vec![
                    ClaimDefinition {
                        name: "full_name".into(),
                        value_type: "string".into(),
                        required: true,
                        description: Some("Full legal name".into()),
                    },
                    ClaimDefinition {
                        name: "date_of_birth".into(),
                        value_type: "date".into(),
                        required: true,
                        description: Some("Date of birth (ISO 8601)".into()),
                    },
                    ClaimDefinition {
                        name: "country".into(),
                        value_type: "string".into(),
                        required: true,
                        description: Some("Country of citizenship (ISO 3166-1 alpha-2)".into()),
                    },
                ],
            },
        );

        self.schemas.insert(
            "age-verification-v1".into(),
            CredentialSchema {
                id: SchemaId::new("age-verification-v1"),
                name: "Age Verification".into(),
                version: "1.0.0".into(),
                description: "Proves age without revealing date of birth".into(),
                claims: vec![
                    ClaimDefinition {
                        name: "date_of_birth".into(),
                        value_type: "date".into(),
                        required: true,
                        description: Some("Date of birth (for commitment)".into()),
                    },
                    ClaimDefinition {
                        name: "min_age_verified".into(),
                        value_type: "integer".into(),
                        required: true,
                        description: Some("Minimum age that was verified".into()),
                    },
                ],
            },
        );

        self.schemas.insert(
            "residency-v1".into(),
            CredentialSchema {
                id: SchemaId::new("residency-v1"),
                name: "Residency Proof".into(),
                version: "1.0.0".into(),
                description: "Proves country of residence".into(),
                claims: vec![
                    ClaimDefinition {
                        name: "country".into(),
                        value_type: "string".into(),
                        required: true,
                        description: Some("Country of residence".into()),
                    },
                    ClaimDefinition {
                        name: "region".into(),
                        value_type: "string".into(),
                        required: false,
                        description: Some("State/region of residence".into()),
                    },
                ],
            },
        );

        self.schemas.insert(
            "humanity-proof-v1".into(),
            CredentialSchema {
                id: SchemaId::new("humanity-proof-v1"),
                name: "Humanity Proof".into(),
                version: "1.0.0".into(),
                description: "AI-resistant proof of humanity".into(),
                claims: vec![
                    ClaimDefinition {
                        name: "verification_methods".into(),
                        value_type: "string".into(),
                        required: true,
                        description: Some("Comma-separated verification methods used".into()),
                    },
                    ClaimDefinition {
                        name: "confidence_score".into(),
                        value_type: "integer".into(),
                        required: true,
                        description: Some("Confidence score (0-100)".into()),
                    },
                ],
            },
        );
    }

    /// Register a custom schema.
    pub fn register(&self, schema: CredentialSchema) -> Result<(), CredentialError> {
        if schema.claims.is_empty() {
            return Err(CredentialError::InvalidSchema(
                "schema must have at least one claim".into(),
            ));
        }
        self.schemas.insert(schema.id.as_str().to_string(), schema);
        Ok(())
    }

    /// Get a schema by ID.
    pub fn get(&self, id: &str) -> Option<CredentialSchema> {
        self.schemas.get(id).map(|entry| entry.clone())
    }

    /// List all schema IDs.
    pub fn list(&self) -> Vec<String> {
        self.schemas.iter().map(|e| e.key().clone()).collect()
    }

    /// Number of registered schemas.
    pub fn count(&self) -> usize {
        self.schemas.len()
    }

    /// Validate claims against a schema.
    pub fn validate_claims(
        &self,
        schema_id: &str,
        claims: &serde_json::Value,
    ) -> Result<(), CredentialError> {
        let schema = self
            .get(schema_id)
            .ok_or_else(|| CredentialError::SchemaNotFound(schema_id.to_string()))?;

        let claims_map = claims
            .as_object()
            .ok_or_else(|| CredentialError::InvalidSchema("claims must be a JSON object".into()))?;

        for claim_def in &schema.claims {
            if claim_def.required && !claims_map.contains_key(&claim_def.name) {
                return Err(CredentialError::InvalidSchema(format!(
                    "missing required claim: {}",
                    claim_def.name
                )));
            }
        }

        Ok(())
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_schemas() {
        let registry = SchemaRegistry::new();
        assert!(registry.get("kyc-basic-v1").is_some());
        assert!(registry.get("age-verification-v1").is_some());
        assert!(registry.get("residency-v1").is_some());
        assert!(registry.get("humanity-proof-v1").is_some());
        assert_eq!(registry.count(), 4);
    }

    #[test]
    fn test_register_custom_schema() {
        let registry = SchemaRegistry::new();
        let schema = CredentialSchema {
            id: SchemaId::new("custom-v1"),
            name: "Custom".into(),
            version: "1.0.0".into(),
            description: "A custom schema".into(),
            claims: vec![ClaimDefinition {
                name: "field1".into(),
                value_type: "string".into(),
                required: true,
                description: None,
            }],
        };
        registry.register(schema).unwrap();
        assert!(registry.get("custom-v1").is_some());
        assert_eq!(registry.count(), 5);
    }

    #[test]
    fn test_register_empty_claims_fails() {
        let registry = SchemaRegistry::new();
        let schema = CredentialSchema {
            id: SchemaId::new("empty-v1"),
            name: "Empty".into(),
            version: "1.0.0".into(),
            description: "No claims".into(),
            claims: vec![],
        };
        assert!(registry.register(schema).is_err());
    }

    #[test]
    fn test_validate_claims_valid() {
        let registry = SchemaRegistry::new();
        let claims = serde_json::json!({
            "full_name": "Alice",
            "date_of_birth": "1990-01-15",
            "country": "BR"
        });
        assert!(registry.validate_claims("kyc-basic-v1", &claims).is_ok());
    }

    #[test]
    fn test_validate_claims_missing_required() {
        let registry = SchemaRegistry::new();
        let claims = serde_json::json!({
            "full_name": "Alice"
        });
        assert!(registry.validate_claims("kyc-basic-v1", &claims).is_err());
    }

    #[test]
    fn test_validate_claims_unknown_schema() {
        let registry = SchemaRegistry::new();
        let claims = serde_json::json!({});
        assert!(registry.validate_claims("nonexistent", &claims).is_err());
    }

    #[test]
    fn test_list_schemas() {
        let registry = SchemaRegistry::new();
        let ids = registry.list();
        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn test_schema_details() {
        let registry = SchemaRegistry::new();
        let schema = registry.get("kyc-basic-v1").unwrap();
        assert_eq!(schema.name, "Basic KYC");
        assert_eq!(schema.claims.len(), 3);
        assert!(schema.claims.iter().any(|c| c.name == "full_name"));
    }
}
