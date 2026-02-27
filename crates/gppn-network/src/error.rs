//! Network error types for the GPPN P2P layer.

use libp2p::{gossipsub, noise, TransportError};

/// Errors that can occur in the GPPN network layer.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// Transport-level error (TCP, Noise, Yamux).
    #[error("transport error: {0}")]
    Transport(String),

    /// GossipSub publish or subscription error.
    #[error("gossipsub error: {0}")]
    Gossipsub(String),

    /// Kademlia DHT error.
    #[error("kademlia error: {0}")]
    Kademlia(String),

    /// Failed to dial a peer.
    #[error("dial error: {0}")]
    Dial(String),

    /// Error listening on an address.
    #[error("listen error: {0}")]
    Listen(String),

    /// Serialization / deserialization error.
    #[error("codec error: {0}")]
    Codec(String),

    /// The node has not been started yet.
    #[error("node not started")]
    NotStarted,

    /// The node is already running.
    #[error("node already running")]
    AlreadyRunning,

    /// The node has been shut down.
    #[error("node shut down")]
    ShutDown,

    /// Channel send/receive failure.
    #[error("channel error: {0}")]
    Channel(String),

    /// Error from the core layer.
    #[error("core error: {0}")]
    Core(#[from] gppn_core::CoreError),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic / catchall error.
    #[error("{0}")]
    Other(String),
}

impl From<gossipsub::PublishError> for NetworkError {
    fn from(err: gossipsub::PublishError) -> Self {
        NetworkError::Gossipsub(err.to_string())
    }
}

impl From<gossipsub::SubscriptionError> for NetworkError {
    fn from(err: gossipsub::SubscriptionError) -> Self {
        NetworkError::Gossipsub(err.to_string())
    }
}

impl From<noise::Error> for NetworkError {
    fn from(err: noise::Error) -> Self {
        NetworkError::Transport(err.to_string())
    }
}

impl<T: std::fmt::Debug> From<TransportError<T>> for NetworkError {
    fn from(err: TransportError<T>) -> Self {
        NetworkError::Transport(format!("{:?}", err))
    }
}

impl From<libp2p::multiaddr::Error> for NetworkError {
    fn from(err: libp2p::multiaddr::Error) -> Self {
        NetworkError::Transport(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NetworkError::Transport("connection refused".into());
        assert_eq!(err.to_string(), "transport error: connection refused");
    }

    #[test]
    fn test_error_not_started() {
        let err = NetworkError::NotStarted;
        assert_eq!(err.to_string(), "node not started");
    }

    #[test]
    fn test_error_already_running() {
        let err = NetworkError::AlreadyRunning;
        assert_eq!(err.to_string(), "node already running");
    }

    #[test]
    fn test_error_shut_down() {
        let err = NetworkError::ShutDown;
        assert_eq!(err.to_string(), "node shut down");
    }

    #[test]
    fn test_error_codec() {
        let err = NetworkError::Codec("invalid protobuf".into());
        assert_eq!(err.to_string(), "codec error: invalid protobuf");
    }

    #[test]
    fn test_error_other() {
        let err = NetworkError::Other("something went wrong".into());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let net_err: NetworkError = io_err.into();
        assert!(matches!(net_err, NetworkError::Io(_)));
        assert!(net_err.to_string().contains("file not found"));
    }
}
