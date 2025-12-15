#![cfg(all(feature = "ptp", target_os = "linux"))]

use std::time::Duration;

use futures::future::join_all;

use crate::adapters::{ptp_client, resolver};
use crate::domain::ptp::PtpProbeResult;
use crate::error::RkikError;

use super::query::parse_target;

/// Options controlling a PTP probe.
#[derive(Debug, Clone)]
pub struct PtpQueryOptions {
    pub domain: u8,
    pub event_port: u16,
    pub general_port: u16,
    pub hw_timestamping: bool,
    pub verbose: bool,
}

impl PtpQueryOptions {
    pub fn new(
        domain: u8,
        event_port: u16,
        general_port: u16,
        hw_timestamping: bool,
        verbose: bool,
    ) -> Self {
        Self {
            domain,
            event_port,
            general_port,
            hw_timestamping,
            verbose,
        }
    }
}

/// Query a single target and produce a [`PtpProbeResult`].
pub async fn query_target(
    target: &str,
    mut ipv6: bool,
    timeout: Duration,
    opts: &PtpQueryOptions,
) -> Result<PtpProbeResult, RkikError> {
    let parsed = parse_target(target)?;
    if parsed.is_ipv6_literal {
        ipv6 = true;
    }

    let ip = resolver::resolve_ip(parsed.host, ipv6)?;
    let event_port = parsed.port.unwrap_or(opts.event_port);

    ptp_client::query_ptp(
        target,
        ip,
        opts.domain,
        event_port,
        opts.general_port,
        opts.hw_timestamping,
        timeout,
        opts.verbose,
    )
    .await
}

/// Query several targets concurrently and return the successful results.
pub async fn query_many(
    targets: &[String],
    ipv6: bool,
    timeout: Duration,
    opts: &PtpQueryOptions,
) -> Result<Vec<PtpProbeResult>, RkikError> {
    let futures = targets
        .iter()
        .map(|t| query_target(t, ipv6, timeout, opts))
        .collect::<Vec<_>>();

    let results = join_all(futures).await;
    let mut out = Vec::new();
    for res in results {
        out.push(res?);
    }
    Ok(out)
}
