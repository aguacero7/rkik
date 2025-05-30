// src/main.rs
use clap::Parser;
use console::{style, Term};
use std::process;

use rkik::{Args, query_server, compare_servers};

fn main() {
    let args = Args::parse();
    let term = Term::stdout();

    match (&args.compare, &args.server, &args.positional) {
        (Some(servers), _, _) if servers.len() == 2 => {
            compare_servers(&servers[0], &servers[1], &term, &args)
        }
        (_, Some(server), _) => {
            query_server(server, &term, &args)
        }
        (_, None, Some(pos)) => {
            query_server(pos, &term, &args)
        }
        _ => {
            term.write_line(&style("Error: Provide either a server, a positional argument, or --compare")
                .red()
                .bold()
                .to_string()).unwrap();
            process::exit(1);
        }
    }
}
