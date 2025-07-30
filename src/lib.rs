use chrono::{DateTime, Local, Utc};
use clap::Parser;
use console::{Term, style};
use futures::future::join_all;
use rsntp::{
    AsyncSntpClient, Config, ReferenceIdentifier, SntpClient, SynchronizationError,
    SynchronizationResult,
};
use std::net::{IpAddr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::process;

#[derive(Parser, Debug)]
#[command(name = "rkik")]
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
    #[arg(short='C',long, num_args = 2..10)]
    pub compare: Option<Vec<String>>,

    /// Show detailed output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Output format: "text" or "json"
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Use IPv6 resolution only
    #[arg(short = '6', long)]
    pub ipv6: bool,

    /// Positional server name or IP (used if --server not provided)
    #[arg(index = 1)]
    pub positional: Option<String>,
}
pub fn resolve_ip_for_mode(host: &str, ipv6_only: bool) -> Result<IpAddr, String> {
    let port = 123;
    let addrs: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for '{}': {}", host, e))?
        .collect();

    let filtered: Vec<IpAddr> = if ipv6_only {
        addrs
            .iter()
            .map(|a| a.ip())
            .filter(|ip| ip.is_ipv6())
            .collect()
    } else {
        // enforce IPv4 first, then IPv6
        // This ensures that if both IPv4 and IPv6 addresses are available,
        // the IPv4 address is preferred.
        let mut v4 = vec![];
        let mut v6 = vec![];
        for a in addrs {
            let ip = a.ip();
            if ip.is_ipv4() {
                v4.push(ip);
            } else {
                v6.push(ip);
            }
        }
        v4.into_iter().chain(v6).collect()
    };

    filtered.into_iter().next().ok_or_else(|| {
        if ipv6_only {
            format!("No IPv6 address found for '{}'", host)
        } else {
            format!("No IP address found for '{}'", host)
        }
    })
}

fn client_for_mode(ipv6: bool) -> SntpClient {
    if ipv6 {
        let config = Config::default().bind_address((Ipv6Addr::UNSPECIFIED, 0).into());
        SntpClient::with_config(config)
    } else {
        let config = Config::default().bind_address(([0, 0, 0, 0], 0).into()); // 0.0.0.0:0
        SntpClient::with_config(config)
    }
}

pub fn synchronize_with_ip(
    client: &SntpClient,
    ip: IpAddr,
) -> Result<SynchronizationResult, SynchronizationError> {
    let addr = SocketAddr::new(ip, 123);
    client.synchronize(addr.to_string())
}

fn format_reference_id(reference_id: &ReferenceIdentifier) -> String {
    reference_id.to_string()
}

pub fn query_server(server: &str, term: &Term, args: &Args) {
    let ip = match resolve_ip_for_mode(server, args.ipv6) {
        Ok(ip) => ip,
        Err(e) => {
            term.write_line(&style(format!("Error: {}", e)).red().to_string())
                .unwrap();
            process::exit(1);
        }
    };

    let client = client_for_mode(args.ipv6);

    match synchronize_with_ip(&client, ip) {
        Ok(result) => {
            let datetime_utc: DateTime<Utc> = result.datetime().try_into().unwrap();
            let local_time: DateTime<Local> = DateTime::from(datetime_utc);
            let offset_ms = result.clock_offset().as_secs_f64() * 1000.0;
            let rtt_ms = result.round_trip_delay().as_secs_f64() * 1000.0;
            let ref_id = format_reference_id(result.reference_identifier());
            let ip_version = if ip.is_ipv6() { "v6" } else { "v4" };

            if args.format == "json" {
                println!(
                    "{{\
                        \"server\": \"{}\", \
                        \"ip\": \"{}\", \
                        \"ip_version\": \"{}\", \
                        \"utc_time\": \"{}\", \
                        \"local_time\": \"{}\", \
                        \"offset_ms\": {:.3}, \
                        \"rtt_ms\": {:.3}, \
                        \"stratum\": {}, \
                        \"reference_id\": \"{}\"\
                    }}",
                    server,
                    ip,
                    ip_version,
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
                    "{} {} ({})",
                    style("IP:").cyan().bold(),
                    style(ip).green(),
                    ip_version
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

pub async fn compare_servers(servers: &[String], term: &Term, args: &Args) {
    if servers.len() < 2 {
        term.write_line(
            &style("Need at least 2 servers to compare")
                .red()
                .to_string(),
        )
        .unwrap();
        return;
    }

    let resolved: Vec<_> = servers
        .iter()
        .map(|s| (s.clone(), resolve_ip_for_mode(s, args.ipv6)))
        .collect();

    let valid: Vec<_> = resolved
        .iter()
        .filter_map(|(name, ip)| match ip {
            Ok(addr) => Some((name.clone(), *addr)),
            Err(e) => {
                term.write_line(
                    &style(format!("Could not resolve {}: {}", name, e))
                        .red()
                        .to_string(),
                )
                .ok();
                None
            }
        })
        .collect();

    if valid.len() < 2 {
        term.write_line(
            &style("Not enough valid servers to compare.")
                .red()
                .to_string(),
        )
        .unwrap();
        return;
    }

    let client = {
        let config = if args.ipv6 {
            Config::default().bind_address((Ipv6Addr::UNSPECIFIED, 0).into())
        } else {
            Config::default().bind_address(([0, 0, 0, 0], 0).into())
        };
        AsyncSntpClient::with_config(config)
    };

    let futures = valid
        .iter()
        .map(|(_, ip)| client.synchronize(SocketAddr::new(*ip, 123).to_string()))
        .collect::<Vec<_>>();

    let results = join_all(futures).await;

    let mut final_results = vec![];

    for ((name, ip), res) in valid.iter().zip(results) {
        match res {
            Ok(r) => {
                let offset = r.clock_offset().as_secs_f64() * 1000.0;
                final_results.push((name.clone(), *ip, offset));
            }
            Err(e) => {
                term.write_line(
                    &style(format!("Failed to query {}: {}", name, e))
                        .red()
                        .to_string(),
                )
                .ok();
            }
        }
    }

    if final_results.len() < 2 {
        term.write_line(
            &style("At least two successful responses required to compare.")
                .red()
                .to_string(),
        )
        .unwrap();
        return;
    }

    if args.format == "json" {
        println!("[");
        for (i, (name, ip, offset)) in final_results.iter().enumerate() {
            println!(
                "  {{ \"server\": \"{}\", \"ip\": \"{}\", \"offset_ms\": {:.3} }}{}",
                name,
                ip,
                offset,
                if i < final_results.len() - 1 { "," } else { "" }
            );
        }
        println!("]");
    } else {
        term.write_line(&format!(
            "{} {} servers",
            style("Comparing (async):").bold(),
            final_results.len()
        ))
        .unwrap();

        for (name, ip, offset) in final_results.iter() {
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

        let min = final_results
            .iter()
            .map(|(_, _, o)| *o)
            .fold(f64::INFINITY, f64::min);
        let max = final_results
            .iter()
            .map(|(_, _, o)| *o)
            .fold(f64::NEG_INFINITY, f64::max);
        let avg =
            final_results.iter().map(|(_, _, o)| *o).sum::<f64>() / final_results.len() as f64;
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
}
