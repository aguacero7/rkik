//! Integration tests for NTS (Network Time Security) functionality

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
