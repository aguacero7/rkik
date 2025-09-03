use rkik::{ProbeResult, parse_target};
use std::time::Duration;

#[tokio::test]
async fn test_query_invalid_host() {
    let err = rkik::query_one("no.such.domain.example", false, Duration::from_secs(1))
        .await
        .expect_err("expected error");
    assert!(matches!(err, rkik::RkikError::Dns(_)));
}
#[test]
fn parse_hostname_no_port() {
    let p = parse_target("pool.ntp.org").unwrap();
    assert_eq!(p.host, "pool.ntp.org");
    assert_eq!(p.port, None);
    assert!(!p.is_ipv6_literal);
}

#[test]
fn parse_hostname_with_port() {
    let p = parse_target("pool.ntp.org:123").unwrap();
    assert_eq!(p.host, "pool.ntp.org");
    assert_eq!(p.port, Some(123));
    assert!(!p.is_ipv6_literal);
}

#[test]
fn parse_ipv4_with_port() {
    let p = parse_target("8.8.8.8:4242").unwrap();
    assert_eq!(p.host, "8.8.8.8");
    assert_eq!(p.port, Some(4242));
    assert!(!p.is_ipv6_literal);
}

#[test]
fn parse_ipv6_bracketed_no_port() {
    let p = parse_target("[2001:db8::1]").unwrap();
    assert_eq!(p.host, "2001:db8::1");
    assert_eq!(p.port, None);
    assert!(p.is_ipv6_literal);
}

#[test]
fn parse_ipv6_bracketed_with_port() {
    let p = parse_target("[2001:db8::1]:123").unwrap();
    assert_eq!(p.host, "2001:db8::1");
    assert_eq!(p.port, Some(123));
    assert!(p.is_ipv6_literal);
}

#[test]
fn parse_ipv6_bare_no_port() {
    let p = parse_target("2001:db8::1").unwrap();
    assert_eq!(p.host, "2001:db8::1");
    assert_eq!(p.port, None);
    assert!(p.is_ipv6_literal);
}

#[test]
fn reject_ipv6_bare_with_colon_port_like() {
    // This would be ambiguous; require brackets for IPv6 with port.
    assert!(parse_target("2001:db8::1:123").is_err());
}

#[test]
fn reject_port_out_of_range() {
    assert!(parse_target("pool.ntp.org:70000").is_err());
    assert!(parse_target("[2001:db8::1]:0").is_err());
}
