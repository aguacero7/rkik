use futures::future::join_all;
use std::net::{IpAddr, SocketAddr};

use crate::errors::RkikError;
use crate::ntp::{client, resolver};

/// Compare multiple servers and return a vector of (name, ip, offset_ms)
pub async fn compare(
    servers: &[String],
    ipv6: bool,
) -> Result<Vec<(String, IpAddr, f64)>, RkikError> {
    if servers.len() < 2 {
        return Err(RkikError::General(
            "Need at least 2 servers to compare".into(),
        ));
    }

    let resolved: Vec<_> = servers
        .iter()
        .map(|s| (s.clone(), resolver::resolve_ip(s, ipv6)))
        .collect();

    let valid: Vec<_> = resolved
        .into_iter()
        .filter_map(|(name, ip)| match ip {
            Ok(addr) => Some((name, addr)),
            Err(_) => None,
        })
        .collect();

    if valid.len() < 2 {
        return Err(RkikError::General(
            "Not enough valid servers to compare.".into(),
        ));
    }

    let client = client::async_client(ipv6).await;
    let futures = valid
        .iter()
        .map(|(_, ip)| client.synchronize(SocketAddr::new(*ip, 123).to_string()))
        .collect::<Vec<_>>();

    let results = join_all(futures).await;

    let mut final_results = vec![];
    for ((name, ip), res) in valid.into_iter().zip(results) {
        if let Ok(r) = res {
            let offset = r.clock_offset().as_secs_f64() * 1000.0;
            final_results.push((name, ip, offset));
        }
    }

    if final_results.len() < 2 {
        return Err(RkikError::General(
            "At least two successful responses required to compare.".into(),
        ));
    }

    Ok(final_results)
}
