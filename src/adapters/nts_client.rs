//! NTS (Network Time Security) client adapter using rkik-nts library.

#[cfg(feature = "nts")]
use rkik_nts::{NtsClient, NtsClientConfig};

use chrono::{DateTime, Utc};
use std::time::Duration;

use crate::error::RkikError;

#[cfg(feature = "json")]
use serde::Serialize;

/// Result of an NTS time query containing all relevant timing and authentication data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
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
    /// NTS-KE diagnostic data
    pub nts_ke_data: Option<NtsKeData>,
}

/// NTS-KE (Key Exchange) diagnostic data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct NtsKeData {
    /// Duration of the NTS-KE handshake (TLS + key exchange)
    pub ke_duration_ms: f64,
    /// Number of cookies received from the server
    pub cookie_count: usize,
    /// Sizes of each cookie in bytes
    pub cookie_sizes: Vec<usize>,
    /// AEAD algorithm negotiated (e.g., "AEAD_AES_SIV_CMAC_256")
    pub aead_algorithm: String,
    /// NTP server address (may differ from NTS-KE server)
    pub ntp_server: String,
    /// TLS certificate information (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<CertificateInfo>,
}

/// TLS Certificate information from NTS-KE handshake
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct CertificateInfo {
    /// Subject of the certificate (CN, O, etc.)
    pub subject: String,
    /// Issuer of the certificate
    pub issuer: String,
    /// Certificate validity period start (RFC3339 format)
    pub valid_from: String,
    /// Certificate validity period end (RFC3339 format)
    pub valid_until: String,
    /// Serial number (hex format)
    pub serial_number: String,
    /// Subject Alternative Names (DNS names)
    pub san_dns_names: Vec<String>,
    /// Signature algorithm
    pub signature_algorithm: String,
    /// Public key algorithm
    pub public_key_algorithm: String,
    /// Certificate fingerprint (SHA-256, hex format)
    pub fingerprint_sha256: String,
    /// Whether the certificate is self-signed
    pub is_self_signed: bool,
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

    // Perform NTS-KE handshake
    client
        .connect()
        .await
        .map_err(|e| RkikError::Nts(format!("NTS-KE failed: {}", e)))?;

    // Get authenticated time
    let time_snapshot = client
        .get_time()
        .await
        .map_err(|e| RkikError::Nts(format!("NTS time query failed: {}", e)))?;

    // Capture NTS-KE diagnostic data from the client
    let nts_ke_data = client.nts_ke_info().map(|ke_result| {
        // Convert rkik-nts CertificateInfo to our CertificateInfo
        let certificate = ke_result.certificate.as_ref().map(|cert| CertificateInfo {
            subject: cert.subject.clone(),
            issuer: cert.issuer.clone(),
            valid_from: cert.valid_from.clone(),
            valid_until: cert.valid_until.clone(),
            serial_number: cert.serial_number.clone(),
            san_dns_names: cert.san_dns_names.clone(),
            signature_algorithm: cert.signature_algorithm.clone(),
            public_key_algorithm: cert.public_key_algorithm.clone(),
            fingerprint_sha256: cert.fingerprint_sha256.clone(),
            is_self_signed: cert.is_self_signed,
        });

        NtsKeData {
            ke_duration_ms: ke_result.ke_duration().as_secs_f64() * 1000.0,
            cookie_count: ke_result.cookie_count(),
            cookie_sizes: ke_result.cookie_sizes(),
            aead_algorithm: ke_result.aead_algorithm.clone(),
            ntp_server: ke_result.ntp_server.to_string(),
            certificate,
        }
    });

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
        nts_ke_data,
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
