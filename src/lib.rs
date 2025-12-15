//! rkik library exposing reusable NTP querying utilities.

pub mod adapters;
pub mod domain;
mod error;
pub mod fmt;
pub mod services;
pub mod stats;

pub use domain::ntp::{ProbeResult, Target};
#[cfg(all(feature = "ptp", target_os = "linux"))]
pub use domain::ptp::{
    ClockIdentity, ClockQuality, PacketStats, PortIdentity, PtpDiagnostics, PtpProbeResult,
    PtpTarget, TimeSource,
};
pub use error::RkikError;
pub use services::compare::compare_many;
#[cfg(all(feature = "ptp", target_os = "linux"))]
pub use services::ptp_query::{
    PtpQueryOptions, query_many as query_many_ptp, query_target as query_one_ptp,
};
pub use services::query::query_one;

#[cfg(feature = "sync")]
pub mod sync;
