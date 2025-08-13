//! One-shot system clock synchronization helpers (feature = "sync").
//! Force a STEP to server UTC + half RTT. Big jumps allowed. Unix-only.

use crate::ProbeResult;
use chrono::{DateTime, Duration, Utc};
use std::io;

#[derive(Debug)]
pub enum SyncError {
    NotSupported,
    Permission(io::Error),
    Sys(io::Error),
}

/// Compute target UTC (server UTC + RTT/2) and step system clock.
pub fn sync_from_probe(probe: &ProbeResult) -> Result<(), SyncError> {
    let half_rtt_ns = (probe.rtt_ms * 1_000_000.0 / 2.0).round() as i64 * 1000;
    let target = probe.utc + Duration::nanoseconds(half_rtt_ns);
    step_to_utc(&target)
}

#[cfg(unix)]
fn step_to_utc(utc: &DateTime<Utc>) -> Result<(), SyncError> {
    use libc::{clock_settime, timespec, CLOCK_REALTIME};

    unsafe {
        if libc::geteuid() != 0 {
            return Err(SyncError::Permission(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "need root or CAP_SYS_TIME",
            )));
        }
    }
    let ts = timespec {
        tv_sec: utc.timestamp() as libc::time_t,
        tv_nsec: utc.timestamp_subsec_nanos() as libc::c_long,
    };
    let rc = unsafe { clock_settime(CLOCK_REALTIME, &ts as *const timespec) };
    if rc != 0 {
        return Err(SyncError::Sys(std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(not(unix))]
fn step_to_utc(_: &DateTime<Utc>) -> Result<(), SyncError> {
    Err(SyncError::NotSupported)
}
