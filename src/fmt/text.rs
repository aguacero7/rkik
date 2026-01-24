use crate::domain::ntp::ProbeResult;
use crate::stats::Stats;
use console::style;

/// Render a probe result into human readable text with the legacy style.
pub fn render_probe(r: &ProbeResult, verbose: bool) -> String {
    let ip_val = if r.target.ip.is_ipv6() {
        // [ipv6] in green
        format!("{}", style(format!("[{}]", r.target.ip)).green())
    } else {
        // ipv4/hostname in green
        format!("{}", style(r.target.ip).green())
    };

    // NTS authentication indicator
    let auth_indicator = if r.authenticated {
        format!(" {}", style("[NTS Authenticated]").green().bold())
    } else {
        #[cfg(feature = "nts")]
        {
            if let Some(ref validation) = r.nts_validation {
                if let Some(ref error) = validation.error {
                    format!(
                        " {} ({})",
                        style("[NTS Failed]").red().bold(),
                        style(error.kind.as_str()).red()
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        #[cfg(not(feature = "nts"))]
        String::new()
    };

    let mut out = format!(
        "{srv_lbl} {srv_val}{auth}\n\
         {ip_lbl} {ip_val}:{port}\n\
         {utc_lbl} {utc_val}\n\
         {loc_lbl} {loc_val}\n\
         {off_lbl} {off_val:.3} ms\n\
         {rtt_lbl} {rtt_val:.3} ms",
        srv_lbl = style("Server:").cyan().bold(),
        srv_val = style(&r.target.name).green(),
        auth = auth_indicator,
        ip_lbl = style("IP:").cyan().bold(),
        ip_val = ip_val,
        port = style(r.target.port).green(),
        utc_lbl = style("UTC Time:").cyan().bold(),
        utc_val = style(r.utc.to_rfc2822()).green(),
        loc_lbl = style("Local Time:").cyan().bold(),
        loc_val = style(r.local.format("%Y-%m-%d %H:%M:%S")).green(),
        off_lbl = style("Clock Offset:").cyan().bold(),
        off_val = r.offset_ms,
        rtt_lbl = style("Round Trip Delay:").cyan().bold(),
        rtt_val = r.rtt_ms,
    );

    if verbose {
        out.push_str(&format!(
            "\n{str_lbl} {str_val}\n{ref_lbl} {ref_val}\n{str_ts}: {timestamp}\n{auth_lbl} {auth_val}",
            str_lbl = style("Stratum:").cyan().bold(),
            str_val = r.stratum,
            ref_lbl = style("Reference ID:").cyan().bold(),
            ref_val = r.ref_id,
            str_ts = style("Timestamp").cyan().bold(),
            timestamp = r.timestamp,
            auth_lbl = style("Authenticated:").cyan().bold(),
            auth_val = if r.authenticated {
                style("Yes (NTS)").green()
            } else {
                style("No").yellow()
            }
        ));

        // NTS-KE diagnostic information (verbose mode only)
        #[cfg(feature = "nts")]
        if let Some(ref nts_ke) = r.nts_ke_data {
            out.push_str(&format!(
                "\n\n{header}\n{ke_dur_lbl} {ke_dur_val:.3} ms\n{cookies_lbl} {cookies_val}\n{algo_lbl} {algo_val}\n{ntp_srv_lbl} {ntp_srv_val}",
                header = style("=== NTS-KE Diagnostics ===").cyan().bold().underlined(),
                ke_dur_lbl = style("Handshake Duration:").cyan().bold(),
                ke_dur_val = nts_ke.ke_duration_ms,
                cookies_lbl = style("Cookies Received:").cyan().bold(),
                cookies_val = style(format!("{} cookies", nts_ke.cookie_count)).green(),
                algo_lbl = style("AEAD Algorithm:").cyan().bold(),
                algo_val = style(&nts_ke.aead_algorithm).green(),
                ntp_srv_lbl = style("NTP Server:").cyan().bold(),
                ntp_srv_val = style(&nts_ke.ntp_server).green(),
            ));

            // Cookie sizes detail
            if !nts_ke.cookie_sizes.is_empty() {
                let cookie_details = nts_ke
                    .cookie_sizes
                    .iter()
                    .enumerate()
                    .map(|(i, size)| format!("  Cookie {}: {} bytes", i + 1, size))
                    .collect::<Vec<_>>()
                    .join("\n");
                out.push_str(&format!(
                    "\n{cookies_det_lbl}\n{cookies_det}",
                    cookies_det_lbl = style("Cookie Sizes:").cyan().bold(),
                    cookies_det = style(cookie_details).dim()
                ));
            }

            // TLS Certificate information
            if let Some(ref cert) = nts_ke.certificate {
                out.push_str(&format!(
                    "\n\n{cert_header}\n{subj_lbl} {subj}\n{issuer_lbl} {issuer}\n{valid_lbl} {valid_from} to {valid_until}\n{fp_lbl}\n  {fp}",
                    cert_header = style("=== TLS Certificate ===").cyan().bold().underlined(),
                    subj_lbl = style("Subject:").cyan().bold(),
                    subj = style(&cert.subject).green(),
                    issuer_lbl = style("Issuer:").cyan().bold(),
                    issuer = style(&cert.issuer).green(),
                    valid_lbl = style("Valid:").cyan().bold(),
                    valid_from = style(&cert.valid_from).green(),
                    valid_until = style(&cert.valid_until).green(),
                    fp_lbl = style("Fingerprint (SHA-256):").cyan().bold(),
                    fp = style(&cert.fingerprint_sha256).dim(),
                ));

                // SANs if available
                if !cert.san_dns_names.is_empty() {
                    out.push_str(&format!(
                        "\n{san_lbl}",
                        san_lbl = style("SANs:").cyan().bold(),
                    ));
                    for san in &cert.san_dns_names {
                        out.push_str(&format!("\n  - {}", style(san).dim()));
                    }
                }

                // Algorithms
                out.push_str(&format!(
                    "\n{sig_lbl} {sig}\n{pk_lbl} {pk}",
                    sig_lbl = style("Signature Algorithm:").cyan().bold(),
                    sig = style(&cert.signature_algorithm).dim(),
                    pk_lbl = style("Public Key Algorithm:").cyan().bold(),
                    pk = style(&cert.public_key_algorithm).dim(),
                ));

                // Self-signed warning
                if cert.is_self_signed {
                    out.push_str(&format!(
                        "\n{warn}",
                        warn = style("⚠ WARNING: Self-signed certificate").yellow().bold()
                    ));
                }
            }
        }

        // NTS validation error details (verbose mode only)
        #[cfg(feature = "nts")]
        if let Some(ref validation) = r.nts_validation {
            if let Some(ref error) = validation.error {
                out.push_str(&format!(
                    "\n\n{header}\n{kind_lbl} {kind_val}\n{msg_lbl} {msg_val}",
                    header = style("=== NTS Validation Error ===")
                        .red()
                        .bold()
                        .underlined(),
                    kind_lbl = style("Error Kind:").red().bold(),
                    kind_val = style(error.kind.as_str()).red(),
                    msg_lbl = style("Message:").red().bold(),
                    msg_val = style(&error.message).red(),
                ));
            }
        }
    }

    out
}

/// Render comparison results line by line with the legacy style.
pub fn render_compare(results: &[ProbeResult], verbose: bool) -> String {
    let mut out = String::new();

    // Header
    if results.len() == 2 {
        out.push_str(&format!(
            "{} -  {}:{} and {}:{}\n",
            style("Comparing").bold(),
            style(&results[0].target.name).green(),
            style(&results[0].target.port).green(),
            style(&results[1].target.name).green(),
            style(&results[1].target.port).green()
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

        let nts_badge = if r.authenticated {
            format!(" {}", style("[NTS]").green().bold())
        } else {
            #[cfg(feature = "nts")]
            {
                if let Some(ref validation) = r.nts_validation {
                    if validation.error.is_some() {
                        format!(" {}", style("[NTS FAILED]").red().bold())
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            #[cfg(not(feature = "nts"))]
            String::new()
        };

        out.push_str(&format!(
            "{}{} [{} {}]: {}\n",
            style(&r.target.name).green().bold(),
            nts_badge,
            ip_style,
            ip_version,
            offset_style
        ));

        if verbose {
            out.push_str(&format!(
                "  {} {}\n  {} {}\n  {} {:.3} ms\n  {} {}\n",
                style("Stratum:").cyan().bold(),
                r.stratum,
                style("Reference ID:").cyan().bold(),
                r.ref_id,
                style("Round Trip Delay:").cyan().bold(),
                r.rtt_ms,
                style("Authenticated:").cyan().bold(),
                if r.authenticated {
                    style("Yes (NTS)").green()
                } else {
                    style("No").yellow()
                }
            ));

            // NTS-KE diagnostics in compare mode
            #[cfg(feature = "nts")]
            if let Some(ref nts_ke) = r.nts_ke_data {
                out.push_str(&format!(
                    "  {} {:.3} ms\n  {} {}\n  {} {}\n",
                    style("NTS-KE Handshake:").cyan().bold(),
                    nts_ke.ke_duration_ms,
                    style("AEAD Algorithm:").cyan().bold(),
                    style(&nts_ke.aead_algorithm).dim(),
                    style("Cookies:").cyan().bold(),
                    style(format!("{} received", nts_ke.cookie_count)).dim()
                ));
            }
        }
    }

    // Stats
    let min = results
        .iter()
        .map(|r| r.offset_ms)
        .fold(f64::INFINITY, f64::min);
    let max = results
        .iter()
        .map(|r| r.offset_ms)
        .fold(f64::NEG_INFINITY, f64::max);
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

/// Render a minimal line for a probe result.
pub fn render_short_probe(r: &ProbeResult) -> String {
    format!(
        "{name}:{port} {offset}",
        name = style(&r.target.name).green(),
        port = r.target.port,
        offset = style(format!("{:.3} ms", r.offset_ms)).yellow()
    )
}

/// Render a minimal line for comparison results.
pub fn render_short_compare(results: &[ProbeResult]) -> String {
    results
        .iter()
        .map(|r| {
            format!(
                "{name}:{port}:{off}",
                name = style(&r.target.name).green(),
                port = r.target.port,
                off = style(format!("{:.3}", r.offset_ms)).yellow()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render statistics for a set of probe results
pub fn render_stats(name: &str, stats: &Stats) -> String {
    fn fmt_ms(v: f64) -> String {
        format!("{:.3} ms", v)
    }

    format!(
        "\n{n}: {avg_lbl} {avg} ({min_lbl} {min}, {max_lbl} {max}) {rtt_lbl} {rtt} ({cnt} {rqst})",
        n = style(name).green().bold(),
        avg_lbl = style("avg").cyan().bold(),
        avg = style(fmt_ms(stats.offset_avg)).green(),
        min_lbl = style("min").cyan().bold(),
        min = style(fmt_ms(stats.offset_min)).green(),
        max_lbl = style("max").cyan().bold(),
        max = style(fmt_ms(stats.offset_max)).green(),
        rtt_lbl = style("rtt").cyan().bold(),
        rtt = style(fmt_ms(stats.rtt_avg)).green(),
        cnt = style(stats.count).green(),
        rqst = style("requests").green(),
    )
}

/// Render a probe in simple mode (offset and IP only).
pub fn render_simple_probe(r: &ProbeResult) -> String {
    format!(
        "{name}:{port} {offset}",
        name = style(&r.target.name).green(),
        port = style(&r.target.port).green(),
        offset = style(format!("{:.3} ms", r.offset_ms)).yellow()
    )
}

/// Render multiple probes in simple mode.
pub fn render_simple_compare(results: &[ProbeResult]) -> String {
    results
        .iter()
        .map(render_simple_probe)
        .collect::<Vec<_>>()
        .join("\n")
}
