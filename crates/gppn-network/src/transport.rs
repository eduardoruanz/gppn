//! libp2p transport stack construction for the GPPN network.
//!
//! Builds a transport layer using TCP + Noise (encryption) + Yamux (multiplexing),
//! which is the standard secure transport for libp2p nodes.

use libp2p::identity::Keypair;

use crate::behaviour::GppnBehaviour;
use crate::error::NetworkError;

/// Configuration for building the GPPN network transport.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// The listen address (e.g., "/ip4/0.0.0.0/tcp/9000").
    pub listen_addr: String,
    /// Idle connection timeout in seconds.
    pub idle_connection_timeout_secs: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".into(),
            idle_connection_timeout_secs: 60,
        }
    }
}

/// Build a libp2p Swarm with the GPPN behaviour using TCP + Noise + Yamux.
///
/// This uses the `SwarmBuilder` API introduced in libp2p 0.54 which provides
/// a clean builder pattern for constructing the full transport + behaviour stack.
pub fn build_swarm(
    keypair: Keypair,
    behaviour_fn: impl FnOnce(&Keypair) -> Result<GppnBehaviour, NetworkError>,
) -> Result<libp2p::Swarm<GppnBehaviour>, NetworkError> {
    // Pre-build the behaviour so we can handle errors cleanly before
    // entering the SwarmBuilder chain.
    let behaviour = behaviour_fn(&keypair)?;

    // Move the pre-built behaviour into an Option so we can take it from
    // inside the closure.
    let mut behaviour_slot = Some(behaviour);

    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .map_err(|e| NetworkError::Transport(e.to_string()))?
        .with_behaviour(|_key| {
            // This closure is called exactly once by the SwarmBuilder.
            // behaviour_slot is guaranteed to be Some because we set it above
            // and the SwarmBuilder only calls this closure once.
            behaviour_slot
                .take()
                .expect("behaviour_slot is always Some at this point")
        })
        .map_err(|e| NetworkError::Transport(e.to_string()))?
        .with_swarm_config(|cfg: libp2p::swarm::Config| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
        })
        .build();

    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.listen_addr, "/ip4/0.0.0.0/tcp/0");
        assert_eq!(config.idle_connection_timeout_secs, 60);
    }

    #[test]
    fn test_transport_config_custom() {
        let config = TransportConfig {
            listen_addr: "/ip4/127.0.0.1/tcp/9000".into(),
            idle_connection_timeout_secs: 120,
        };
        assert_eq!(config.listen_addr, "/ip4/127.0.0.1/tcp/9000");
        assert_eq!(config.idle_connection_timeout_secs, 120);
    }

    #[test]
    fn test_build_swarm_success() {
        let keypair = Keypair::generate_ed25519();
        let result = build_swarm(keypair, |key| {
            crate::behaviour::GppnBehaviour::new(key)
        });
        assert!(result.is_ok());
    }
}
