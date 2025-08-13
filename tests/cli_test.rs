use assert_cmd::Command;
use predicates::str::contains;
#[cfg(feature = "network-tests")]
use rkik::adapters::resolver::resolve_ip;

#[cfg(feature = "network-tests")]
#[test]
fn test_resolve_ip_v4_priority() {
    let ip = resolve_ip("1.pool.ntp.org", false).expect("Should resolve");
    assert!(ip.is_ipv4(), "Expected IPv4, got {}", ip);
}

#[cfg(feature = "network-tests")]
#[test]
fn test_resolve_ip_v6_only() {
    let ip = resolve_ip("2.pool.ntp.org", true).expect("Should resolve");
    assert!(ip.is_ipv6(), "Expected IPv6, got {}", ip);
}

#[cfg(feature = "network-tests")]
#[test]
fn test_positional_argument_as_server() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("--nocolor")
        .arg("1.pool.ntp.org")
        .assert()
        .success()
        .stdout(contains("Server:"));
}

#[test]
fn test_invalid_input_no_args() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("--nocolor")
        .assert()
        .failure()
        .stdout(contains("Error:"));
}
