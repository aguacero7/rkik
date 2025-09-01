use chrono::{DateTime, Local, Utc};
use clap::Parser;
use console::{Term, style};
use rsntp::{ReferenceIdentifier, SntpClient};
use std::net::{IpAddr, ToSocketAddrs};

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
#[command(long_about = Some(
        "Query and compare NTP servers from the CLI.\n\
         \n\
         Examples:\n\
           rkik 0.pool.ntp.org\n\
           rkik --server time.google.com --verbose\n\
           rkik --compare ntp1 ntp2 --format json\n\
         \n\
         Supports both IPv4 and IPv6, positional or flagged arguments."
    ))]
pub struct Args {
    /// Query a single NTP server
    #[arg(short, long)]
    pub server: Option<String>,

    /// Compare two servers
    #[arg(long, num_args = 2)]
    pub compare: Option<Vec<String>>,

    /// Show detailed output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format: "text" or "json"
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Use IPv6 resolution
    #[arg(long)]
    pub ipv6: bool,

    /// Positional server name or IP (used if --server not provided)
    #[arg(index = 1)]
    pub positional: Option<String>,
}

pub fn resolve_ip(host: &str, ipv6: bool) -> Option<String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(ip.to_string());
    }
    let port = 123;
    let addrs = if ipv6 {
        (host, port).to_socket_addrs().ok()?.find(|a| a.is_ipv6())
    } else {
        (host, port).to_socket_addrs().ok()?.find(|a| a.is_ipv4())
    };
    addrs.map(|a| a.ip().to_string())
}

fn format_reference_id(reference_id: &ReferenceIdentifier) -> String {
    reference_id.to_string()
}

pub fn query_server(server: &str, term: &Term, args: &Args) {
    let client = SntpClient::new();

    match client.synchronize(server) {
        Ok(result) => {
            let ip = resolve_ip(server, args.ipv6).unwrap_or_else(|| "unknown".into());
            let datetime_utc: DateTime<Utc> = result.datetime().try_into().unwrap();
            let local_time: DateTime<Local> = DateTime::from(datetime_utc);
            let offset_ms = result.clock_offset().as_secs_f64() * 1000.0;
            let rtt_ms = result.round_trip_delay().as_secs_f64() * 1000.0;
            let ref_id = format_reference_id(result.reference_identifier());

            if args.format == "json" {
                println!(
                    "{{\
                        \"server\": \"{}\", \
                        \"ip\": \"{}\", \
                        \"utc_time\": \"{}\", \
                        \"local_time\": \"{}\", \
                        \"offset_ms\": {:.3}, \
                        \"rtt_ms\": {:.3}, \
                        \"stratum\": {}, \
                        \"reference_id\": \"{}\"\
                    }}",
                    server,
                    ip,
                    datetime_utc.to_rfc3339(),
                    local_time.format("%Y-%m-%d %H:%M:%S"),
                    offset_ms,
                    rtt_ms,
                    result.stratum(),
                    ref_id
                );
            } else {
                term.write_line(&format!(
                    "{} {}",
                    style("Server:").cyan().bold(),
                    style(server).green()
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} {}",
                    style("IP:").cyan().bold(),
                    style(ip).green()
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} {}",
                    style("UTC Time:").cyan().bold(),
                    style(datetime_utc.to_rfc2822()).green()
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} {}",
                    style("Local Time:").cyan().bold(),
                    style(local_time.format("%Y-%m-%d %H:%M:%S")).green()
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} {:.3} ms",
                    style("Clock Offset:").cyan().bold(),
                    offset_ms
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} {:.3} ms",
                    style("Round Trip Delay:").cyan().bold(),
                    rtt_ms
                ))
                .unwrap();
                if args.verbose {
                    term.write_line(&format!(
                        "{} {}",
                        style("Stratum:").cyan().bold(),
                        result.stratum()
                    ))
                    .unwrap();
                    term.write_line(&format!(
                        "{} {}",
                        style("Reference ID:").cyan().bold(),
                        ref_id
                    ))
                    .unwrap();
                }
            }
        }
        Err(e) => term
            .write_line(&format!("Error querying server '{}': {}", server, e))
            .unwrap(),
    }
}

pub fn compare_servers(server1: &str, server2: &str, term: &Term, args: &Args) {
    let client = SntpClient::new();
    let result1 = client.synchronize(server1);
    let result2 = client.synchronize(server2);

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            let offset1 = r1.clock_offset().as_secs_f64() * 1000.0;
            let offset2 = r2.clock_offset().as_secs_f64() * 1000.0;
            let diff = (offset1 - offset2).abs();

            let ip1 = resolve_ip(server1, args.ipv6).unwrap_or_else(|| "unknown".into());
            let ip2 = resolve_ip(server2, args.ipv6).unwrap_or_else(|| "unknown".into());

            if args.format == "json" {
                println!(
                    "{{\
                        \"server1\": \"{}\", \
                        \"ip1\": \"{}\", \
                        \"offset1_ms\": {:.3}, \
                        \"server2\": \"{}\", \
                        \"ip2\": \"{}\", \
                        \"offset2_ms\": {:.3}, \
                        \"difference_ms\": {:.3}\
                    }}",
                    server1, ip1, offset1, server2, ip2, offset2, diff
                );
            } else {
                term.write_line(&format!(
                    "{} {} and {}",
                    style("Comparing").bold(),
                    style(server1).yellow(),
                    style(server2).yellow()
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} [{}]: {:.3} ms",
                    style(server1).green(),
                    ip1,
                    offset1
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} [{}]: {:.3} ms",
                    style(server2).green(),
                    ip2,
                    offset2
                ))
                .unwrap();
                term.write_line(&format!(
                    "{} {:.3} ms",
                    style("Difference:").cyan().bold(),
                    diff
                ))
                .unwrap();
            }
        }
        (Err(e1), Err(e2)) => term
            .write_line(&format!(
                "Error querying '{}': {}\nError querying '{}': {}",
                server1, e1, server2, e2
            ))
            .unwrap(),
        (Err(e), _) => term
            .write_line(&format!("Error querying '{}': {}", server1, e))
            .unwrap(),
        (_, Err(e)) => term
            .write_line(&format!("Error querying '{}': {}", server2, e))
            .unwrap(),
    }
}
