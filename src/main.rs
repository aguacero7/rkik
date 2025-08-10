use clap::Parser;
use console::{Term, style};
use std::process;

use rkik::{Args, compare, output::writer, query};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let term = Term::stdout();

    match (&args.compare, &args.server, &args.positional) {
        (Some(servers), _, _) if servers.len() >= 2 => match compare(servers, args.ipv6).await {
            Ok(results) => writer::write_compare(&term, &results, &args.format),
            Err(e) => {
                term.write_line(&style(format!("Error: {:?}", e)).red().to_string())
                    .unwrap();
                process::exit(1);
            }
        },
        (_, Some(server), _) => match query(server, args.ipv6) {
            Ok(result) => writer::write_query(&term, server, &result, &args.format, args.verbose),
            Err(e) => {
                term.write_line(&style(format!("Error: {:?}", e)).red().to_string())
                    .unwrap();
                process::exit(1);
            }
        },
        (_, None, Some(pos)) => match query(pos, args.ipv6) {
            Ok(result) => writer::write_query(&term, pos, &result, &args.format, args.verbose),
            Err(e) => {
                term.write_line(&style(format!("Error: {:?}", e)).red().to_string())
                    .unwrap();
                process::exit(1);
            }
        },
        _ => {
            term.write_line(
                &style("Error: Provide either a server, a positional argument, or --compare, -h to show help.")
                    .red()
                    .bold()
                    .to_string(),
            )
            .unwrap();
            process::exit(1);
        }
    }
}
