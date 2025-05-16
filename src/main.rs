use clap::{Parser};
use rsntp::SntpClient;
use rsntp::ReferenceIdentifier;
use chrono::{DateTime, Local, Utc};
use console::{style, Term};
use std::process;

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
struct Args {
    #[arg(short, long)]
    server: Option<String>,

    #[arg(long, num_args = 2)]
    compare: Option<Vec<String>>,


    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long, default_value = "text")]
    format: String,
}

fn main() {
    let args = Args::parse();
    let term = Term::stdout();

    match (&args.server, &args.compare) {
        (Some(server), _) => query_server(server, &term, &args),
        (_, Some(servers)) if servers.len() == 2 => compare_servers(&servers[0], &servers[1], &term, &args),
        _ => {
            term.write_line(&style("Error: Either --server or --compare must be provided").red().bold().to_string()).unwrap();
            process::exit(1);
        }
    }
}

fn format_reference_id(reference_id: &ReferenceIdentifier) -> String {
    return reference_id.to_string()
}

fn query_server(server: &str, term: &Term, args: &Args) {
    let client = SntpClient::new();
    match client.synchronize(server) {
        Ok(result) => {
            let datetime_utc: DateTime<Utc> = result.datetime().try_into().unwrap();
            let local_time: DateTime<Local> = DateTime::from(datetime_utc);
            let offset_ms = result.clock_offset().as_secs_f64() * 1000.0;
            let rtt_ms = result.round_trip_delay().as_secs_f64() * 1000.0;
            let ref_id = format_reference_id(result.reference_identifier());

            if args.format == "json" {
                println!("{{ \"server\": \"{}\", \"utc_time\": \"{}\", \"local_time\": \"{}\", \"offset_ms\": {:.3}, \"rtt_ms\": {:.3}, \"stratum\": {}, \"reference_id\": \"{}\" }}",
                         server, datetime_utc.to_rfc2822(), local_time.format("%Y-%m-%d %H:%M:%S"), offset_ms, rtt_ms, result.stratum(), ref_id);
            } else {
                term.write_line(&format!("{} {}", style("Server:").cyan().bold(), style(server).green())).unwrap();
                term.write_line(&format!("{} {}", style("UTC Time:").cyan().bold(), style(datetime_utc.to_rfc2822()).green())).unwrap();
                term.write_line(&format!("{} {}", style("Local Time:").cyan().bold(), style(local_time.format("%Y-%m-%d %H:%M:%S")).green())).unwrap();
                term.write_line(&format!("{} {:.3} ms", style("Clock Offset:").cyan().bold(), offset_ms)).unwrap();
                term.write_line(&format!("{} {:.3} ms", style("Round Trip Delay:").cyan().bold(), rtt_ms)).unwrap();
                if args.verbose {
                    term.write_line(&format!("{} {}", style("Stratum:").cyan().bold(), result.stratum())).unwrap();
                    term.write_line(&format!("{} {}", style("Reference ID:").cyan().bold(), ref_id)).unwrap();
                }
            }
        }
        Err(e) => term.write_line(&format!("Error querying server: {}", e)).unwrap(),
    }
}

fn compare_servers(server1: &str, server2: &str, term: &Term, args: &Args) {
    let client = SntpClient::new();
    let result1 = client.synchronize(server1);
    let result2 = client.synchronize(server2);

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            let offset1 = r1.clock_offset().as_secs_f64() * 1000.0;
            let offset2 = r2.clock_offset().as_secs_f64() * 1000.0;
            let diff = (offset1 - offset2).abs();

            if args.format == "json" {
                println!("{{ \"server1\": \"{}\", \"offset1\": {:.3}, \"server2\": \"{}\", \"offset2\": {:.3}, \"difference\": {:.3} }}",
                         server1, offset1, server2, offset2, diff);
            } else {
                term.write_line(&format!("Comparing {} and {}", style(server1).yellow(), style(server2).yellow())).unwrap();
                term.write_line(&format!("{}: {:.3} ms", style(server1).green(), offset1)).unwrap();
                term.write_line(&format!("{}: {:.3} ms", style(server2).green(), offset2)).unwrap();
                term.write_line(&format!("{} {:.3} ms", style("Difference:").cyan().bold(), diff)).unwrap();
            }
        }
        (Err(e1), Err(e2)) => term.write_line(&format!("Error querying {}: {}\nError querying {}: {}", server1, e1, server2, e2)).unwrap(),
        (Err(e), _) => term.write_line(&format!("Error querying {}: {}", server1, e)).unwrap(),
        (_, Err(e)) => term.write_line(&format!("Error querying {}: {}", server2, e)).unwrap(),
    }
}
