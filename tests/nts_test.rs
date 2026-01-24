//! Integration tests for NTS (Network Time Security) functionality

#[cfg(feature = "nts")]
use chrono::{DateTime, Local, Utc};
#[cfg(feature = "nts")]
use console::set_colors_enabled;
#[cfg(feature = "nts")]
use rkik::adapters::nts_client::{
    map_error_to_kind, NtsError, NtsErrorKind, NtsKeData, NtsValidationOutcome,
};
#[cfg(feature = "nts")]
use rkik::fmt;
#[cfg(feature = "nts")]
use rkik::{ProbeResult, Target};
#[cfg(feature = "nts")]
use std::net::IpAddr;

#[cfg(feature = "nts")]
fn sample_nts_probe() -> ProbeResult {
    let utc: DateTime<Utc> = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    let local: DateTime<Local> = DateTime::from(utc);
    let ip: IpAddr = "127.0.0.1".parse().unwrap();

    ProbeResult {
        target: Target {
            name: "nts.test".into(),
            ip,
            port: 123,
        },
        offset_ms: 1.5,
        rtt_ms: 0.6,
        stratum: 1,
        ref_id: "GPS".into(),
        utc,
        local,
        timestamp: utc.timestamp(),
        authenticated: true,
        nts_ke_data: Some(NtsKeData {
            ke_duration_ms: 12.5,
            cookie_count: 2,
            cookie_sizes: vec![], // Cookie sizes no longer exposed in rkik-nts 0.4.0
            aead_algorithm: "AEAD_AES_SIV_CMAC_256".into(),
            ntp_server: "nts.test".into(),
            certificate: None,
        }),
        nts_validation: Some(NtsValidationOutcome::success()),
    }
}

/// Create a sample NTS probe result with an AEAD failure for testing
#[cfg(feature = "nts")]
fn sample_nts_probe_with_error() -> ProbeResult {
    let utc: DateTime<Utc> = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    let local: DateTime<Local> = DateTime::from(utc);
    let ip: IpAddr = "127.0.0.1".parse().unwrap();

    ProbeResult {
        target: Target {
            name: "nts.test".into(),
            ip,
            port: 123,
        },
        offset_ms: 0.0,
        rtt_ms: 0.0,
        stratum: 0,
        ref_id: "".into(),
        utc,
        local,
        timestamp: utc.timestamp(),
        authenticated: false,
        nts_ke_data: None,
        nts_validation: Some(NtsValidationOutcome::failure(NtsError::new(
            NtsErrorKind::AeadFailure,
            "NTS AEAD authentication failed",
        ))),
    }
}

#[cfg(feature = "nts")]
#[test]
fn nts_text_render_includes_authenticated_markers_and_diagnostics() {
    set_colors_enabled(false);
    let probe = sample_nts_probe();
    let rendered = fmt::text::render_probe(&probe, true);
    assert!(
        rendered.contains("[NTS Authenticated]"),
        "expected authenticated badge in probe output: {}",
        rendered
    );
    assert!(
        rendered.contains("=== NTS-KE Diagnostics ==="),
        "expected diagnostics section in verbose probe output: {}",
        rendered
    );

    let rendered_compare = fmt::text::render_compare(std::slice::from_ref(&probe), true);
    assert!(
        rendered_compare.contains("[NTS]"),
        "expected NTS badge in compare output: {}",
        rendered_compare
    );
}

#[cfg(all(feature = "nts", feature = "json"))]
#[test]
fn nts_json_render_controls_diagnostics_with_verbosity() {
    let probes = vec![sample_nts_probe()];
    let compact = fmt::json::to_json(&probes, /* pretty */ false, /* verbose */ false).unwrap();
    assert!(
        compact.contains("\"authenticated\":true"),
        "compact JSON should include authenticated flag: {compact}"
    );
    assert!(
        !compact.contains("nts_ke_data"),
        "compact JSON should omit NTS-KE diagnostics: {compact}"
    );

    let verbose = fmt::json::to_json(&probes, /* pretty */ false, /* verbose */ true).unwrap();
    assert!(
        verbose.contains("\"nts_ke_data\""),
        "verbose JSON should include diagnostics: {verbose}"
    );
    assert!(
        verbose.contains("\"cookie_count\":2"),
        "verbose JSON should serialize NTS-KE fields: {verbose}"
    );
}

#[cfg(all(feature = "nts", feature = "network-tests"))]
#[tokio::test]
async fn test_nts_query_swedish_server() {
    use rkik::services::query::query_one;
    use std::time::Duration;

    let result = query_one(
        "nts.ntp.se",
        false,                   // ipv6
        Duration::from_secs(15), // timeout
        true,                    // use_nts
        4460,                    // nts_port
    )
    .await;

    assert!(result.is_ok(), "NTS query should succeed: {:?}", result);

    let probe = result.unwrap();
    assert_eq!(probe.target.name, "nts.ntp.se");
    assert!(probe.authenticated, "Result should be NTS authenticated");
    assert!(probe.rtt_ms > 0.0, "RTT should be positive");
}

#[cfg(all(feature = "nts", feature = "network-tests"))]
#[tokio::test]
async fn test_nts_query_cloudflare() {
    use rkik::services::query::query_one;
    use std::time::Duration;

    let result = query_one(
        "time.cloudflare.com",
        false,                   // ipv6
        Duration::from_secs(15), // timeout
        true,                    // use_nts
        4460,                    // nts_port
    )
    .await;

    // Note: Cloudflare NTS might not always be available
    if let Ok(probe) = result {
        assert_eq!(probe.target.name, "time.cloudflare.com");
        assert!(probe.authenticated, "Result should be NTS authenticated");
        assert!(probe.rtt_ms > 0.0, "RTT should be positive");
    }
}

#[cfg(all(feature = "nts", feature = "network-tests"))]
#[tokio::test]
async fn test_nts_compare_servers() {
    use rkik::services::compare::compare_many;
    use std::time::Duration;

    let servers = vec!["nts.ntp.se".to_string()];

    let result = compare_many(
        &servers,
        false,                   // ipv6
        Duration::from_secs(15), // timeout
        true,                    // use_nts
        4460,                    // nts_port
    )
    .await;

    assert!(result.is_ok(), "NTS compare should succeed: {:?}", result);

    let probes = result.unwrap();
    assert_eq!(probes.len(), 1);
    assert!(
        probes[0].authenticated,
        "All results should be NTS authenticated"
    );
}

#[cfg(feature = "nts")]
#[tokio::test]
async fn test_nts_disabled_on_regular_server() {
    use rkik::services::query::query_one;
    use std::time::Duration;

    // Query a regular NTP server without NTS
    let result = query_one(
        "time.google.com",
        false,                  // ipv6
        Duration::from_secs(5), // timeout
        false,                  // use_nts = false
        4460,                   // nts_port (ignored)
    )
    .await;

    if let Ok(probe) = result {
        assert!(
            !probe.authenticated,
            "Regular NTP query should not be authenticated"
        );
    }
}

// ============================================================================
// NTS Validation Error Tests
// ============================================================================

#[cfg(feature = "nts")]
#[test]
fn nts_error_kind_as_str_returns_correct_values() {
    assert_eq!(NtsErrorKind::AeadFailure.as_str(), "aead_failure");
    assert_eq!(
        NtsErrorKind::MissingAuthenticator.as_str(),
        "missing_authenticator"
    );
    assert_eq!(NtsErrorKind::InvalidUniqueId.as_str(), "invalid_unique_id");
    assert_eq!(
        NtsErrorKind::UnauthenticatedResponse.as_str(),
        "unauthenticated_response"
    );
    assert_eq!(
        NtsErrorKind::KeHandshakeFailed.as_str(),
        "ke_handshake_failed"
    );
    assert_eq!(
        NtsErrorKind::CertificateInvalid.as_str(),
        "certificate_invalid"
    );
    assert_eq!(NtsErrorKind::MissingCookies.as_str(), "missing_cookies");
    assert_eq!(
        NtsErrorKind::MalformedExtensions.as_str(),
        "malformed_extensions"
    );
    assert_eq!(
        NtsErrorKind::InvalidOriginTimestamp.as_str(),
        "invalid_origin_timestamp"
    );
    assert_eq!(NtsErrorKind::Timeout.as_str(), "timeout");
    assert_eq!(NtsErrorKind::Network.as_str(), "network");
    assert_eq!(NtsErrorKind::Unknown.as_str(), "unknown");
}

#[cfg(feature = "nts")]
#[test]
fn nts_error_kind_plugin_exit_codes_are_correct() {
    // Security-critical failures should return CRITICAL (2)
    assert_eq!(NtsErrorKind::AeadFailure.plugin_exit_code(), 2);
    assert_eq!(NtsErrorKind::MissingAuthenticator.plugin_exit_code(), 2);
    assert_eq!(NtsErrorKind::UnauthenticatedResponse.plugin_exit_code(), 2);
    assert_eq!(NtsErrorKind::InvalidUniqueId.plugin_exit_code(), 2);
    assert_eq!(NtsErrorKind::InvalidOriginTimestamp.plugin_exit_code(), 2);

    // Configuration/connection issues should return UNKNOWN (3)
    assert_eq!(NtsErrorKind::KeHandshakeFailed.plugin_exit_code(), 3);
    assert_eq!(NtsErrorKind::CertificateInvalid.plugin_exit_code(), 3);
    assert_eq!(NtsErrorKind::MissingCookies.plugin_exit_code(), 3);
    assert_eq!(NtsErrorKind::MalformedExtensions.plugin_exit_code(), 3);
    assert_eq!(NtsErrorKind::Timeout.plugin_exit_code(), 3);
    assert_eq!(NtsErrorKind::Network.plugin_exit_code(), 3);
    assert_eq!(NtsErrorKind::Unknown.plugin_exit_code(), 3);
}

#[cfg(feature = "nts")]
#[test]
fn nts_validation_outcome_success_is_authenticated() {
    let outcome = NtsValidationOutcome::success();
    assert!(outcome.authenticated);
    assert!(outcome.error.is_none());
}

#[cfg(feature = "nts")]
#[test]
fn nts_validation_outcome_failure_has_error() {
    let error = NtsError::new(NtsErrorKind::AeadFailure, "AEAD auth failed");
    let outcome = NtsValidationOutcome::failure(error);
    assert!(!outcome.authenticated);
    assert!(outcome.error.is_some());
    assert_eq!(
        outcome.error.as_ref().unwrap().kind,
        NtsErrorKind::AeadFailure
    );
}

#[cfg(feature = "nts")]
#[test]
fn nts_text_render_shows_failure_badge_on_error() {
    set_colors_enabled(false);
    let probe = sample_nts_probe_with_error();
    let rendered = fmt::text::render_probe(&probe, false);
    assert!(
        rendered.contains("[NTS Failed]"),
        "expected failure badge in probe output: {}",
        rendered
    );
    assert!(
        rendered.contains("aead_failure"),
        "expected error kind in probe output: {}",
        rendered
    );
}

#[cfg(feature = "nts")]
#[test]
fn nts_text_render_shows_error_details_in_verbose() {
    set_colors_enabled(false);
    let probe = sample_nts_probe_with_error();
    let rendered = fmt::text::render_probe(&probe, true);
    assert!(
        rendered.contains("=== NTS Validation Error ==="),
        "expected error section in verbose output: {}",
        rendered
    );
    assert!(
        rendered.contains("Error Kind:"),
        "expected error kind label in verbose output: {}",
        rendered
    );
    assert!(
        rendered.contains("NTS AEAD authentication failed"),
        "expected error message in verbose output: {}",
        rendered
    );
}

#[cfg(feature = "nts")]
#[test]
fn nts_compare_render_shows_failure_badge() {
    set_colors_enabled(false);
    let probe = sample_nts_probe_with_error();
    let rendered = fmt::text::render_compare(std::slice::from_ref(&probe), false);
    assert!(
        rendered.contains("[NTS FAILED]"),
        "expected NTS FAILED badge in compare output: {}",
        rendered
    );
}

#[cfg(all(feature = "nts", feature = "json"))]
#[test]
fn nts_json_includes_validation_error_in_verbose() {
    let probe = sample_nts_probe_with_error();
    let json = fmt::json::to_json(std::slice::from_ref(&probe), false, true).unwrap();
    assert!(
        json.contains("\"nts\""),
        "verbose JSON should include nts field: {}",
        json
    );
    assert!(
        json.contains("\"authenticated\":false"),
        "verbose JSON should show authenticated false: {}",
        json
    );
    assert!(
        json.contains("\"kind\":\"aead_failure\""),
        "verbose JSON should include error kind: {}",
        json
    );
    assert!(
        json.contains("\"message\":\"NTS AEAD authentication failed\""),
        "verbose JSON should include error message: {}",
        json
    );
}

#[cfg(all(feature = "nts", feature = "json"))]
#[test]
fn nts_json_omits_validation_in_non_verbose() {
    let probe = sample_nts_probe_with_error();
    let json = fmt::json::to_json(std::slice::from_ref(&probe), false, false).unwrap();
    assert!(
        !json.contains("\"nts\""),
        "non-verbose JSON should omit nts field: {}",
        json
    );
    assert!(
        json.contains("\"authenticated\":false"),
        "JSON should still include authenticated flag: {}",
        json
    );
}

// ============================================================================
// map_error_to_kind Tests
// ============================================================================

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_aead_failures() {
    assert_eq!(
        map_error_to_kind("AEAD failure while decrypting"),
        NtsErrorKind::AeadFailure
    );
    assert_eq!(
        map_error_to_kind("authentication tag verification failed"),
        NtsErrorKind::AeadFailure
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_authenticator_errors() {
    assert_eq!(
        map_error_to_kind("missing authenticator in NTS extension"),
        NtsErrorKind::MissingAuthenticator
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_unique_id_errors() {
    assert_eq!(
        map_error_to_kind("invalid unique identifier"),
        NtsErrorKind::InvalidUniqueId
    );
    assert_eq!(
        map_error_to_kind("UID mismatch in response"),
        NtsErrorKind::InvalidUniqueId
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_origin_timestamp_errors() {
    assert_eq!(
        map_error_to_kind("origin timestamp validation failed"),
        NtsErrorKind::InvalidOriginTimestamp
    );
    assert_eq!(
        map_error_to_kind("possible replay attack detected"),
        NtsErrorKind::InvalidOriginTimestamp
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_cookie_errors() {
    assert_eq!(
        map_error_to_kind("no cookies received from server"),
        NtsErrorKind::MissingCookies
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_certificate_before_malformed() {
    // "malformed certificate" should map to CertificateInvalid, not MalformedExtensions
    assert_eq!(
        map_error_to_kind("malformed certificate in chain"),
        NtsErrorKind::CertificateInvalid
    );
    assert_eq!(
        map_error_to_kind("certificate extensions invalid"),
        NtsErrorKind::CertificateInvalid
    );
    assert_eq!(
        map_error_to_kind("cert verification failed"),
        NtsErrorKind::CertificateInvalid
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_malformed_extensions() {
    // Pure extension errors without certificate context
    assert_eq!(
        map_error_to_kind("NTS extension field malformed"),
        NtsErrorKind::MalformedExtensions
    );
    assert_eq!(
        map_error_to_kind("missing required extension"),
        NtsErrorKind::MalformedExtensions
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_handshake_errors() {
    assert_eq!(
        map_error_to_kind("NTS-KE handshake failed"),
        NtsErrorKind::KeHandshakeFailed
    );
    assert_eq!(
        map_error_to_kind("TLS negotiation error"),
        NtsErrorKind::KeHandshakeFailed
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_timeout_before_network() {
    // Timeout should be preferred over network for timeout messages
    assert_eq!(
        map_error_to_kind("connection timed out"),
        NtsErrorKind::Timeout
    );
    assert_eq!(
        map_error_to_kind("network timeout waiting for response"),
        NtsErrorKind::Timeout
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_classifies_network_errors() {
    assert_eq!(
        map_error_to_kind("network unreachable"),
        NtsErrorKind::Network
    );
    assert_eq!(
        map_error_to_kind("connection refused"),
        NtsErrorKind::Network
    );
}

#[cfg(feature = "nts")]
#[test]
fn map_error_to_kind_returns_unknown_for_unrecognized() {
    assert_eq!(
        map_error_to_kind("some completely unrelated error"),
        NtsErrorKind::Unknown
    );
}
