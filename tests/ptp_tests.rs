#![cfg(feature = "ptp")]

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use console::set_colors_enabled;
use rkik::{
    fmt,
    stats::compute_ptp_stats,
    ClockIdentity,
    ClockQuality,
    PacketStats,
    PortIdentity,
    PtpDiagnostics,
    PtpProbeResult,
    PtpTarget,
    TimeSource,
};
use std::net::IpAddr;

fn sample_probe(offset_ns: i64, delay_ns: i64, include_diag: bool) -> PtpProbeResult {
    let naive = NaiveDateTime::from_timestamp_opt(1_700_000_000, 123_000_000).unwrap();
    let utc: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let local: DateTime<Local> = DateTime::from(utc);
    let ip: IpAddr = "127.0.0.1".parse().unwrap();

    let diagnostics = include_diag.then(|| PtpDiagnostics {
        master_port_identity: PortIdentity {
            clock_identity: ClockIdentity([0, 1, 2, 3, 4, 5, 6, 7]),
            port_number: 1,
        },
        hardware_timestamping: true,
        timestamp_mode: "hw".to_string(),
        steps_removed: 0,
        current_utc_offset: 37,
        current_utc_offset_valid: true,
        leap59: false,
        leap61: false,
        time_traceable: true,
        frequency_traceable: true,
        ptp_timescale: true,
        packet_stats: PacketStats {
            sync_sent: 1,
            sync_received: 2,
            follow_up_received: 3,
            delay_req_sent: 4,
            delay_resp_received: 5,
            announce_received: 6,
        },
        measurement_duration_ms: 1.5,
    });

    PtpProbeResult {
        target: PtpTarget {
            name: "lab-master".into(),
            ip,
            domain: 24,
            event_port: 3319,
            general_port: 3320,
        },
        offset_ns,
        mean_path_delay_ns: delay_ns,
        master_identity: ClockIdentity([0, 1, 2, 3, 4, 5, 6, 7]),
        clock_quality: ClockQuality {
            clock_class: 6,
            clock_accuracy: 0x20,
            offset_scaled_log_variance: 0x100,
        },
        time_source: TimeSource::Gps,
        utc,
        local,
        timestamp: utc.timestamp(),
        diagnostics,
    }
}

#[test]
fn ptp_stats_report_expected_values() {
    let probes = vec![sample_probe(1_000, 5_000, false), sample_probe(-500, 7_000, false)];
    let stats = compute_ptp_stats(&probes);
    assert_eq!(stats.count, 2);
    assert!((stats.offset_avg_ns - 250.0).abs() < f64::EPSILON);
    assert_eq!(stats.offset_min_ns, -500.0);
    assert_eq!(stats.offset_max_ns, 1_000.0);
    assert_eq!(stats.mean_path_delay_avg_ns, 6_000.0);
}

#[test]
fn ptp_text_short_render_includes_domain() {
    set_colors_enabled(false);
    let probe = sample_probe(1_000, 5_000, false);
    let rendered = fmt::ptp_text::render_short_probe(&probe);
    assert!(
        rendered.contains("lab-master:24"),
        "expected domain in '{}'",
        rendered
    );
    assert!(
        rendered.contains("1000 ns"),
        "expected offset in '{}'",
        rendered
    );
}

#[cfg(feature = "json")]
#[test]
fn ptp_json_renderer_includes_protocol_and_diagnostics() {
    let probe = sample_probe(2_000, 8_000, true);
    let json = fmt::ptp_json::to_json(&[probe], true, true).expect("json serialization");
    assert!(
        json.contains("\"protocol\":\"ptp\""),
        "missing protocol field"
    );
    assert!(
        json.contains("\"diagnostics\""),
        "missing diagnostics block"
    );
}
