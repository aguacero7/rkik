use console::{Term, style};

use crate::ntp::model::NtpResult;

/// Print query result in text format
pub fn print_text(term: &Term, server: &str, result: &NtpResult, verbose: bool) {
    let ip_version = if result.ip.is_ipv6() { "v6" } else { "v4" };
    term.write_line(&format!(
        "{} {}",
        style("Server:").cyan().bold(),
        style(server).green()
    ))
    .unwrap();
    term.write_line(&format!(
        "{} {} ({})",
        style("IP:").cyan().bold(),
        style(result.ip).green(),
        ip_version
    ))
    .unwrap();
    term.write_line(&format!(
        "{} {}",
        style("UTC Time:").cyan().bold(),
        style(result.datetime_utc.to_rfc2822()).green()
    ))
    .unwrap();
    term.write_line(&format!(
        "{} {}",
        style("Local Time:").cyan().bold(),
        style(result.local_time.format("%Y-%m-%d %H:%M:%S")).green()
    ))
    .unwrap();
    term.write_line(&format!(
        "{} {:.3} ms",
        style("Clock Offset:").cyan().bold(),
        result.offset_ms
    ))
    .unwrap();
    term.write_line(&format!(
        "{} {:.3} ms",
        style("Round Trip Delay:").cyan().bold(),
        result.rtt_ms
    ))
    .unwrap();
    if verbose {
        term.write_line(&format!(
            "{} {}",
            style("Stratum:").cyan().bold(),
            result.stratum
        ))
        .unwrap();
        term.write_line(&format!(
            "{} {}",
            style("Reference ID:").cyan().bold(),
            result.reference_id
        ))
        .unwrap();
    }
}

/// Print query result in JSON format
pub fn print_json(result: &NtpResult, server: &str) {
    let ip_version = if result.ip.is_ipv6() { "v6" } else { "v4" };
    println!(
        "{{\"server\": \"{}\", \"ip\": \"{}\", \"ip_version\": \"{}\", \"utc_time\": \"{}\", \"local_time\": \"{}\", \"offset_ms\": {:.3}, \"rtt_ms\": {:.3}, \"stratum\": {}, \"reference_id\": \"{}\"}}",
        server,
        result.ip,
        ip_version,
        result.datetime_utc.to_rfc3339(),
        result.local_time.format("%Y-%m-%d %H:%M:%S"),
        result.offset_ms,
        result.rtt_ms,
        result.stratum,
        result.reference_id
    );
}

/// Print comparison results in text format
pub fn print_compare_text(term: &Term, results: &[(String, IpAddr, f64)]) {
    term.write_line(&format!(
        "{} {} servers",
        style("Comparing (async):").bold(),
        results.len()
    ))
    .unwrap();

    for (name, ip, offset) in results.iter() {
        let ip_version = if ip.is_ipv6() { "v6" } else { "v4" };
        term.write_line(&format!(
            "{} [{} {}]: {:.3} ms",
            style(name).green(),
            ip,
            ip_version,
            offset
        ))
        .unwrap();
    }

    let min = results
        .iter()
        .map(|(_, _, o)| *o)
        .fold(f64::INFINITY, f64::min);
    let max = results
        .iter()
        .map(|(_, _, o)| *o)
        .fold(f64::NEG_INFINITY, f64::max);
    let avg = results.iter().map(|(_, _, o)| *o).sum::<f64>() / results.len() as f64;
    let diff = max - min;

    term.write_line(&format!(
        "{} {:.3} ms (min: {:.3}, max: {:.3}, avg: {:.3})",
        style("Max drift:").cyan().bold(),
        diff,
        min,
        max,
        avg
    ))
    .unwrap();
}

use std::net::IpAddr;

/// Print comparison results in JSON format
pub fn print_compare_json(results: &[(String, IpAddr, f64)]) {
    println!("[");
    for (i, (name, ip, offset)) in results.iter().enumerate() {
        println!(
            "  {{ \"server\": \"{}\", \"ip\": \"{}\", \"offset_ms\": {:.3} }}{}",
            name,
            ip,
            offset,
            if i < results.len() - 1 { "," } else { "" }
        );
    }
    println!("]");
}
