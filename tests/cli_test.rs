use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use rkik::resolve_ip_for_mode;

#[cfg(feature = "network-tests")]
#[test]
fn test_resolve_ip_v4_priority() {
    let ip = resolve_ip_for_mode("1.pool.ntp.org", false).expect("Should resolve");
    assert!(ip.is_ipv4(), "Expected IPv4, got {}", ip);
}

#[cfg(feature = "network-tests")]
#[test]
fn test_resolve_ip_v6_only() {
    let ip = resolve_ip_for_mode("2.pool.ntp.org", true).expect("Should resolve");
    assert!(ip.is_ipv6(), "Expected IPv6, got {}", ip);
}

#[cfg(feature = "network-tests")]
#[test]
fn test_resolve_direct_ipv4_literal() {
    let ip = resolve_ip_for_mode("8.8.8.8", false).expect("Should parse literal");
    assert_eq!(ip.to_string(), "8.8.8.8");
}

#[cfg(feature = "network-tests")]
#[test]
fn test_resolve_direct_ipv6_literal() {
    let ip = resolve_ip_for_mode("2001:4860:4860::8888", true).expect("Should parse literal");
    assert_eq!(ip.to_string(), "2001:4860:4860::8888");
}

#[cfg(feature = "network-tests")]
#[test]
fn test_positional_argument_as_server() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("1.pool.ntp.org")
        .assert()
        .success()
        .stdout(contains("Server:"));
}

#[cfg(feature = "network-tests")]
#[test]
fn test_server_flag_argument() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("--server")
        .arg("1.pool.ntp.org")
        .assert()
        .success()
        .stdout(contains("Server:"));
}

#[cfg(feature = "network-tests")]
#[test]
fn test_compare_argument_json_output() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.args([
        "--compare",
        "0.pool.ntp.org",
        "1.pool.ntp.org",
        "--format",
        "json",
    ])
    .assert()
    .success()
    .stdout(contains("\"difference_ms\""));
}

#[test]
fn test_invalid_input_no_args() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.assert().failure().stdout(contains("Error:"));
}

#[cfg(feature = "network-tests")]
#[test]
fn test_cli_ipv6_text_output() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.args(["--server", "2.pool.ntp.org", "--ipv6", "--format", "text"])
        .assert()
        .success()
        .stdout(contains("IP:").and(contains("v6")));
}
