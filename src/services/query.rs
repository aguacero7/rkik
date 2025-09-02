use chrono::{DateTime, Local, Utc};
use std::time::Duration;

use crate::adapters::{ntp_client, resolver};
use crate::domain::ntp::{ProbeResult, Target};
use crate::error::RkikError;
use rsntp::ReferenceIdentifier;
use tracing::instrument;

/// Query a single target and return a [`ProbeResult`].
#[instrument(skip(timeout))]
pub async fn query_one(
    target: &str,
    ipv6_only: bool,
    timeout: Duration,
) -> Result<ProbeResult, RkikError> {
    let ip_and_port =target.split(":");
    if(ip_and_port.len ==2){
        let ip = resolver::resolve_ip(ip_and_port[0], ipv6_only)?;
        let port = ip_and_port[1];
    }else{
        let ip = resolver::resolve_ip(target, ipv6_only)?;
        let port = 123;
    }
    let res = ntp_client::query(ip, ipv6_only, timeout,port).await?;
    let utc: DateTime<Utc> = match res.datetime().try_into() {
        Ok(dt) => dt,
        Err(e) => return Err(RkikError::Other(e.to_string())),
    };
    let local: DateTime<Local> = DateTime::from(utc);
    let offset_ms = res.clock_offset().as_secs_f64() * 1000.0;
    let rtt_ms = res.round_trip_delay().as_secs_f64() * 1000.0;
    let stratum = res.stratum();
    let ref_id = format_reference_id(res.reference_identifier());
    let timestamp = utc.timestamp();
    Ok(ProbeResult {
        target: Target {
            name: target.to_string(),
            ip,
            port
        },
        offset_ms,
        rtt_ms,
        stratum,
        ref_id,
        utc,
        local,
        timestamp,
    })
}

fn format_reference_id(reference_id: &ReferenceIdentifier) -> String {
    reference_id.to_string()
}
