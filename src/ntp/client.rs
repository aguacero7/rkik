use rsntp::{
    AsyncSntpClient, Config, ReferenceIdentifier, SntpClient, SynchronizationError,
    SynchronizationResult,
};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use crate::errors::RkikError;

pub fn client_for_mode(ipv6: bool) -> SntpClient {
    if ipv6 {
        let config = Config::default().bind_address((Ipv6Addr::UNSPECIFIED, 0).into());
        SntpClient::with_config(config)
    } else {
        let config = Config::default().bind_address(([0, 0, 0, 0], 0).into());
        SntpClient::with_config(config)
    }
}

pub async fn async_client(ipv6: bool) -> AsyncSntpClient {
    let config = if ipv6 {
        Config::default().bind_address((Ipv6Addr::UNSPECIFIED, 0).into())
    } else {
        Config::default().bind_address(([0, 0, 0, 0], 0).into())
    };
    AsyncSntpClient::with_config(config)
}

pub fn synchronize_with_ip(
    client: &SntpClient,
    ip: IpAddr,
) -> Result<SynchronizationResult, SynchronizationError> {
    let addr = SocketAddr::new(ip, 123);
    client.synchronize(addr.to_string())
}

pub fn format_reference_id(reference_id: &ReferenceIdentifier) -> String {
    reference_id.to_string()
}

pub fn sync(ip: IpAddr, ipv6: bool) -> Result<SynchronizationResult, RkikError> {
    let client = client_for_mode(ipv6);
    synchronize_with_ip(&client, ip).map_err(|e| RkikError::SyncError(e.to_string()))
}

pub async fn async_sync(ip: IpAddr, ipv6: bool) -> Result<SynchronizationResult, RkikError> {
    let client = async_client(ipv6).await;
    let addr = SocketAddr::new(ip, 123);
    client
        .synchronize(addr.to_string())
        .await
        .map_err(|e| RkikError::SyncError(e.to_string()))
}
