#[cfg(feature = "json")]
use serde::ser::SerializeStruct;
#[cfg(feature = "json")]
use serde::{Serialize, Serializer};
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
    /// NTS (Network Time Security) error.
    #[error("nts: {0}")]
    Nts(String),
    /// Underlying IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Other error cases.
    #[error("other: {0}")]
    Other(String),
    /// Error wrapper that carries target context (hostname/IP).
    #[error("{target} - {source}")]
    TargetContext {
        target: String,
        #[source]
        source: Box<RkikError>,
    },
}

impl RkikError {
    /// Attach a target to this error (if one is not already present).
    pub fn with_target(self, target: impl Into<String>) -> Self {
        let target = target.into();
        if target.trim().is_empty() || self.target().is_some() {
            return self;
        }
        Self::TargetContext {
            target,
            source: Box::new(self),
        }
    }

    /// Return target context when available.
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::TargetContext { target, .. } => Some(target.as_str()),
            _ => None,
        }
    }

    /// Return stable error kind for machine parsing.
    pub fn kind(&self) -> &'static str {
        match self.root() {
            Self::Dns(_) => "dns",
            Self::Network(_) => "network",
            Self::Protocol(_) => "protocol",
            Self::Nts(_) => "nts",
            Self::Io(_) => "io",
            Self::Other(_) => "other",
            Self::TargetContext { .. } => unreachable!("root() strips target wrappers"),
        }
    }

    /// Return raw (unwrapped) message payload without kind prefix.
    pub fn message(&self) -> String {
        match self.root() {
            Self::Dns(msg)
            | Self::Network(msg)
            | Self::Protocol(msg)
            | Self::Nts(msg)
            | Self::Other(msg) => msg.clone(),
            Self::Io(err) => err.to_string(),
            Self::TargetContext { .. } => unreachable!("root() strips target wrappers"),
        }
    }

    /// True when the underlying error is DNS-related.
    pub fn is_dns(&self) -> bool {
        matches!(self.root(), Self::Dns(_))
    }

    /// True when the underlying error is a network timeout.
    pub fn is_network_timeout(&self) -> bool {
        matches!(self.root(), Self::Network(msg) if msg == "timeout")
    }

    /// True when the underlying error is NTS-related.
    pub fn is_nts(&self) -> bool {
        matches!(self.root(), Self::Nts(_))
    }

    /// Serialize this error as JSON text.
    #[cfg(feature = "json")]
    pub fn to_json_string(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }

    fn root(&self) -> &Self {
        match self {
            Self::TargetContext { source, .. } => source.root(),
            other => other,
        }
    }
}

#[cfg(feature = "json")]
impl Serialize for RkikError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("RkikError", 3)?;
        st.serialize_field("kind", self.kind())?;
        st.serialize_field("message", &self.message())?;
        if let Some(target) = self.target() {
            st.serialize_field("target", target)?;
        }
        st.end()
    }
}

impl From<rsntp::SynchronizationError> for RkikError {
    fn from(err: rsntp::SynchronizationError) -> Self {
        match err {
            rsntp::SynchronizationError::IOError(e) => RkikError::Network(e.to_string()),
            rsntp::SynchronizationError::ProtocolError(e) => RkikError::Protocol(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RkikError;

    #[test]
    fn with_target_wraps_display_and_preserves_kind() {
        let err = RkikError::Network("timeout".into()).with_target("192.168.1.100");
        assert_eq!(err.to_string(), "192.168.1.100 - network: timeout");
        assert_eq!(err.target(), Some("192.168.1.100"));
        assert_eq!(err.kind(), "network");
        assert_eq!(err.message(), "timeout");
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_error_contains_target_field() {
        let err = RkikError::Dns("resolution failed".into()).with_target("time.example.com");
        let raw = err
            .to_json_string(false)
            .expect("json encoding should work");
        assert!(raw.contains("\"kind\":\"dns\""));
        assert!(raw.contains("\"message\":\"resolution failed\""));
        assert!(raw.contains("\"target\":\"time.example.com\""));
    }
}
