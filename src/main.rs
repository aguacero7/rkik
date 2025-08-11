// src/main.rs
use clap::Parser;
use console::{Term, style};
use std::process;

use rkik::{Args, OutputFormat, compare_servers, query_server};

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    let term = Term::stdout();
    if args.json {
        args.format = OutputFormat::Json;
    }
    if args.pretty & !args.json {
        term.write_line(
            &style("Error: There is no pretty print for the plain text display")
                .red()
                .bold()
                .to_string(),
        )
        .unwrap();
        process::exit(1);
    }
    match (&args.compare, &args.server, &args.positional) {
        (Some(servers), _, _) if servers.len() >= 2 => {
            compare_servers(servers, &term, &args).await;
        }
        (_, Some(server), _) => query_server(server, &term, &args),
        (_, None, Some(pos)) => query_server(pos, &term, &args),
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
