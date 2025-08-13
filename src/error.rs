use thiserror::Error;

/// Top-level error type for rkik library.
#[derive(Error, Debug)]
pub enum RkikError {
    /// DNS resolution failure.
    #[error("dns: {0}")]
    Dns(String),
    /// Network related error.
    #[error("network: {0}")]
    Network(String),
    /// Protocol violation.
    #[error("protocol: {0}")]
    Protocol(String),
    /// Underlying IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Other error cases.
    #[error("other: {0}")]
    Other(String),
}

impl From<rsntp::SynchronizationError> for RkikError {
    fn from(err: rsntp::SynchronizationError) -> Self {
        match err {
            rsntp::SynchronizationError::IOError(e) => RkikError::Network(e.to_string()),
            rsntp::SynchronizationError::ProtocolError(e) => RkikError::Protocol(e.to_string()),
        }
    }
}
