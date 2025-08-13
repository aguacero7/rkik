use crate::domain::ntp::ProbeResult;
use console::style;

/// Render a probe result into human readable text.
pub fn render_probe(r: &ProbeResult) -> String {
    let offset = style(format!("{:.3} ms", r.offset_ms)).yellow();
    let rtt = style(format!("{:.3} ms", r.rtt_ms)).magenta();
    format!(
        "Server: {}\nIP: {}\nUTC Time: {}\nLocal Time: {}\nClock Offset: {}\nRound Trip Delay: {}",
        style(&r.target.name).green().bold(),
        style(r.target.ip).cyan(),
        style(r.utc.to_rfc2822()).blue(),
        style(r.local.format("%Y-%m-%d %H:%M:%S").to_string()).blue(),
        offset,
        rtt
    )
}

/// Render comparison results line by line.
pub fn render_compare(results: &[ProbeResult]) -> String {
    let mut out = String::new();
    for r in results {
        let ip_style = if r.target.ip.is_ipv6() {
            style(r.target.ip).cyan()
        } else {
            style(r.target.ip).blue()
        };
        let ip_version = if r.target.ip.is_ipv6() { "v6" } else { "v4" };
        let version_style = style(ip_version).dim();
        let offset_style = style(format!("{:.3} ms", r.offset_ms)).yellow();
        out.push_str(&format!(
            "{} [{} {}]: {}\n",
            style(&r.target.name).green().bold(),
            ip_style,
            version_style,
            offset_style
        ));
    }
    out
}
