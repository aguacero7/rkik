#![cfg(all(feature = "ptp", target_os = "linux"))]

use chrono::{DateTime, Local, Utc};
use console::set_colors_enabled;
use rkik::{
    ClockIdentity, ClockQuality, PacketStats, PortIdentity, PtpDiagnostics, PtpProbeResult,
    PtpTarget, TimeSource, fmt, stats::compute_ptp_stats,
};
use std::net::IpAddr;

fn sample_probe(offset_ns: i64, delay_ns: i64, include_diag: bool) -> PtpProbeResult {
    let utc: DateTime<Utc> = DateTime::from_timestamp(1_700_000_000, 123_000_000).unwrap();
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
    let probes = vec![
        sample_probe(1_000, 5_000, false),
        sample_probe(-500, 7_000, false),
    ];
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

#[test]
fn ptp_stats_handle_empty_input() {
    let stats = compute_ptp_stats(&[]);
    assert_eq!(stats.count, 0);
    assert_eq!(stats.offset_avg_ns, 0.0);
    assert_eq!(stats.offset_min_ns, 0.0);
    assert_eq!(stats.offset_max_ns, 0.0);
    assert_eq!(stats.mean_path_delay_avg_ns, 0.0);
}

#[test]
fn ptp_text_compare_render_includes_offsets() {
    set_colors_enabled(false);
    let probes = vec![
        sample_probe(1_000, 5_000, false),
        sample_probe(-500, 7_000, false),
    ];
    let rendered = fmt::ptp_text::render_compare(&probes, false);
    assert!(
        rendered.contains("Comparing PTP"),
        "expected compare header in '{}'",
        rendered
    );
    assert!(
        rendered.contains("1000 ns") || rendered.contains("-500 ns"),
        "expected offset values in '{}'",
        rendered
    );
}

#[test]
fn ptp_text_short_compare_render_includes_domain_and_offset() {
    set_colors_enabled(false);
    let probes = vec![
        sample_probe(1_000, 5_000, false),
        sample_probe(-500, 7_000, false),
    ];
    let rendered = fmt::ptp_text::render_short_compare(&probes);
    assert!(
        rendered.contains("lab-master:24"),
        "expected domain information in '{}'",
        rendered
    );
    assert!(
        rendered.contains("1000ns") || rendered.contains("-500ns"),
        "expected offset values in '{}'",
        rendered
    );
}

#[test]
fn ptp_text_stats_render_includes_key_numbers() {
    set_colors_enabled(false);
    let probes = vec![
        sample_probe(1_000, 5_000, false),
        sample_probe(-500, 7_000, false),
    ];
    let stats = compute_ptp_stats(&probes);
    let rendered = fmt::ptp_text::render_stats("lab-master", &stats);
    assert!(
        rendered.contains("avg 250"),
        "expected average offset in '{}'",
        rendered
    );
    assert!(
        rendered.contains("min -500"),
        "expected min offset in '{}'",
        rendered
    );
    assert!(
        rendered.contains("max 1000"),
        "expected max offset in '{}'",
        rendered
    );
    assert!(
        rendered.contains("2 samples"),
        "expected sample count in '{}'",
        rendered
    );
}
