use assert_cmd::Command;
use predicates::str::contains;
#[cfg(feature = "network-tests")]
use rkik::adapters::resolver::resolve_ip;
use std::fs;
use std::path::PathBuf;

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

#[test]
fn test_ntp_subcommand_help() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.args(["ntp", "--help"])
        .assert()
        .success()
        .stdout(contains("Run a standard NTP probe"));
}

#[test]
fn test_help_command_without_subcommand() {
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.arg("help")
        .assert()
        .success()
        .stdout(contains("Usage: rkik"));
}

#[test]
fn test_config_path_respects_env_override() {
    let dir = config_test_dir("config-path");
    let mut cmd = Command::cargo_bin("rkik").unwrap();
    cmd.env("RKIK_CONFIG_DIR", dir.to_string_lossy().as_ref())
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(contains(dir.to_string_lossy().as_ref()));
}

#[test]
fn test_preset_add_and_list() {
    let dir = config_test_dir("preset-add");
    let mut add = Command::cargo_bin("rkik").unwrap();
    add.env("RKIK_CONFIG_DIR", dir.to_string_lossy().as_ref())
        .args(["preset", "add", "nightly", "--", "ntp", "pool.ntp.org"])
        .assert()
        .success();

    let mut list = Command::cargo_bin("rkik").unwrap();
    list.env("RKIK_CONFIG_DIR", dir.to_string_lossy().as_ref())
        .args(["preset", "list"])
        .assert()
        .success()
        .stdout(contains("nightly"));
}

fn config_test_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("rkik-test-{name}"));
    let _ = fs::remove_dir_all(&path);
    path
}
