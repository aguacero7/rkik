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
pub fn sync_from_probe(probe: &ProbeResult, dry_run: bool) -> Result<(), SyncError> {
    let offset_us = (probe.offset_ms * 1000.0).round() as i64; // ms -> µs
    let target = Utc::now() + Duration::microseconds(offset_us);
    step_to_utc(&target, dry_run)
}

pub fn get_sys_permissions() -> bool {
    #[cfg(unix)]
    unsafe {
        if libc::geteuid() != 0 {
            return false;
        }
    }
    return true;
}

#[cfg(unix)]
fn step_to_utc(utc: &DateTime<Utc>, dry_run: bool) -> Result<(), SyncError> {
    use libc::{CLOCK_REALTIME, clock_settime, timespec};

    if dry_run {
        return Ok(());
    }
    let ts = timespec {
        tv_sec: utc.timestamp() as libc::time_t,
        tv_nsec: utc.timestamp_subsec_nanos() as libc::c_long,
    };
    let rc = unsafe { clock_settime(CLOCK_REALTIME, &ts as *const timespec) };
    if rc != 0 {
        let e = std::io::Error::last_os_error();
        return Err(match e.raw_os_error() {
            Some(code) if code == libc::EPERM || code == libc::EACCES => SyncError::Permission(e),
            _ => SyncError::Sys(e),
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn step_to_utc(_: &DateTime<Utc>, _: bool) -> Result<(), SyncError> {
    Err(SyncError::NotSupported)
}
