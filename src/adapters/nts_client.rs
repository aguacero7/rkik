//! NTS (Network Time Security) client adapter using rkik-nts library.

#[cfg(feature = "nts")]
use rkik_nts::{NtsClient, NtsClientConfig};

use chrono::{DateTime, Utc};
use std::time::Duration;

use crate::error::RkikError;

/// Result of an NTS time query containing all relevant timing and authentication data.
#[derive(Debug, Clone)]
pub struct NtsTimeResult {
    /// The network time received from the NTS server
    pub network_time: DateTime<Utc>,
    /// Clock offset in milliseconds (positive means local clock is ahead)
    pub offset_ms: f64,
    /// Round-trip time in milliseconds
    pub rtt_ms: f64,
    /// Whether the response was cryptographically authenticated
    pub authenticated: bool,
    /// Server hostname
    pub server: String,
}

/// Query an NTS-enabled server and return the authenticated time result.
///
/// # Arguments
///
/// * `server` - The hostname of the NTS server (e.g., "time.cloudflare.com")
/// * `nts_ke_port` - Optional NTS-KE port (defaults to 4460 if None)
/// * `timeout` - Timeout duration for both NTS-KE and NTP operations
///
/// # Returns
///
/// Returns an `NtsTimeResult` containing the authenticated time data, or an error
/// if the NTS key exchange or NTP query fails.
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use rkik::adapters::nts_client::query_nts;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = query_nts("time.cloudflare.com", Some(4460), Duration::from_secs(10)).await?;
/// println!("Offset: {} ms (authenticated: {})", result.offset_ms, result.authenticated);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "nts")]
pub async fn query_nts(
    server: &str,
    nts_ke_port: Option<u16>,
    timeout: Duration,
) -> Result<NtsTimeResult, RkikError> {
    // Configure NTS client
    let mut config = NtsClientConfig::new(server);

    if let Some(port) = nts_ke_port {
        config = config.with_port(port);
    }

    config = config.with_timeout(timeout);

    // Create and connect NTS client
    let mut client = NtsClient::new(config);
    client
        .connect()
        .await
        .map_err(|e| RkikError::Nts(format!("NTS-KE failed: {}", e)))?;

    // Get authenticated time
    let time_snapshot = client
        .get_time()
        .await
        .map_err(|e| RkikError::Nts(format!("NTS time query failed: {}", e)))?;

    // Convert SystemTime to DateTime<Utc>
    let network_time: DateTime<Utc> = time_snapshot.network_time.into();

    // Convert offset from Duration to milliseconds
    // offset is the difference between network_time and system_time
    let offset_ms = time_snapshot.offset.as_secs_f64() * 1000.0;

    // Convert round_trip_delay from Duration to milliseconds
    let rtt_ms = time_snapshot.round_trip_delay.as_secs_f64() * 1000.0;

    // Convert to our result format
    Ok(NtsTimeResult {
        network_time,
        offset_ms,
        rtt_ms,
        authenticated: time_snapshot.authenticated,
        server: time_snapshot.server.clone(),
    })
}

/// Stub function when NTS feature is disabled
#[cfg(not(feature = "nts"))]
pub async fn query_nts(
    _server: &str,
    _nts_ke_port: Option<u16>,
    _timeout: Duration,
) -> Result<NtsTimeResult, RkikError> {
    Err(RkikError::Other(
        "NTS support not enabled. Compile with --features nts".to_string(),
    ))
}
