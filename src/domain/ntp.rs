use chrono::{DateTime, Local, Utc};
use std::net::IpAddr;

#[cfg(feature = "json")]
use serde::Serialize;

/// Target host resolved to an IP address.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Target {
    pub name: String,
    pub ip: IpAddr,
}

/// Result of probing an NTP server.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct ProbeResult {
    pub target: Target,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    pub stratum: u8,
    pub ref_id: String,
    pub utc: DateTime<Utc>,
    pub local: DateTime<Local>,
}
