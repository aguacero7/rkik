pub mod ntp;

#[cfg(all(feature = "ptp", target_os = "linux"))]
pub mod ptp;
