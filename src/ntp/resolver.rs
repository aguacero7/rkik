use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::errors::RkikError;

/// Resolve the given host to an IP address. If `ipv6_only` is true,
/// prefer IPv6 addresses. Otherwise prefer IPv4.
pub fn resolve_ip(host: &str, ipv6_only: bool) -> Result<IpAddr, RkikError> {
    let port = 123;
    let addrs: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|e| {
            RkikError::ResolveError(format!("DNS resolution failed for '{}': {}", host, e))
        })?
        .collect();

    let filtered: Vec<IpAddr> = if ipv6_only {
        addrs
            .iter()
            .map(|a| a.ip())
            .filter(|ip| ip.is_ipv6())
            .collect()
    } else {
        let mut v4 = vec![];
        let mut v6 = vec![];
        for a in addrs {
            let ip = a.ip();
            if ip.is_ipv4() {
                v4.push(ip);
            } else {
                v6.push(ip);
            }
        }
        v4.into_iter().chain(v6).collect()
    };

    filtered.into_iter().next().ok_or_else(|| {
        if ipv6_only {
            RkikError::ResolveError(format!("No IPv6 address found for '{}'", host))
        } else {
            RkikError::ResolveError(format!("No IP address found for '{}'", host))
        }
    })
}
