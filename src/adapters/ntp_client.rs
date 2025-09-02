use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;

use rsntp::{AsyncSntpClient, Config, SynchronizationResult};

use crate::error::RkikError;

/// Query an NTP server asynchronously and return the synchronization result.
pub async fn query(
    ip: IpAddr,
    ipv6: bool,
    timeout: Duration,
    port: i8
) -> Result<SynchronizationResult, RkikError> {
    let cfg = if ipv6 {
        Config::default().bind_address((Ipv6Addr::UNSPECIFIED, 0).into())
    } else {
        Config::default().bind_address(([0, 0, 0, 0], 0).into())
    };
    let client = AsyncSntpClient::with_config(cfg);
    // rsntp does not expose explicit timeout; rely on tokio timeout
    let addr = format!("{}:{}", ip, port);
    let fut = client.synchronize(addr);
    let res = tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| RkikError::Network("timeout".into()))??;
    Ok(res)
}
