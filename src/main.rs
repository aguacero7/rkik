use clap::{Parser, Subcommand};
use rsntp::SntpClient;
use chrono::{DateTime, Local, Utc};
use console::{style, Term};
use std::process;
use serde_json::json;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool", long_about = None)]
struct Args {
    /// NTP server address
    #[arg(short, long)]
    server: Option<String>,

    /// Compare two servers
    #[arg(long)]
    compare: Option<Vec<String>>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
}

fn main() {
    let args = Args::parse();
    let term = Term::stdout();

    if let Some(server) = &args.server {
        query_server(server, &term, &args);
    } else if let Some(servers) = &args.compare {
        if servers.len() == 2 {
            compare_servers(&servers, &term, &args);
        } else {
            term.write_line(&style("Error: --compare requires exactly two servers").red().bold().to_string()).unwrap();
            process::exit(1);
        }
    } else {
        term.write_line(&style("Error: Either --server or --compare must be provided").red().bold().to_string()).unwrap();
        process::exit(1);
    }
}

fn format_reference_id(reference_id: u32) -> String {
    format!("0x{:08X}", reference_id)
}

fn query_server(server: &str, term: &Term, args: &Args) {
    let client = SntpClient::new();
    match client.synchronize(server) {
        Ok(result) => {
            let datetime_utc: DateTime<Utc> = result.datetime().try_into().unwrap();
            let local_time: DateTime<Local> = DateTime::from(datetime_utc);
            let offset_ms = result.clock_offset().as_secs_f64() * 1000.0;
            let rtt_ms = result.round_trip_delay().as_secs_f64() * 1000.0;

            if args.format == "json" {
                let json_output = json!({
                    "server": server,
                    "utc_time": datetime_utc.to_rfc2822(),
                    "local_time": local_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    "offset_ms": offset_ms,
                    "rtt_ms": rtt_ms,
                    "stratum": result.stratum(),
                });
                println!("{}", json_output.to_string());
            } else {
                term.write_line(&format!("{} {}", style("Server:").cyan().bold(), style(server).green())).unwrap();
                term.write_line(&format!("{} {}", style("UTC Time:").cyan().bold(), style(datetime_utc.to_rfc2822()).green())).unwrap();
                term.write_line(&format!("{} {}", style("Local Time:").cyan().bold(), style(local_time.format("%Y-%m-%d %H:%M:%S")).green())).unwrap();
                term.write_line(&format!("{} {:.3} ms", style("Clock Offset:").cyan().bold(), offset_ms)).unwrap();
                term.write_line(&format!("{} {:.3} ms", style("Round Trip Delay:").cyan().bold(), rtt_ms)).unwrap();
                if args.verbose {
                    term.write_line(&format!("{} {}", style("Stratum:").cyan().bold(), result.stratum())).unwrap();
                }
            }
        }
        Err(e) => {
            term.write_line(&format!("Error querying server: {}", e)).unwrap();
            process::exit(1);
        }
    }
}

fn compare_servers(servers: &[String], term: &Term, args: &Args) {
    let client = SntpClient::new();
    let server1 = &servers[0];
    let server2 = &servers[1];

    let result1 = client.synchronize(server1);
    let result2 = client.synchronize(server2);

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            let offset1 = r1.clock_offset().as_secs_f64() * 1000.0;
            let offset2 = r2.clock_offset().as_secs_f64() * 1000.0;
            let diff = (offset1 - offset2).abs();

            if args.format == "json" {
                let json_output = json!({
                    "server1": server1,
                    "offset1": offset1,
                    "server2": server2,
                    "offset2": offset2,
                    "difference": diff
                });
                println!("{}", json_output.to_string());
            } else {
                term.write_line(&format!("Comparing {} and {}", style(server1).yellow(), style(server2).yellow())).unwrap();
                term.write_line(&format!("{}: {:.3} ms", style(server1).green(), offset1)).unwrap();
                term.write_line(&format!("{}: {:.3} ms", style(server2).green(), offset2)).unwrap();
                term.write_line(&format!("{} {:.3} ms", style("Difference:").cyan().bold(), diff)).unwrap();
            }
        }
        (Err(e1), Err(e2)) => {
            term.write_line(&format!("Error querying {}: {}
Error querying {}: {}", server1, e1, server2, e2)).unwrap();
        }
        (Err(e), _) => {
            term.write_line(&format!("Error querying {}: {}", server1, e)).unwrap();
        }
        (_, Err(e)) => {
            term.write_line(&format!("Error querying {}: {}", server2, e)).unwrap();
        }
    }
}
