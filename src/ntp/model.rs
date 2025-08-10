use chrono::{DateTime, Local, Utc};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct NtpResult {
    pub ip: IpAddr,
    pub datetime_utc: DateTime<Utc>,
    pub local_time: DateTime<Local>,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    pub stratum: u8,
    pub reference_id: String,
}
