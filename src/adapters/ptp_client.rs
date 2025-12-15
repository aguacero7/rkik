//! Simulated PTP client adapter.
//!
//! The real IEEE 1588 implementation would require tight hardware integration
//! and kernel timestamping support. For the purposes of rkik we provide a fast,
//! deterministic probe that mirrors the shape of PTP data so the rest of the
//! application can be exercised.

#![cfg(all(feature = "ptp", target_os = "linux"))]

use chrono::{Local, Utc};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::time;

use crate::domain::ptp::{
    ClockIdentity, ClockQuality, PacketStats, PortIdentity, PtpDiagnostics, PtpProbeResult,
    PtpTarget, TimeSource,
};
use crate::error::RkikError;

/// Query a PTP master and return a simulated [`PtpProbeResult`].
pub async fn query_ptp(
    target_name: &str,
    ip: IpAddr,
    domain: u8,
    event_port: u16,
    general_port: u16,
    hw_timestamping: bool,
    timeout: Duration,
    verbose: bool,
) -> Result<PtpProbeResult, RkikError> {
    // The simulated probe finishes quickly, but we still respect the caller-provided timeout.
    time::timeout(timeout, async move {
        let start = Instant::now();
        let seed = build_seed(
            target_name,
            ip,
            domain,
            event_port,
            general_port,
            hw_timestamping,
        );

        let master_identity = derive_clock_identity(seed);
        let master_port = PortIdentity {
            clock_identity: master_identity,
            port_number: 1,
        };
        let clock_quality = derive_clock_quality(seed);
        let time_source = derive_time_source(seed);
        let offset_ns = derive_offset(seed);
        let mean_path_delay_ns = derive_path_delay(seed);

        let utc = Utc::now();
        let local = Local::now();
        let timestamp = utc.timestamp();

        let diagnostics = verbose.then(|| PtpDiagnostics {
            master_port_identity: master_port,
            hardware_timestamping: hw_timestamping,
            timestamp_mode: if hw_timestamping {
                "hardware timestamping (simulated)".to_string()
            } else {
                "software timestamping (simulated)".to_string()
            },
            steps_removed: ((seed >> 3) % 4) as u16,
            current_utc_offset: 37,
            current_utc_offset_valid: true,
            leap59: false,
            leap61: false,
            time_traceable: (seed & 0x1) == 0,
            frequency_traceable: (seed & 0x2) == 0,
            ptp_timescale: true,
            packet_stats: derive_packet_stats(seed),
            measurement_duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        });

        PtpProbeResult {
            target: PtpTarget {
                name: target_name.to_string(),
                ip,
                domain,
                event_port,
                general_port,
            },
            offset_ns,
            mean_path_delay_ns,
            master_identity,
            clock_quality,
            time_source,
            utc,
            local,
            timestamp,
            diagnostics,
        }
    })
    .await
    .map_err(|_| RkikError::Other(format!("ptp query timed out after {:?}", timeout)))
}

fn build_seed(
    name: &str,
    ip: IpAddr,
    domain: u8,
    event_port: u16,
    general_port: u16,
    hw_timestamping: bool,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    ip.hash(&mut hasher);
    domain.hash(&mut hasher);
    event_port.hash(&mut hasher);
    general_port.hash(&mut hasher);
    hw_timestamping.hash(&mut hasher);
    hasher.finish()
}

fn derive_clock_identity(seed: u64) -> ClockIdentity {
    let mut bytes = [0u8; 8];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = ((seed >> (i * 8)) & 0xFF) as u8;
    }
    ClockIdentity(bytes)
}

fn derive_clock_quality(seed: u64) -> ClockQuality {
    ClockQuality {
        clock_class: match seed & 0x7 {
            0 => 6,
            1 => 7,
            2 => 13,
            3 => 52,
            4 => 58,
            5 => 187,
            6 => 248,
            _ => 255,
        },
        clock_accuracy: 0x20 + ((seed >> 8) as u8 % 0x10),
        offset_scaled_log_variance: ((seed >> 16) as u16) | 0x0100,
    }
}

fn derive_time_source(seed: u64) -> TimeSource {
    match seed % 7 {
        0 => TimeSource::AtomicClock,
        1 => TimeSource::Gps,
        2 => TimeSource::TerrestrialRadio,
        3 => TimeSource::Ptp,
        4 => TimeSource::Ntp,
        5 => TimeSource::HandSet,
        _ => TimeSource::InternalOscillator,
    }
}

fn derive_offset(seed: u64) -> i64 {
    let range = 200_000i64; // +/- 2 ms after converting to nanoseconds
    let raw = (seed as i64 % (range * 2)) - range;
    raw * 10 // convert to nanoseconds
}

fn derive_path_delay(seed: u64) -> i64 {
    let base = ((seed >> 11) % 50_000) as i64; // up to 50 us
    (base + 1_000) * 10 // ensure non-zero
}

fn derive_packet_stats(seed: u64) -> PacketStats {
    PacketStats {
        sync_sent: ((seed >> 5) % 3) as u32,
        sync_received: ((seed >> 8) % 10) as u32 + 1,
        follow_up_received: ((seed >> 10) % 8) as u32,
        delay_req_sent: ((seed >> 12) % 5) as u32 + 1,
        delay_resp_received: ((seed >> 14) % 5) as u32 + 1,
        announce_received: ((seed >> 16) % 3) as u32,
    }
}
