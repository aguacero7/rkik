pub mod ntp_client;
pub mod nts_client;
#[cfg(all(feature = "ptp", target_os = "linux"))]
pub mod ptp_client;
pub mod resolver;
