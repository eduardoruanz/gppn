pub mod error;
pub mod types;
pub mod payment_message;
pub mod state_machine;
pub mod config;

/// Generated protobuf types — source of truth from proto/gppn/v1/*.proto
pub mod proto {
    pub mod gppn {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/gppn.v1.rs"));
        }
    }
}

pub use error::CoreError;
pub use payment_message::PaymentMessage;
pub use state_machine::{PaymentState, PaymentStateMachine, PaymentEvent};
pub use types::{Amount, Currency, Did, SettlementHint, Condition, ConditionType, RoutingHint};
