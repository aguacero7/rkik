#![cfg(feature = "ptp")]

use chrono::SecondsFormat;
use console::style;

use crate::domain::ptp::PtpProbeResult;
use crate::stats::PtpStats;

fn format_ns(value: i64) -> String {
    format!("{} ns ({:.3} us)", value, value as f64 / 1_000.0)
}

/// Render a full verbose/succinct PTP probe.
pub fn render_probe(result: &PtpProbeResult, verbose: bool) -> String {
    let mut out = format!(
        "{srv_lbl} {srv_val}\n\
         {ip_lbl} {ip}:{event}/{general}\n\
         {dom_lbl} {domain}\n\
         {utc_lbl} {utc}\n\
         {loc_lbl} {local}\n\
         {off_lbl} {offset}\n\
         {delay_lbl} {delay}\n\
         {master_lbl} {master}\n\
         {class_lbl} {class} ({class_desc})\n\
         {acc_lbl} {acc} ({acc_desc})\n\
         {src_lbl} {source}",
        srv_lbl = style("Server:").cyan().bold(),
        srv_val = style(&result.target.name).green(),
        ip_lbl = style("IP:").cyan().bold(),
        ip = style(result.target.ip).green(),
        event = style(result.target.event_port).green(),
        general = style(result.target.general_port).green(),
        dom_lbl = style("Domain:").cyan().bold(),
        domain = result.target.domain,
        utc_lbl = style("UTC Time:").cyan().bold(),
        utc = style(result.utc.to_rfc3339_opts(SecondsFormat::Nanos, true)).green(),
        loc_lbl = style("Local Time:").cyan().bold(),
        local = style(result.local.to_string()).green(),
        off_lbl = style("Clock Offset:").cyan().bold(),
        offset = format_ns(result.offset_ns),
        delay_lbl = style("Mean Path Delay:").cyan().bold(),
        delay = format_ns(result.mean_path_delay_ns),
        master_lbl = style("Master Clock:").cyan().bold(),
        master = style(result.master_identity).green(),
        class_lbl = style("Clock Class:").cyan().bold(),
        class = result.clock_quality.clock_class,
        class_desc = result.clock_quality.class_description(),
        acc_lbl = style("Clock Accuracy:").cyan().bold(),
        acc = format!("0x{:02X}", result.clock_quality.clock_accuracy),
        acc_desc = result.clock_quality.accuracy_description(),
        src_lbl = style("Time Source:").cyan().bold(),
        source = style(result.time_source).green(),
    );

    if let Some(diag) = &result.diagnostics {
        out.push_str(&format!(
            "\n\n{diag_hdr}\n{port_lbl} {port}\n{ts_lbl} {ts_mode}\n{hw_lbl} {hw}\n\
             {steps_lbl} {steps}\n{utc_off_lbl} {offset}s (valid: {valid})\n\
             {trace_lbl} time={time}, freq={freq}\n{pkt_hdr}\n  Sync RX: {sync_rx}\n  Delay Resp RX: {delay_rx}\n  Announce RX: {ann_rx}\n  Delay Req TX: {delay_tx}\n{meas_lbl} {meas:.3} ms",
            diag_hdr = style("=== PTP Diagnostics ===").cyan().bold().underlined(),
            port_lbl = style("Master Port:").cyan().bold(),
            port = style(diag.master_port_identity).green(),
            ts_lbl = style("Timestamp Mode:").cyan().bold(),
            ts_mode = style(&diag.timestamp_mode).green(),
            hw_lbl = style("Hardware Timestamping:").cyan().bold(),
            hw = style(if diag.hardware_timestamping { "Yes" } else { "No" }).green(),
            steps_lbl = style("Steps Removed:").cyan().bold(),
            steps = diag.steps_removed,
            utc_off_lbl = style("Current UTC Offset:").cyan().bold(),
            offset = diag.current_utc_offset,
            valid = diag.current_utc_offset_valid,
            trace_lbl = style("Traceable:").cyan().bold(),
            time = diag.time_traceable,
            freq = diag.frequency_traceable,
            pkt_hdr = style("Packet Statistics:").cyan().bold(),
            sync_rx = diag.packet_stats.sync_received,
            delay_rx = diag.packet_stats.delay_resp_received,
            ann_rx = diag.packet_stats.announce_received,
            delay_tx = diag.packet_stats.delay_req_sent,
            meas_lbl = style("Measurement Duration:").cyan().bold(),
            meas = diag.measurement_duration_ms,
        ));
    } else if verbose {
        out.push_str("\n\nDiagnostics unavailable (run with --verbose to enable detailed stats).");
    }

    out
}

/// Render multiple PTP probes for compare mode.
pub fn render_compare(results: &[PtpProbeResult], verbose: bool) -> String {
    let mut out = String::new();
    if results.len() == 2 {
        out.push_str(&format!(
            "{} {} and {}\n",
            style("Comparing PTP:").bold(),
            style(&results[0].target.name).green(),
            style(&results[1].target.name).green()
        ));
    } else {
        out.push_str(&format!(
            "{} {} masters\n",
            style("Comparing PTP (async):").bold(),
            results.len()
        ));
    }

    for r in results {
        out.push_str(&format!(
            "{name} [{ip}] -> {offset}\n",
            name = style(&r.target.name).green().bold(),
            ip = style(r.target.ip).cyan(),
            offset = style(format_ns(r.offset_ns)).yellow()
        ));
        if verbose {
            out.push_str(&format!(
                "  {} {}\n  {} {}\n",
                style("Master:").cyan().bold(),
                r.master_identity,
                style("Time Source:").cyan().bold(),
                r.time_source,
            ));
        }
    }

    out
}

/// Render a single-line summary for repeated probes.
pub fn render_short_probe(result: &PtpProbeResult) -> String {
    format!(
        "{name}:{domain} {offset}",
        name = style(&result.target.name).green(),
        domain = result.target.domain,
        offset = style(format!("{} ns", result.offset_ns)).yellow()
    )
}

/// Render simple multi-line output.
pub fn render_simple_probe(result: &PtpProbeResult) -> String {
    format!(
        "{name}:{domain} {offset} delay {delay}",
        name = style(&result.target.name).green(),
        domain = style(result.target.domain).green(),
        offset = style(format!("{} ns", result.offset_ns)).yellow(),
        delay = style(format!("{} ns", result.mean_path_delay_ns)).cyan()
    )
}

/// Render statistics for repeated PTP probes.
pub fn render_stats(name: &str, stats: &PtpStats) -> String {
    format!(
        "\n{n}: avg {avg:.0} ns (min {min:.0}, max {max:.0}) delay avg {delay:.0} ns ({cnt} samples)",
        n = style(name).green().bold(),
        avg = stats.offset_avg_ns,
        min = stats.offset_min_ns,
        max = stats.offset_max_ns,
        delay = stats.mean_path_delay_avg_ns,
        cnt = stats.count
    )
}

/// Render compact compare output.
pub fn render_short_compare(results: &[PtpProbeResult]) -> String {
    results
        .iter()
        .map(|r| {
            format!(
                "{name}:{domain}:{offset}",
                name = style(&r.target.name).green(),
                domain = r.target.domain,
                offset = style(format!("{}ns", r.offset_ns)).yellow()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render simple compare output (one per line).
pub fn render_simple_compare(results: &[PtpProbeResult]) -> String {
    results
        .iter()
        .map(render_simple_probe)
        .collect::<Vec<_>>()
        .join("\n")
}
