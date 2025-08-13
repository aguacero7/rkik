use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::error::RkikError;

/// Resolve the IP address for a host name according to IPv4/IPv6 mode.
pub fn resolve_ip(target: &str, ipv6_only: bool) -> Result<IpAddr, RkikError> {
    let port = 123;
    let addrs: Vec<SocketAddr> = (target, port)
        .to_socket_addrs()
        .map_err(|e| RkikError::Dns(format!("{}", e)))?
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
            RkikError::Dns(format!("No IPv6 address found for '{}'", target))
        } else {
            RkikError::Dns(format!("No IP address found for '{}'", target))
        }
    })
}
