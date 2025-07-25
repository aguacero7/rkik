use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::{contains, is_match};
use rkik::resolve_ip;
use std::net::IpAddr;

#[test]
fn test_resolve_ip_v4() {
    let ip = resolve_ip_for_mode("1.pool.ntp.org", false);
    assert!(ip.is_some());
    assert!(ip.unwrap().parse::<IpAddr>().unwrap().is_ipv4());
}

#[test]
fn test_resolve_ip_v6() {
    let ip = resolve_ip_for_mode("2.pool.ntp.org", true);
    assert!(ip.is_some());
    assert!(ip.unwrap().parse::<IpAddr>().unwrap().is_ipv6());
}

#[test]
fn test_resolve_direct_ipv4() {
    let ip = resolve_ip_for_mode("8.8.8.8", false);
    assert_eq!(ip.unwrap(), "8.8.8.8");
}

#[test]
fn test_resolve_direct_ipv6() {
    let ip = resolve_ip_for_mode("2001:4860:4860::8888", true);
    assert_eq!(ip.unwrap(), "2001:4860:4860::8888");
}

#[test]
fn test_positional_argument_as_server() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("1.pool.ntp.org")
        .assert()
        .success()
        .stdout(contains("Server:").or(contains("server")));
}

#[test]
fn test_server_flag_argument() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("--server")
        .arg("1.pool.ntp.org")
        .assert()
        .success()
        .stdout(contains("Server:").or(contains("server")));
}

#[test]
fn test_compare_argument() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("--compare")
        .args(["0.pool.ntp.org", "1.pool.ntp.org"])
        .assert()
        .success()
        .stdout(is_match("Difference:|difference_ms").unwrap());
}

#[test]
fn test_invalid_input() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.assert()
        .failure()
        .stdout(contains("Error: Provide either a server"));
}

#[test]
fn test_cli_ipv6() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.args(["--server", "time.cloudflare.com", "--ipv6", "--format", "text"])
        .assert()
        .success()
        .stdout(contains("IP:").and(contains("v6")));
}