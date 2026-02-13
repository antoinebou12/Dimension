//! Error types for the network crate.

use std::fmt;

/// Errors that can occur in the network crate.
#[derive(Debug)]
pub enum NetworkError {
    /// Serialization or deserialization failed.
    Serialization(String),
    /// Transport-level error (connection, timeout, etc.).
    Transport(String),
    /// Invalid protocol message or state.
    Protocol(String),
    /// IO error.
    Io(std::io::Error),
    /// gRPC status error.
    #[cfg(feature = "grpc")]
    Grpc(tonic::Status),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::Serialization(s) => write!(f, "serialization error: {s}"),
            NetworkError::Transport(s) => write!(f, "transport error: {s}"),
            NetworkError::Protocol(s) => write!(f, "protocol error: {s}"),
            NetworkError::Io(e) => write!(f, "io error: {e}"),
            #[cfg(feature = "grpc")]
            NetworkError::Grpc(s) => write!(f, "grpc error: {s}"),
        }
    }
}

impl std::error::Error for NetworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NetworkError::Io(e) => Some(e),
            #[cfg(feature = "grpc")]
            NetworkError::Grpc(_) => None,
            _ => None,
        }
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(e: std::io::Error) -> Self {
        NetworkError::Io(e)
    }
}

#[cfg(feature = "grpc")]
impl From<tonic::Status> for NetworkError {
    fn from(s: tonic::Status) -> Self {
        NetworkError::Grpc(s)
    }
}
