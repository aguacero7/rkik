use chrono::{DateTime, Local, Utc};

use crate::errors::RkikError;
use crate::ntp::{client, model::NtpResult, resolver};

/// Query a single server and return the normalized result
pub fn query(server: &str, ipv6: bool) -> Result<NtpResult, RkikError> {
    let ip = resolver::resolve_ip(server, ipv6)?;
    let sync_result = client::sync(ip, ipv6)?;
    let datetime_utc: DateTime<Utc> = sync_result.datetime().try_into().unwrap();
    let local_time: DateTime<Local> = DateTime::from(datetime_utc);
    Ok(NtpResult {
        ip,
        datetime_utc,
        local_time,
        offset_ms: sync_result.clock_offset().as_secs_f64() * 1000.0,
        rtt_ms: sync_result.round_trip_delay().as_secs_f64() * 1000.0,
        stratum: sync_result.stratum(),
        reference_id: client::format_reference_id(sync_result.reference_identifier()),
    })
}
