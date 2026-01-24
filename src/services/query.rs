use chrono::{DateTime, Local, Utc};
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "nts")]
use crate::adapters::nts_client;
use crate::adapters::{ntp_client, resolver};
use crate::domain::ntp::{ProbeResult, Target};
use crate::error::RkikError;
use rsntp::ReferenceIdentifier;
use tracing::instrument;

/// Parsed view of a target string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTarget<'a> {
    pub host: &'a str,
    pub port: Option<u16>,
    pub is_ipv6_literal: bool,
}
/// Strict port parsing with range check (1..=65535).
fn parse_port_strict(s: &str) -> Result<u16, RkikError> {
    let raw = u32::from_str(s).map_err(|_| RkikError::Other(format!("invalid port: '{s}'")))?;
    if raw == 0 || raw > u16::MAX as u32 {
        return Err(RkikError::Other(format!(
            "port out of range [1..65535]: {raw}"
        )));
    }
    Ok(raw as u16)
}

/// Count occurrences of ':' (helps distinguish host:port vs bare IPv6).
#[inline]
fn colon_count(s: &str) -> usize {
    s.as_bytes().iter().filter(|&&b| b == b':').count()
}

/// Parse a user target string without regexes.
///
/// Supported forms:
/// - "hostname"
/// - "hostname:123"
/// - "1.2.3.4"
/// - "1.2.3.4:123"
/// - "[2001:db8::1]"
/// - "[2001:db8::1]:123"
/// - "2001:db8::1"              (bare IPv6, **no** port allowed)
///
/// Rules:
/// - If it starts with '[', it must be bracketed IPv6, optional ":port".
/// - Otherwise:
///   - If there's more than one ':', treat as **bare IPv6** (no port).
///   - If there's exactly one ':', treat as "host:port".
///   - If there's zero ':', treat as "host" (hostname or IPv4).
pub fn parse_target(input: &str) -> Result<ParsedTarget<'_>, RkikError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(RkikError::Other("empty target".into()));
    }

    // Case 1: Bracketed IPv6: "[v6]" or "[v6]:port"
    if let Some(rest) = s.strip_prefix('[') {
        // Find the matching ']'
        let Some(bracket_pos) = rest.find(']') else {
            return Err(RkikError::Other(format!("missing closing ']' in '{s}'")));
        };
        let host = &rest[..bracket_pos]; // inside brackets (IPv6 literal)
        let tail = &rest[bracket_pos + 1..]; // after ']'

        // Optional ":port" after the bracket
        let port = if let Some(p) = tail.strip_prefix(':') {
            Some(parse_port_strict(p)?)
        } else if tail.is_empty() {
            None
        } else {
            return Err(RkikError::Other(format!(
                "unexpected trailing characters in '{s}'"
            )));
        };

        return Ok(ParsedTarget {
            host,
            port,
            is_ipv6_literal: true,
        });
    }

    // Case 2: Non-bracketed input
    match colon_count(s) {
        // No colon: "hostname" or "1.2.3.4"
        0 => Ok(ParsedTarget {
            host: s,
            port: None,
            is_ipv6_literal: false,
        }),

        // Exactly one colon: "host:port" (hostname or IPv4)
        1 => {
            let mut it = s.rsplitn(2, ':');
            let port_str = it.next().unwrap();
            let host = it.next().unwrap_or("");
            if host.is_empty() {
                return Err(RkikError::Other(format!(
                    "missing host before port in '{s}'"
                )));
            }
            let port = parse_port_strict(port_str)?;
            Ok(ParsedTarget {
                host,
                port: Some(port),
                is_ipv6_literal: false,
            })
        }

        _ => Ok(ParsedTarget {
            host: s,
            port: None,
            is_ipv6_literal: true,
        }),
    }
}

fn format_reference_id(reference_id: &ReferenceIdentifier) -> String {
    reference_id.to_string()
}

/// Query a single target and return a [`ProbeResult`].
///
/// # Arguments
///
/// * `target` - The target server (hostname or IP address)
/// * `ipv6` - Whether to use IPv6
/// * `timeout` - Timeout duration
/// * `use_nts` - Whether to use NTS (Network Time Security) authentication
/// * `nts_port` - NTS-KE port number (typically 4460)
#[instrument(skip(timeout))]
pub async fn query_one(
    target: &str,
    mut ipv6: bool,
    timeout: Duration,
    use_nts: bool,
    nts_port: u16,
) -> Result<ProbeResult, RkikError> {
    // NTS branch
    #[cfg(feature = "nts")]
    if use_nts {
        let parsed = parse_target(target)?;
        let nts_result = nts_client::query_nts(parsed.host, Some(nts_port), timeout).await?;

        // Resolve IP for display purposes
        let ip: IpAddr = resolver::resolve_ip(parsed.host, ipv6)?;
        let local: DateTime<Local> = DateTime::from(nts_result.network_time);
        let timestamp = nts_result.network_time.timestamp();

        return Ok(ProbeResult {
            target: Target {
                name: target.to_string(),
                ip,
                port: parsed.port.unwrap_or(123),
            },
            offset_ms: nts_result.offset_ms,
            rtt_ms: nts_result.rtt_ms,
            stratum: 0, // NTS library doesn't expose stratum
            ref_id: nts_result.server.clone(),
            utc: nts_result.network_time,
            local,
            timestamp,
            authenticated: nts_result.authenticated,
            #[cfg(feature = "nts")]
            nts_ke_data: nts_result.nts_ke_data,
            #[cfg(feature = "nts")]
            nts_validation: Some(nts_result.nts_validation),
        });
    }

    // Standard NTP branch (without NTS)
    #[cfg(not(feature = "nts"))]
    if use_nts {
        return Err(RkikError::Other(
            "NTS support not enabled. Compile with --features nts".to_string(),
        ));
    }

    let parsed = parse_target(target)?;

    let ip: IpAddr = resolver::resolve_ip(parsed.host, ipv6)?;

    let port: u16 = parsed.port.unwrap_or(123);
    if parsed.is_ipv6_literal {
        ipv6 = true;
    }
    let res = ntp_client::query(ip, ipv6, timeout, port).await?;

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
            port,
        },
        offset_ms,
        rtt_ms,
        stratum,
        ref_id,
        utc,
        local,
        timestamp,
        authenticated: false, // Standard NTP is not authenticated
        #[cfg(feature = "nts")]
        nts_ke_data: None, // No NTS-KE data for standard NTP queries
        #[cfg(feature = "nts")]
        nts_validation: None, // No NTS validation for standard NTP queries
    })
}
