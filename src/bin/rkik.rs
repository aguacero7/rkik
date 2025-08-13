use clap::{Parser, ValueEnum};
use console::{Term, set_colors_enabled, style};
use std::process;
use std::time::Duration;

use rkik::{ProbeResult, RkikError, compare_many, fmt, query_one};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
struct Args {
    /// Query a single NTP server
    #[arg(short, long)]
    server: Option<String>,

    /// Compare multiple servers
    #[arg(short = 'C', long, num_args = 2..)]
    compare: Option<Vec<String>>,

    /// Show detailed output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Output format: text or json
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Alias for JSON output
    #[arg(long)]
    json: bool,

    /// Pretty-print JSON
    #[arg(long)]
    pretty: bool,

    /// Disable colored output
    #[arg(long = "no-color", alias = "nocolor")]
    no_color: bool,

    /// Use IPv6 resolution only
    #[arg(short = '6', long)]
    ipv6: bool,

    /// Timeout in seconds
    #[arg(long, default_value_t = 5)]
    timeout: u64,

    /// Positional server name or IP
    #[arg(index = 1)]
    positional: Option<String>,
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    if args.no_color {
        set_colors_enabled(false);
    }
    let term = Term::stdout();
    if args.json {
        args.format = OutputFormat::Json;
    }
    let timeout = Duration::from_secs(args.timeout);

    let exit_code = match (&args.compare, &args.server, &args.positional) {
        (Some(list), _, _) => match compare_many(list, args.ipv6, timeout).await {
            Ok(results) => {
                output(
                    &term,
                    &results,
                    args.format.clone(),
                    args.pretty,
                    args.verbose,
                );
                0
            }
            Err(e) => handle_error(&term, e),
        },
        (_, Some(server), _) => match query_one(server, args.ipv6, timeout).await {
            Ok(res) => {
                output(
                    &term,
                    std::slice::from_ref(&res),
                    args.format.clone(),
                    args.pretty,
                    args.verbose,
                );
                0
            }
            Err(e) => handle_error(&term, e),
        },
        (_, None, Some(pos)) => match query_one(pos, args.ipv6, timeout).await {
            Ok(res) => {
                output(
                    &term,
                    std::slice::from_ref(&res),
                    args.format.clone(),
                    args.pretty,
                    args.verbose,
                );
                0
            }
            Err(e) => handle_error(&term, e),
        },
        _ => {
            term.write_line(
                &style("Error: Provide either a server, a positional argument, or --compare")
                    .red()
                    .bold()
                    .to_string(),
            )
            .ok();
            1
        }
    };

    process::exit(exit_code);
}

fn output(term: &Term, results: &[ProbeResult], fmt: OutputFormat, pretty: bool, verbose: bool) {
    match fmt {
        OutputFormat::Text => {
            if results.len() == 1 {
                let s = fmt::text::render_probe(&results[0], verbose);
                term.write_line(&s).ok();
            } else {
                let s = fmt::text::render_compare(results, verbose);
                term.write_line(&s).ok();
            }
        }
        OutputFormat::Json => match fmt::json::to_json(results, pretty) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("error serializing: {}", e),
        },
    }
}

fn handle_error(term: &Term, err: RkikError) -> i32 {
    let msg = format!("{}", err);
    term.write_line(&style(format!("Error: {}", msg)).red().to_string())
        .ok();
    match err {
        RkikError::Dns(_) => 2,
        RkikError::Network(ref s) if s == "timeout" => 3,
        _ => 1,
    }
}
