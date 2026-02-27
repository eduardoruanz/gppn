use prost::Message;

use crate::error::CoreError;
use crate::state_machine::PaymentState;
use crate::types::{Amount, Condition, ConditionType, Did, RoutingHint, SettlementHint};

/// A GPPN Payment Message — the fundamental unit of the protocol.
///
/// This wraps the protobuf-generated PaymentMessage with domain logic,
/// validation, and convenience methods.
#[derive(Debug, Clone)]
pub struct PaymentMessage {
    /// Unique identifier (UUID v7, timestamp-based).
    pub pm_id: uuid::Uuid,
    /// Protocol version.
    pub version: u8,
    /// Sender's DID.
    pub sender_did: Did,
    /// Receiver's DID.
    pub receiver_did: Did,
    /// Payment amount.
    pub amount: Amount,
    /// Preferred settlement mechanisms, ordered by priority.
    pub settlement_preferences: Vec<SettlementHint>,
    /// Conditions attached to the payment.
    pub conditions: Vec<Condition>,
    /// Encrypted metadata (opaque to relays).
    pub metadata: Vec<u8>,
    /// Time-to-live in seconds.
    pub ttl: u32,
    /// Creation timestamp (Unix milliseconds).
    pub timestamp: u64,
    /// Ed25519 signature over the signing payload.
    pub signature: Vec<u8>,
    /// Routing hints for path discovery.
    pub routing_hints: Vec<RoutingHint>,
    /// Current state of the payment.
    pub state: PaymentState,
}

impl PaymentMessage {
    /// Create a new PaymentMessageBuilder.
    pub fn builder() -> PaymentMessageBuilder {
        PaymentMessageBuilder::default()
    }

    /// Validate the payment message.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.sender_did.uri().is_empty() {
            return Err(CoreError::MissingField("sender_did".into()));
        }
        if self.receiver_did.uri().is_empty() {
            return Err(CoreError::MissingField("receiver_did".into()));
        }
        if self.sender_did == self.receiver_did {
            return Err(CoreError::ValidationError(
                "sender and receiver must be different".into(),
            ));
        }
        if self.amount.is_zero() {
            return Err(CoreError::InvalidAmount("amount must be greater than zero".into()));
        }
        if self.ttl == 0 {
            return Err(CoreError::ValidationError("TTL must be greater than zero".into()));
        }
        if self.timestamp == 0 {
            return Err(CoreError::MissingField("timestamp".into()));
        }
        if self.version == 0 {
            return Err(CoreError::ValidationError("version must be greater than zero".into()));
        }

        Ok(())
    }

    /// Check if the payment has expired based on current time.
    pub fn is_expired(&self) -> bool {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let expiry_ms = self.timestamp + (self.ttl as u64 * 1000);
        now_ms > expiry_ms
    }

    /// Check if the payment has expired relative to a given timestamp.
    pub fn is_expired_at(&self, now_ms: u64) -> bool {
        let expiry_ms = self.timestamp + (self.ttl as u64 * 1000);
        now_ms > expiry_ms
    }

    /// Compute the canonical signing payload.
    /// This produces a deterministic byte sequence for Ed25519 signing.
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        // Version
        payload.push(self.version);

        // PM ID (16 bytes)
        payload.extend_from_slice(self.pm_id.as_bytes());

        // Sender DID (length-prefixed)
        let sender_bytes = self.sender_did.uri().as_bytes();
        payload.extend_from_slice(&(sender_bytes.len() as u32).to_be_bytes());
        payload.extend_from_slice(sender_bytes);

        // Receiver DID (length-prefixed)
        let receiver_bytes = self.receiver_did.uri().as_bytes();
        payload.extend_from_slice(&(receiver_bytes.len() as u32).to_be_bytes());
        payload.extend_from_slice(receiver_bytes);

        // Amount: value as big-endian u128 + currency code
        payload.extend_from_slice(&self.amount.value.to_be_bytes());
        let currency_str = format!("{}", self.amount.currency);
        let currency_bytes = currency_str.as_bytes();
        payload.extend_from_slice(&(currency_bytes.len() as u32).to_be_bytes());
        payload.extend_from_slice(currency_bytes);

        // TTL
        payload.extend_from_slice(&self.ttl.to_be_bytes());

        // Timestamp
        payload.extend_from_slice(&self.timestamp.to_be_bytes());

        // Metadata hash (not the full metadata, just its hash for determinism)
        if !self.metadata.is_empty() {
            let hash = blake3::hash(&self.metadata);
            payload.extend_from_slice(hash.as_bytes());
        }

        payload
    }

    /// Serialize to protobuf bytes.
    pub fn serialize(&self) -> Result<Vec<u8>, CoreError> {
        let proto = self.to_proto();
        let mut buf = Vec::with_capacity(proto.encoded_len());
        proto.encode(&mut buf)?;
        Ok(buf)
    }

    /// Deserialize from protobuf bytes.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, CoreError> {
        let proto = crate::proto::gppn::v1::PaymentMessage::decode(bytes)?;
        Self::from_proto(&proto)
    }

    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> crate::proto::gppn::v1::PaymentMessage {
        crate::proto::gppn::v1::PaymentMessage {
            pm_id: self.pm_id.as_bytes().to_vec(),
            version: self.version as u32,
            sender_did: self.sender_did.uri().to_string(),
            receiver_did: self.receiver_did.uri().to_string(),
            amount: Some(self.amount.to_proto()),
            settlement_preferences: self
                .settlement_preferences
                .iter()
                .map(|s| crate::proto::gppn::v1::SettlementHint {
                    adapter_id: s.adapter_id.clone(),
                    priority: s.priority as u32,
                    params: s.params.clone(),
                })
                .collect(),
            conditions: self
                .conditions
                .iter()
                .map(|c| crate::proto::gppn::v1::Condition {
                    condition_type: match c.condition_type {
                        ConditionType::TimeExpiry => 1,
                        ConditionType::Hashlock => 2,
                        ConditionType::MultiSig => 3,
                        ConditionType::Escrow => 4,
                    },
                    params: c.params.clone(),
                })
                .collect(),
            metadata: self.metadata.clone(),
            ttl: self.ttl,
            timestamp: self.timestamp,
            signature: self.signature.clone(),
            routing_hints: self
                .routing_hints
                .iter()
                .map(|r| crate::proto::gppn::v1::RoutingHint {
                    target_did: r.target_did.clone(),
                    preferred_adapters: r.preferred_adapters.clone(),
                    max_hops: r.max_hops,
                })
                .collect(),
            state: self.state.to_proto_i32(),
        }
    }

    /// Create from protobuf representation.
    pub fn from_proto(
        proto: &crate::proto::gppn::v1::PaymentMessage,
    ) -> Result<Self, CoreError> {
        let pm_id = if proto.pm_id.len() == 16 {
            uuid::Uuid::from_bytes(
                proto.pm_id[..16]
                    .try_into()
                    .map_err(|_| CoreError::ValidationError("invalid pm_id length".into()))?,
            )
        } else {
            return Err(CoreError::ValidationError("pm_id must be 16 bytes".into()));
        };

        let amount = proto
            .amount
            .as_ref()
            .ok_or_else(|| CoreError::MissingField("amount".into()))?;

        Ok(Self {
            pm_id,
            version: proto.version as u8,
            sender_did: Did::from_parts("key", &proto.sender_did.replace("did:gppn:key:", "")),
            receiver_did: Did::from_parts("key", &proto.receiver_did.replace("did:gppn:key:", "")),
            amount: Amount::from_proto(amount)?,
            settlement_preferences: proto
                .settlement_preferences
                .iter()
                .map(|s| SettlementHint {
                    adapter_id: s.adapter_id.clone(),
                    priority: s.priority as u8,
                    params: s.params.clone(),
                })
                .collect(),
            conditions: proto
                .conditions
                .iter()
                .map(|c| Condition {
                    condition_type: match c.condition_type {
                        1 => ConditionType::TimeExpiry,
                        2 => ConditionType::Hashlock,
                        3 => ConditionType::MultiSig,
                        4 => ConditionType::Escrow,
                        _ => ConditionType::TimeExpiry,
                    },
                    params: c.params.clone(),
                })
                .collect(),
            metadata: proto.metadata.clone(),
            ttl: proto.ttl,
            timestamp: proto.timestamp,
            signature: proto.signature.clone(),
            routing_hints: proto
                .routing_hints
                .iter()
                .map(|r| RoutingHint {
                    target_did: r.target_did.clone(),
                    preferred_adapters: r.preferred_adapters.clone(),
                    max_hops: r.max_hops,
                })
                .collect(),
            state: PaymentState::from_proto_i32(proto.state)?,
        })
    }
}

/// Builder for constructing PaymentMessage instances.
#[derive(Default)]
pub struct PaymentMessageBuilder {
    sender_did: Option<Did>,
    receiver_did: Option<Did>,
    amount: Option<Amount>,
    settlement_preferences: Vec<SettlementHint>,
    conditions: Vec<Condition>,
    metadata: Vec<u8>,
    ttl: u32,
    routing_hints: Vec<RoutingHint>,
}

impl PaymentMessageBuilder {
    /// Set the sender DID.
    pub fn sender(mut self, did: Did) -> Self {
        self.sender_did = Some(did);
        self
    }

    /// Set the receiver DID.
    pub fn receiver(mut self, did: Did) -> Self {
        self.receiver_did = Some(did);
        self
    }

    /// Set the payment amount.
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Add a settlement preference.
    pub fn settlement_preference(mut self, hint: SettlementHint) -> Self {
        self.settlement_preferences.push(hint);
        self
    }

    /// Add a condition.
    pub fn condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set encrypted metadata.
    pub fn metadata(mut self, metadata: Vec<u8>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set time-to-live in seconds.
    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = ttl;
        self
    }

    /// Add a routing hint.
    pub fn routing_hint(mut self, hint: RoutingHint) -> Self {
        self.routing_hints.push(hint);
        self
    }

    /// Build the PaymentMessage.
    pub fn build(self) -> Result<PaymentMessage, CoreError> {
        let sender_did = self
            .sender_did
            .ok_or_else(|| CoreError::MissingField("sender_did".into()))?;
        let receiver_did = self
            .receiver_did
            .ok_or_else(|| CoreError::MissingField("receiver_did".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| CoreError::MissingField("amount".into()))?;

        let ttl = if self.ttl > 0 { self.ttl } else { 300 }; // Default 5 minutes

        let pm = PaymentMessage {
            pm_id: uuid::Uuid::now_v7(),
            version: 1,
            sender_did,
            receiver_did,
            amount,
            settlement_preferences: self.settlement_preferences,
            conditions: self.conditions,
            metadata: self.metadata,
            ttl,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            signature: Vec::new(),
            routing_hints: self.routing_hints,
            state: PaymentState::Created,
        };

        pm.validate()?;
        Ok(pm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Currency, FiatCurrency};

    fn make_test_pm() -> PaymentMessage {
        PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(100_00, Currency::Fiat(FiatCurrency::BRL)))
            .ttl(300)
            .build()
            .expect("failed to build test PM")
    }

    #[test]
    fn test_builder_happy_path() {
        let pm = make_test_pm();
        assert_eq!(pm.version, 1);
        assert_eq!(pm.state, PaymentState::Created);
        assert_eq!(pm.sender_did.uri(), "did:gppn:key:alice123");
        assert_eq!(pm.receiver_did.uri(), "did:gppn:key:bob456");
        assert_eq!(pm.amount.value, 100_00);
        assert_eq!(pm.ttl, 300);
        assert!(!pm.pm_id.is_nil());
        assert!(pm.timestamp > 0);
    }

    #[test]
    fn test_builder_missing_sender() {
        let result = PaymentMessage::builder()
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_receiver() {
        let result = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_amount() {
        let result = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .ttl(60)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_zero_amount_fails() {
        let result = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(0, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_same_sender_receiver_fails() {
        let result = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "alice123"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_default_ttl() {
        let pm = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .build()
            .unwrap();
        assert_eq!(pm.ttl, 300); // 5 minutes default
    }

    #[test]
    fn test_is_expired_at() {
        let pm = make_test_pm();
        // Not expired at creation time
        assert!(!pm.is_expired_at(pm.timestamp));
        // Not expired 1 second later
        assert!(!pm.is_expired_at(pm.timestamp + 1000));
        // Expired after TTL
        assert!(pm.is_expired_at(pm.timestamp + (pm.ttl as u64 * 1000) + 1));
    }

    #[test]
    fn test_signing_payload_deterministic() {
        let pm = make_test_pm();
        let payload1 = pm.signing_payload();
        let payload2 = pm.signing_payload();
        assert_eq!(payload1, payload2);
        assert!(!payload1.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let pm = make_test_pm();
        let bytes = pm.serialize().expect("serialization failed");
        assert!(!bytes.is_empty());

        let deserialized = PaymentMessage::deserialize(&bytes).expect("deserialization failed");
        assert_eq!(deserialized.pm_id, pm.pm_id);
        assert_eq!(deserialized.version, pm.version);
        assert_eq!(deserialized.amount.value, pm.amount.value);
        assert_eq!(deserialized.ttl, pm.ttl);
        assert_eq!(deserialized.timestamp, pm.timestamp);
        assert_eq!(deserialized.state, pm.state);
    }

    #[test]
    fn test_with_settlement_preferences() {
        let pm = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .settlement_preference(SettlementHint {
                adapter_id: "sa-ethereum".into(),
                priority: 0,
                params: std::collections::HashMap::new(),
            })
            .settlement_preference(SettlementHint {
                adapter_id: "sa-internal".into(),
                priority: 1,
                params: std::collections::HashMap::new(),
            })
            .build()
            .unwrap();

        assert_eq!(pm.settlement_preferences.len(), 2);
        assert_eq!(pm.settlement_preferences[0].adapter_id, "sa-ethereum");
        assert_eq!(pm.settlement_preferences[1].priority, 1);
    }

    #[test]
    fn test_with_conditions() {
        let pm = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .condition(Condition {
                condition_type: ConditionType::Hashlock,
                params: vec![1, 2, 3],
            })
            .build()
            .unwrap();

        assert_eq!(pm.conditions.len(), 1);
        assert_eq!(pm.conditions[0].condition_type, ConditionType::Hashlock);
    }

    #[test]
    fn test_with_routing_hints() {
        let pm = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .routing_hint(RoutingHint {
                target_did: "did:gppn:key:relay1".into(),
                preferred_adapters: vec!["sa-ethereum".into()],
                max_hops: 3,
            })
            .build()
            .unwrap();

        assert_eq!(pm.routing_hints.len(), 1);
        assert_eq!(pm.routing_hints[0].max_hops, 3);
    }

    #[test]
    fn test_crypto_currencies() {
        let pm = PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice123"))
            .receiver(Did::from_parts("key", "bob456"))
            .amount(Amount::new(1_000_000, Currency::Crypto(crate::types::CryptoCurrency::USDC)))
            .ttl(60)
            .build()
            .unwrap();

        assert_eq!(pm.amount.value, 1_000_000);
        assert_eq!(pm.amount.currency, Currency::Crypto(crate::types::CryptoCurrency::USDC));
    }
}
