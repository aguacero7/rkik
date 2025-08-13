use crate::domain::ntp::ProbeResult;
use console::style;

/// Render a probe result into human readable text with the legacy style.
pub fn render_probe(r: &ProbeResult, verbose: bool) -> String {
    let ip_version = if r.target.ip.is_ipv6() { "v6" } else { "v4" };

    // bloc principal (strictement identique au rendu legacy)
    let mut out = format!(
        "{srv_lbl} {srv_val}\n\
         {ip_lbl} {ip_val} ({ver})\n\
         {utc_lbl} {utc_val}\n\
         {loc_lbl} {loc_val}\n\
         {off_lbl} {off_val:.3} ms\n\
         {rtt_lbl} {rtt_val:.3} ms",
        srv_lbl = style("Server:").cyan().bold(),
        srv_val = style(&r.target.name).green(),
        ip_lbl  = style("IP:").cyan().bold(),
        ip_val  = style(r.target.ip).green(),
        ver     = ip_version,
        utc_lbl = style("UTC Time:").cyan().bold(),
        utc_val = style(r.utc.to_rfc2822()).green(),
        loc_lbl = style("Local Time:").cyan().bold(),
        loc_val = style(r.local.format("%Y-%m-%d %H:%M:%S")).green(),
        off_lbl = style("Clock Offset:").cyan().bold(),
        off_val = r.offset_ms,
        rtt_lbl = style("Round Trip Delay:").cyan().bold(),
        rtt_val = r.rtt_ms,
    );

    // bloc verbose : lignes additionnelles Stratum / Reference ID
    if verbose {
        out.push_str(&format!(
            "\n{str_lbl} {str_val}\n{ref_lbl} {ref_val}",
            str_lbl = style("Stratum:").cyan().bold(),
            str_val = r.stratum,
            ref_lbl = style("Reference ID:").cyan().bold(),
            ref_val = r.ref_id
        ));
    }

    out
}

/// Render comparison results line by line with the legacy style.
pub fn render_compare(results: &[ProbeResult], verbose: bool) -> String {
    let mut out = String::new();

    // Header
    if results.len() == 2 {
        out.push_str(&format!(
            "{} {} and {}\n",
            style("Comparing").bold(),
            style(&results[0].target.name).green(),
            style(&results[1].target.name).green()
        ));
    } else {
        out.push_str(&format!(
            "{} {} servers\n",
            style("Comparing (async):").bold(),
            results.len()
        ));
    }

    // Lines
    for r in results {
        let ip_style = if r.target.ip.is_ipv6() {
            style(r.target.ip).cyan()
        } else {
            style(r.target.ip).blue()
        };
        let ip_version = if r.target.ip.is_ipv6() { "v6" } else { "v4" };
        let offset_style = style(format!("{:.3} ms", r.offset_ms)).yellow();

        out.push_str(&format!(
            "{} [{} {}]: {}\n",
            style(&r.target.name).green().bold(),
            ip_style,
            ip_version,
            offset_style
        ));

        if verbose {
            out.push_str(&format!(
                "  {} {}\n  {} {}\n  {} {:.3} ms\n",
                style("Stratum:").cyan().bold(),
                r.stratum,
                style("Reference ID:").cyan().bold(),
                r.ref_id,
                style("Round Trip Delay:").cyan().bold(),
                r.rtt_ms
            ));
        }
    }

    // Stats
    let min = results.iter().map(|r| r.offset_ms).fold(f64::INFINITY, f64::min);
    let max = results.iter().map(|r| r.offset_ms).fold(f64::NEG_INFINITY, f64::max);
    let avg = results.iter().map(|r| r.offset_ms).sum::<f64>() / results.len() as f64;
    let diff = max - min;

    out.push_str(&format!(
        "{} {:.3} ms (min: {:.3}, max: {:.3}, avg: {:.3})\n",
        style("Max drift:").cyan().bold(),
        diff,
        min,
        max,
        avg
    ));

    out
}
