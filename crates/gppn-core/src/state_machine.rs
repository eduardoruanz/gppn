use std::fmt;

use crate::error::CoreError;

/// The 8 states of a Payment Message lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PaymentState {
    /// Payment has been created but not yet routed.
    Created,
    /// A route has been found for the payment.
    Routed,
    /// The receiver has accepted the payment.
    Accepted,
    /// Settlement is in progress.
    Settling,
    /// Settlement confirmed — payment is final.
    Settled,
    /// Payment failed (can be retried via re-route).
    Failed,
    /// Payment expired — TTL exceeded. Final state.
    Expired,
    /// Payment was cancelled by the sender. Final state.
    Cancelled,
}

impl PaymentState {
    /// Whether this is a final (terminal) state.
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Settled | Self::Expired | Self::Cancelled)
    }

    /// Convert to the protobuf PaymentState enum value.
    pub fn to_proto_i32(&self) -> i32 {
        match self {
            Self::Created => 1,
            Self::Routed => 2,
            Self::Accepted => 3,
            Self::Settling => 4,
            Self::Settled => 5,
            Self::Failed => 6,
            Self::Expired => 7,
            Self::Cancelled => 8,
        }
    }

    /// Create from protobuf PaymentState enum value.
    pub fn from_proto_i32(value: i32) -> Result<Self, CoreError> {
        match value {
            1 => Ok(Self::Created),
            2 => Ok(Self::Routed),
            3 => Ok(Self::Accepted),
            4 => Ok(Self::Settling),
            5 => Ok(Self::Settled),
            6 => Ok(Self::Failed),
            7 => Ok(Self::Expired),
            8 => Ok(Self::Cancelled),
            _ => Err(CoreError::ValidationError(format!(
                "invalid payment state value: {}",
                value
            ))),
        }
    }
}

impl fmt::Display for PaymentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Routed => write!(f, "Routed"),
            Self::Accepted => write!(f, "Accepted"),
            Self::Settling => write!(f, "Settling"),
            Self::Settled => write!(f, "Settled"),
            Self::Failed => write!(f, "Failed"),
            Self::Expired => write!(f, "Expired"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Events that trigger state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentEvent {
    /// A route was found for the payment.
    RouteFound,
    /// The receiver accepted the payment.
    Accepted,
    /// Settlement has started.
    SettlementStarted,
    /// Settlement was confirmed successfully.
    SettlementConfirmed,
    /// Settlement or routing failed.
    SettlementFailed,
    /// The payment TTL expired.
    Expired,
    /// The sender cancelled the payment.
    Cancelled,
    /// Retry routing after a failure.
    RetryRoute,
}

/// Manages payment state transitions according to the GPPN protocol.
///
/// Valid transitions:
/// - Created → Routed (RouteFound)
/// - Created → Expired (Expired)
/// - Created → Cancelled (Cancelled)
/// - Routed → Accepted (Accepted)
/// - Routed → Failed (SettlementFailed)
/// - Routed → Expired (Expired)
/// - Routed → Cancelled (Cancelled)
/// - Accepted → Settling (SettlementStarted)
/// - Accepted → Failed (SettlementFailed)
/// - Accepted → Expired (Expired)
/// - Accepted → Cancelled (Cancelled)
/// - Settling → Settled (SettlementConfirmed)
/// - Settling → Failed (SettlementFailed)
/// - Failed → Routed (RetryRoute)
pub struct PaymentStateMachine;

impl PaymentStateMachine {
    /// Attempt a state transition based on an event.
    /// Returns the new state on success, or an error for invalid transitions.
    pub fn transition(
        current: PaymentState,
        event: PaymentEvent,
    ) -> Result<PaymentState, CoreError> {
        let new_state = match (current, event) {
            // From Created
            (PaymentState::Created, PaymentEvent::RouteFound) => PaymentState::Routed,
            (PaymentState::Created, PaymentEvent::Expired) => PaymentState::Expired,
            (PaymentState::Created, PaymentEvent::Cancelled) => PaymentState::Cancelled,

            // From Routed
            (PaymentState::Routed, PaymentEvent::Accepted) => PaymentState::Accepted,
            (PaymentState::Routed, PaymentEvent::SettlementFailed) => PaymentState::Failed,
            (PaymentState::Routed, PaymentEvent::Expired) => PaymentState::Expired,
            (PaymentState::Routed, PaymentEvent::Cancelled) => PaymentState::Cancelled,

            // From Accepted
            (PaymentState::Accepted, PaymentEvent::SettlementStarted) => PaymentState::Settling,
            (PaymentState::Accepted, PaymentEvent::SettlementFailed) => PaymentState::Failed,
            (PaymentState::Accepted, PaymentEvent::Expired) => PaymentState::Expired,
            (PaymentState::Accepted, PaymentEvent::Cancelled) => PaymentState::Cancelled,

            // From Settling
            (PaymentState::Settling, PaymentEvent::SettlementConfirmed) => PaymentState::Settled,
            (PaymentState::Settling, PaymentEvent::SettlementFailed) => PaymentState::Failed,

            // From Failed — can retry
            (PaymentState::Failed, PaymentEvent::RetryRoute) => PaymentState::Routed,

            // All other transitions are invalid
            _ => {
                let target = match event {
                    PaymentEvent::RouteFound => PaymentState::Routed,
                    PaymentEvent::Accepted => PaymentState::Accepted,
                    PaymentEvent::SettlementStarted => PaymentState::Settling,
                    PaymentEvent::SettlementConfirmed => PaymentState::Settled,
                    PaymentEvent::SettlementFailed => PaymentState::Failed,
                    PaymentEvent::Expired => PaymentState::Expired,
                    PaymentEvent::Cancelled => PaymentState::Cancelled,
                    PaymentEvent::RetryRoute => PaymentState::Routed,
                };
                return Err(CoreError::InvalidStateTransition {
                    from: current,
                    to: target,
                });
            }
        };

        tracing::debug!(
            from = %current,
            to = %new_state,
            event = ?event,
            "payment state transition"
        );

        Ok(new_state)
    }

    /// Check if a transition is valid without performing it.
    pub fn can_transition(current: PaymentState, event: PaymentEvent) -> bool {
        Self::transition(current, event).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        // Created → Routed → Accepted → Settling → Settled
        let state = PaymentState::Created;
        let state = PaymentStateMachine::transition(state, PaymentEvent::RouteFound).unwrap();
        assert_eq!(state, PaymentState::Routed);

        let state = PaymentStateMachine::transition(state, PaymentEvent::Accepted).unwrap();
        assert_eq!(state, PaymentState::Accepted);

        let state = PaymentStateMachine::transition(state, PaymentEvent::SettlementStarted).unwrap();
        assert_eq!(state, PaymentState::Settling);

        let state = PaymentStateMachine::transition(state, PaymentEvent::SettlementConfirmed).unwrap();
        assert_eq!(state, PaymentState::Settled);

        assert!(state.is_final());
    }

    #[test]
    fn test_failure_and_retry() {
        // Created → Routed → Failed → Routed (retry)
        let state = PaymentState::Created;
        let state = PaymentStateMachine::transition(state, PaymentEvent::RouteFound).unwrap();
        let state = PaymentStateMachine::transition(state, PaymentEvent::SettlementFailed).unwrap();
        assert_eq!(state, PaymentState::Failed);
        assert!(!state.is_final());

        let state = PaymentStateMachine::transition(state, PaymentEvent::RetryRoute).unwrap();
        assert_eq!(state, PaymentState::Routed);
    }

    #[test]
    fn test_expiry_from_created() {
        let state = PaymentStateMachine::transition(PaymentState::Created, PaymentEvent::Expired).unwrap();
        assert_eq!(state, PaymentState::Expired);
        assert!(state.is_final());
    }

    #[test]
    fn test_expiry_from_routed() {
        let state = PaymentStateMachine::transition(PaymentState::Routed, PaymentEvent::Expired).unwrap();
        assert_eq!(state, PaymentState::Expired);
    }

    #[test]
    fn test_expiry_from_accepted() {
        let state = PaymentStateMachine::transition(PaymentState::Accepted, PaymentEvent::Expired).unwrap();
        assert_eq!(state, PaymentState::Expired);
    }

    #[test]
    fn test_cancellation_from_created() {
        let state = PaymentStateMachine::transition(PaymentState::Created, PaymentEvent::Cancelled).unwrap();
        assert_eq!(state, PaymentState::Cancelled);
        assert!(state.is_final());
    }

    #[test]
    fn test_cancellation_from_routed() {
        let state = PaymentStateMachine::transition(PaymentState::Routed, PaymentEvent::Cancelled).unwrap();
        assert_eq!(state, PaymentState::Cancelled);
    }

    #[test]
    fn test_cancellation_from_accepted() {
        let state = PaymentStateMachine::transition(PaymentState::Accepted, PaymentEvent::Cancelled).unwrap();
        assert_eq!(state, PaymentState::Cancelled);
    }

    #[test]
    fn test_settling_failure() {
        let state = PaymentStateMachine::transition(PaymentState::Settling, PaymentEvent::SettlementFailed).unwrap();
        assert_eq!(state, PaymentState::Failed);
    }

    #[test]
    fn test_invalid_transition_from_settled() {
        // Settled is final — no transitions allowed
        let result = PaymentStateMachine::transition(PaymentState::Settled, PaymentEvent::RouteFound);
        assert!(result.is_err());

        let result = PaymentStateMachine::transition(PaymentState::Settled, PaymentEvent::Cancelled);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_from_expired() {
        let result = PaymentStateMachine::transition(PaymentState::Expired, PaymentEvent::RouteFound);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_from_cancelled() {
        let result = PaymentStateMachine::transition(PaymentState::Cancelled, PaymentEvent::RetryRoute);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_settle_from_created() {
        let result = PaymentStateMachine::transition(PaymentState::Created, PaymentEvent::SettlementStarted);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_accept_from_created() {
        let result = PaymentStateMachine::transition(PaymentState::Created, PaymentEvent::Accepted);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_cancel_settling() {
        let result = PaymentStateMachine::transition(PaymentState::Settling, PaymentEvent::Cancelled);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_expire_settling() {
        // Once settling has started, only success or failure allowed
        let result = PaymentStateMachine::transition(PaymentState::Settling, PaymentEvent::Expired);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_transition() {
        assert!(PaymentStateMachine::can_transition(PaymentState::Created, PaymentEvent::RouteFound));
        assert!(!PaymentStateMachine::can_transition(PaymentState::Settled, PaymentEvent::RouteFound));
    }

    #[test]
    fn test_all_final_states() {
        assert!(PaymentState::Settled.is_final());
        assert!(PaymentState::Expired.is_final());
        assert!(PaymentState::Cancelled.is_final());
        assert!(!PaymentState::Created.is_final());
        assert!(!PaymentState::Routed.is_final());
        assert!(!PaymentState::Accepted.is_final());
        assert!(!PaymentState::Settling.is_final());
        assert!(!PaymentState::Failed.is_final());
    }

    #[test]
    fn test_proto_roundtrip() {
        for state in [
            PaymentState::Created,
            PaymentState::Routed,
            PaymentState::Accepted,
            PaymentState::Settling,
            PaymentState::Settled,
            PaymentState::Failed,
            PaymentState::Expired,
            PaymentState::Cancelled,
        ] {
            let proto_val = state.to_proto_i32();
            let back = PaymentState::from_proto_i32(proto_val).unwrap();
            assert_eq!(state, back);
        }
    }

    #[test]
    fn test_invalid_proto_value() {
        assert!(PaymentState::from_proto_i32(0).is_err());
        assert!(PaymentState::from_proto_i32(99).is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", PaymentState::Created), "Created");
        assert_eq!(format!("{}", PaymentState::Settled), "Settled");
    }
}
