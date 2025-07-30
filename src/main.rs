// src/main.rs
use clap::Parser;
use console::{Term, style};
use std::process;

use rkik::{Args, compare_servers, query_server};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let term = Term::stdout();

    match (&args.compare, &args.server, &args.positional) {
        (Some(servers), _, _) if servers.len() >= 2 => {
            compare_servers(&servers, &term, &args).await;
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
