use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use rsntp::{AsyncSntpClient, Config, SynchronizationResult};

use crate::error::RkikError;

/// Query an NTP server asynchronously and return the synchronization result.
///
/// The local socket is bound in the address family of `ip`; `_ipv6` is kept
/// for API compatibility and no longer influences the bind address.
pub async fn query(
    ip: IpAddr,
    _ipv6: bool,
    timeout: Duration,
    port: u16,
) -> Result<SynchronizationResult, RkikError> {
    let cfg = if ip.is_ipv6() {
        Config::default().bind_address((Ipv6Addr::UNSPECIFIED, 0).into())
    } else {
        Config::default().bind_address(([0, 0, 0, 0], 0).into())
    };
    let client = AsyncSntpClient::with_config(cfg);
    // rsntp does not expose explicit timeout; rely on tokio timeout
    let fut = client.synchronize(SocketAddr::new(ip, port));
    let res = tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| RkikError::Network("timeout".into()))??;
    Ok(res)
}
