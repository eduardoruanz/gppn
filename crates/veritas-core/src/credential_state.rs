use std::fmt;

use crate::error::CoreError;

/// The states of a Verifiable Credential lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CredentialState {
    /// Credential is being prepared, not yet signed.
    Draft,
    /// Credential has been signed by the issuer.
    Issued,
    /// Credential is active and can be used for presentations.
    Active,
    /// Credential is temporarily suspended.
    Suspended,
    /// Credential has been permanently revoked. Final state.
    Revoked,
    /// Credential has expired. Final state.
    Expired,
}

impl CredentialState {
    /// Whether this is a final (terminal) state.
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Revoked | Self::Expired)
    }

    /// Convert to the protobuf CredentialState enum value.
    pub fn to_proto_i32(&self) -> i32 {
        match self {
            Self::Draft => 1,
            Self::Issued => 2,
            Self::Active => 3,
            Self::Suspended => 4,
            Self::Revoked => 5,
            Self::Expired => 6,
        }
    }

    /// Create from protobuf CredentialState enum value.
    pub fn from_proto_i32(value: i32) -> Result<Self, CoreError> {
        match value {
            1 => Ok(Self::Draft),
            2 => Ok(Self::Issued),
            3 => Ok(Self::Active),
            4 => Ok(Self::Suspended),
            5 => Ok(Self::Revoked),
            6 => Ok(Self::Expired),
            _ => Err(CoreError::ValidationError(format!(
                "invalid credential state value: {}",
                value
            ))),
        }
    }
}

impl fmt::Display for CredentialState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Issued => write!(f, "Issued"),
            Self::Active => write!(f, "Active"),
            Self::Suspended => write!(f, "Suspended"),
            Self::Revoked => write!(f, "Revoked"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// Events that trigger credential state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialEvent {
    /// Issuer signs and issues the credential.
    Issue,
    /// Holder activates the credential for use.
    Activate,
    /// Issuer suspends the credential temporarily.
    Suspend,
    /// Issuer reinstates a suspended credential.
    Reinstate,
    /// Issuer permanently revokes the credential.
    Revoke,
    /// The credential's TTL has expired.
    Expire,
}

/// Manages credential state transitions according to the Veritas protocol.
///
/// Valid transitions:
/// - Draft → Issued (Issue)
/// - Issued → Active (Activate)
/// - Issued → Revoked (Revoke)
/// - Active → Suspended (Suspend)
/// - Active → Revoked (Revoke)
/// - Active → Expired (Expire)
/// - Suspended → Active (Reinstate)
/// - Suspended → Revoked (Revoke)
/// - Suspended → Expired (Expire)
pub struct CredentialStateMachine;

impl CredentialStateMachine {
    /// Attempt a state transition based on an event.
    /// Returns the new state on success, or an error for invalid transitions.
    pub fn transition(
        current: CredentialState,
        event: CredentialEvent,
    ) -> Result<CredentialState, CoreError> {
        let new_state = match (current, event) {
            // From Draft
            (CredentialState::Draft, CredentialEvent::Issue) => CredentialState::Issued,

            // From Issued
            (CredentialState::Issued, CredentialEvent::Activate) => CredentialState::Active,
            (CredentialState::Issued, CredentialEvent::Revoke) => CredentialState::Revoked,

            // From Active
            (CredentialState::Active, CredentialEvent::Suspend) => CredentialState::Suspended,
            (CredentialState::Active, CredentialEvent::Revoke) => CredentialState::Revoked,
            (CredentialState::Active, CredentialEvent::Expire) => CredentialState::Expired,

            // From Suspended
            (CredentialState::Suspended, CredentialEvent::Reinstate) => CredentialState::Active,
            (CredentialState::Suspended, CredentialEvent::Revoke) => CredentialState::Revoked,
            (CredentialState::Suspended, CredentialEvent::Expire) => CredentialState::Expired,

            // All other transitions are invalid
            _ => {
                let target = match event {
                    CredentialEvent::Issue => CredentialState::Issued,
                    CredentialEvent::Activate => CredentialState::Active,
                    CredentialEvent::Suspend => CredentialState::Suspended,
                    CredentialEvent::Reinstate => CredentialState::Active,
                    CredentialEvent::Revoke => CredentialState::Revoked,
                    CredentialEvent::Expire => CredentialState::Expired,
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
            "credential state transition"
        );

        Ok(new_state)
    }

    /// Check if a transition is valid without performing it.
    pub fn can_transition(current: CredentialState, event: CredentialEvent) -> bool {
        Self::transition(current, event).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        // Draft → Issued → Active
        let state = CredentialState::Draft;
        let state = CredentialStateMachine::transition(state, CredentialEvent::Issue).unwrap();
        assert_eq!(state, CredentialState::Issued);

        let state = CredentialStateMachine::transition(state, CredentialEvent::Activate).unwrap();
        assert_eq!(state, CredentialState::Active);
    }

    #[test]
    fn test_suspend_and_reinstate() {
        let state = CredentialState::Active;
        let state = CredentialStateMachine::transition(state, CredentialEvent::Suspend).unwrap();
        assert_eq!(state, CredentialState::Suspended);

        let state = CredentialStateMachine::transition(state, CredentialEvent::Reinstate).unwrap();
        assert_eq!(state, CredentialState::Active);
    }

    #[test]
    fn test_revoke_from_active() {
        let state =
            CredentialStateMachine::transition(CredentialState::Active, CredentialEvent::Revoke)
                .unwrap();
        assert_eq!(state, CredentialState::Revoked);
        assert!(state.is_final());
    }

    #[test]
    fn test_revoke_from_suspended() {
        let state =
            CredentialStateMachine::transition(CredentialState::Suspended, CredentialEvent::Revoke)
                .unwrap();
        assert_eq!(state, CredentialState::Revoked);
    }

    #[test]
    fn test_revoke_from_issued() {
        let state =
            CredentialStateMachine::transition(CredentialState::Issued, CredentialEvent::Revoke)
                .unwrap();
        assert_eq!(state, CredentialState::Revoked);
    }

    #[test]
    fn test_expire_from_active() {
        let state =
            CredentialStateMachine::transition(CredentialState::Active, CredentialEvent::Expire)
                .unwrap();
        assert_eq!(state, CredentialState::Expired);
        assert!(state.is_final());
    }

    #[test]
    fn test_expire_from_suspended() {
        let state =
            CredentialStateMachine::transition(CredentialState::Suspended, CredentialEvent::Expire)
                .unwrap();
        assert_eq!(state, CredentialState::Expired);
    }

    #[test]
    fn test_invalid_transition_from_revoked() {
        let result = CredentialStateMachine::transition(
            CredentialState::Revoked,
            CredentialEvent::Reinstate,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_from_expired() {
        let result =
            CredentialStateMachine::transition(CredentialState::Expired, CredentialEvent::Activate);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_activate_from_draft() {
        let result =
            CredentialStateMachine::transition(CredentialState::Draft, CredentialEvent::Activate);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_suspend_from_draft() {
        let result =
            CredentialStateMachine::transition(CredentialState::Draft, CredentialEvent::Suspend);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_issue_from_active() {
        let result =
            CredentialStateMachine::transition(CredentialState::Active, CredentialEvent::Issue);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_reinstate_active() {
        let result =
            CredentialStateMachine::transition(CredentialState::Active, CredentialEvent::Reinstate);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_expire_draft() {
        let result =
            CredentialStateMachine::transition(CredentialState::Draft, CredentialEvent::Expire);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_transition() {
        assert!(CredentialStateMachine::can_transition(
            CredentialState::Draft,
            CredentialEvent::Issue
        ));
        assert!(!CredentialStateMachine::can_transition(
            CredentialState::Revoked,
            CredentialEvent::Activate
        ));
    }

    #[test]
    fn test_all_final_states() {
        assert!(CredentialState::Revoked.is_final());
        assert!(CredentialState::Expired.is_final());
        assert!(!CredentialState::Draft.is_final());
        assert!(!CredentialState::Issued.is_final());
        assert!(!CredentialState::Active.is_final());
        assert!(!CredentialState::Suspended.is_final());
    }

    #[test]
    fn test_proto_roundtrip() {
        for state in [
            CredentialState::Draft,
            CredentialState::Issued,
            CredentialState::Active,
            CredentialState::Suspended,
            CredentialState::Revoked,
            CredentialState::Expired,
        ] {
            let proto_val = state.to_proto_i32();
            let back = CredentialState::from_proto_i32(proto_val).unwrap();
            assert_eq!(state, back);
        }
    }

    #[test]
    fn test_invalid_proto_value() {
        assert!(CredentialState::from_proto_i32(0).is_err());
        assert!(CredentialState::from_proto_i32(99).is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CredentialState::Draft), "Draft");
        assert_eq!(format!("{}", CredentialState::Active), "Active");
        assert_eq!(format!("{}", CredentialState::Revoked), "Revoked");
    }

    #[test]
    fn test_full_lifecycle() {
        // Draft → Issued → Active → Suspended → Active → Expired
        let s = CredentialState::Draft;
        let s = CredentialStateMachine::transition(s, CredentialEvent::Issue).unwrap();
        let s = CredentialStateMachine::transition(s, CredentialEvent::Activate).unwrap();
        let s = CredentialStateMachine::transition(s, CredentialEvent::Suspend).unwrap();
        let s = CredentialStateMachine::transition(s, CredentialEvent::Reinstate).unwrap();
        assert_eq!(s, CredentialState::Active);
        let s = CredentialStateMachine::transition(s, CredentialEvent::Expire).unwrap();
        assert_eq!(s, CredentialState::Expired);
        assert!(s.is_final());
    }
}
