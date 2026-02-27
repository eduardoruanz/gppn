use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::error::CoreError;

/// Value in atomic units (satoshis, centavos, wei, etc.) represented as u128.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    /// Value in the smallest unit of the currency.
    pub value: u128,
    /// The currency of this amount.
    pub currency: Currency,
}

impl Amount {
    /// Create a new amount.
    pub fn new(value: u128, currency: Currency) -> Self {
        Self { value, currency }
    }

    /// Convert to protobuf Amount.
    pub fn to_proto(&self) -> crate::proto::gppn::v1::Amount {
        crate::proto::gppn::v1::Amount {
            value_high: (self.value >> 64) as u64,
            value_low: self.value as u64,
            currency: Some(self.currency.to_proto()),
        }
    }

    /// Create from protobuf Amount.
    pub fn from_proto(proto: &crate::proto::gppn::v1::Amount) -> Result<Self, CoreError> {
        let currency = proto
            .currency
            .as_ref()
            .ok_or_else(|| CoreError::MissingField("currency".into()))?;
        let value = ((proto.value_high as u128) << 64) | (proto.value_low as u128);
        Ok(Self {
            value,
            currency: Currency::from_proto(currency)?,
        })
    }

    /// Check if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}

/// Currency types supported by GPPN.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    /// Fiat currency with ISO 4217 code.
    Fiat(FiatCurrency),
    /// Cryptocurrency.
    Crypto(CryptoCurrency),
    /// Custom token.
    Custom(String),
}

impl Currency {
    /// Convert to protobuf Currency.
    pub fn to_proto(&self) -> crate::proto::gppn::v1::Currency {
        match self {
            Currency::Fiat(fiat) => crate::proto::gppn::v1::Currency {
                currency_type: crate::proto::gppn::v1::CurrencyType::Fiat as i32,
                code: fiat.code().to_string(),
                decimals: fiat.decimals(),
            },
            Currency::Crypto(crypto) => crate::proto::gppn::v1::Currency {
                currency_type: crate::proto::gppn::v1::CurrencyType::Crypto as i32,
                code: crypto.code().to_string(),
                decimals: crypto.decimals(),
            },
            Currency::Custom(code) => crate::proto::gppn::v1::Currency {
                currency_type: crate::proto::gppn::v1::CurrencyType::Custom as i32,
                code: code.clone(),
                decimals: 0,
            },
        }
    }

    /// Create from protobuf Currency.
    pub fn from_proto(proto: &crate::proto::gppn::v1::Currency) -> Result<Self, CoreError> {
        match proto.currency_type {
            x if x == crate::proto::gppn::v1::CurrencyType::Fiat as i32 => {
                FiatCurrency::from_code(&proto.code)
                    .map(Currency::Fiat)
                    .ok_or_else(|| CoreError::ValidationError(format!("unknown fiat currency: {}", proto.code)))
            }
            x if x == crate::proto::gppn::v1::CurrencyType::Crypto as i32 => {
                CryptoCurrency::from_code(&proto.code)
                    .map(Currency::Crypto)
                    .ok_or_else(|| CoreError::ValidationError(format!("unknown crypto currency: {}", proto.code)))
            }
            x if x == crate::proto::gppn::v1::CurrencyType::Custom as i32 => {
                Ok(Currency::Custom(proto.code.clone()))
            }
            _ => Err(CoreError::ValidationError("unknown currency type".into())),
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Fiat(fiat) => write!(f, "{}", fiat.code()),
            Currency::Crypto(crypto) => write!(f, "{}", crypto.code()),
            Currency::Custom(code) => write!(f, "{}", code),
        }
    }
}

/// ISO 4217 fiat currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiatCurrency {
    BRL,
    USD,
    EUR,
    GBP,
    JPY,
    CNY,
    CHF,
    AUD,
    CAD,
    INR,
}

impl FiatCurrency {
    /// ISO 4217 code.
    pub fn code(&self) -> &str {
        match self {
            Self::BRL => "BRL",
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::GBP => "GBP",
            Self::JPY => "JPY",
            Self::CNY => "CNY",
            Self::CHF => "CHF",
            Self::AUD => "AUD",
            Self::CAD => "CAD",
            Self::INR => "INR",
        }
    }

    /// Number of decimal places.
    pub fn decimals(&self) -> u32 {
        match self {
            Self::JPY => 0,
            _ => 2,
        }
    }

    /// Parse from ISO 4217 code.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "BRL" => Some(Self::BRL),
            "USD" => Some(Self::USD),
            "EUR" => Some(Self::EUR),
            "GBP" => Some(Self::GBP),
            "JPY" => Some(Self::JPY),
            "CNY" => Some(Self::CNY),
            "CHF" => Some(Self::CHF),
            "AUD" => Some(Self::AUD),
            "CAD" => Some(Self::CAD),
            "INR" => Some(Self::INR),
            _ => None,
        }
    }
}

/// Cryptocurrencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CryptoCurrency {
    BTC,
    ETH,
    SOL,
    USDC,
    USDT,
}

impl CryptoCurrency {
    /// Currency symbol.
    pub fn code(&self) -> &str {
        match self {
            Self::BTC => "BTC",
            Self::ETH => "ETH",
            Self::SOL => "SOL",
            Self::USDC => "USDC",
            Self::USDT => "USDT",
        }
    }

    /// Number of decimal places.
    pub fn decimals(&self) -> u32 {
        match self {
            Self::BTC => 8,
            Self::ETH => 18,
            Self::SOL => 9,
            Self::USDC | Self::USDT => 6,
        }
    }

    /// Parse from code.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "BTC" => Some(Self::BTC),
            "ETH" => Some(Self::ETH),
            "SOL" => Some(Self::SOL),
            "USDC" => Some(Self::USDC),
            "USDT" => Some(Self::USDT),
            _ => None,
        }
    }
}

/// Decentralized Identifier (DID) in the GPPN protocol.
/// Format: `did:gppn:<method>:<identifier>`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    /// Create a new DID from a full URI string.
    pub fn new(uri: String) -> Result<Self, CoreError> {
        if !uri.starts_with("did:gppn:") {
            return Err(CoreError::InvalidDid(format!(
                "DID must start with 'did:gppn:', got: {}",
                uri
            )));
        }
        let parts: Vec<&str> = uri.split(':').collect();
        if parts.len() < 4 {
            return Err(CoreError::InvalidDid(format!(
                "DID must have format 'did:gppn:<method>:<identifier>', got: {}",
                uri
            )));
        }
        Ok(Self(uri))
    }

    /// Create a DID from method and identifier components.
    pub fn from_parts(method: &str, identifier: &str) -> Self {
        Self(format!("did:gppn:{}:{}", method, identifier))
    }

    /// Get the full DID URI.
    pub fn uri(&self) -> &str {
        &self.0
    }

    /// Extract the method (key, web, chain).
    pub fn method(&self) -> Option<&str> {
        self.0.split(':').nth(2)
    }

    /// Extract the identifier.
    pub fn identifier(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.splitn(4, ':').collect();
        parts.get(3).copied()
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Settlement hint indicating preferred settlement mechanisms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementHint {
    /// Adapter identifier (e.g., "sa-ethereum", "sa-pix").
    pub adapter_id: String,
    /// Priority: 0 = highest.
    pub priority: u8,
    /// Adapter-specific parameters.
    pub params: HashMap<String, String>,
}

/// Payment condition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionType {
    /// Expires at a specific timestamp.
    TimeExpiry,
    /// Hash time-locked.
    Hashlock,
    /// Requires N of M signatures.
    MultiSig,
    /// Escrow with mediator.
    Escrow,
}

/// A condition attached to a payment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    /// Type of condition.
    pub condition_type: ConditionType,
    /// Serialized condition parameters.
    pub params: Vec<u8>,
}

/// Routing hint to guide path discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingHint {
    /// Target DID for routing.
    pub target_did: String,
    /// Preferred settlement adapters.
    pub preferred_adapters: Vec<String>,
    /// Maximum number of hops.
    pub max_hops: u32,
}
