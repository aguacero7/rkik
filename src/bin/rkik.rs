use atty::Stream;
use clap::{Parser, ValueEnum};
use console::{Term, set_colors_enabled, style};
#[cfg(feature = "sync")]
use rkik::sync::{SyncError, sync_from_probe};
use std::process;
use std::time::Duration;

use rkik::{ProbeResult, RkikError, compare_many, fmt, query_one};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Simple,
}

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(version = env!("CARGO_PKG_VERSION"))]
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
    #[arg(short = 'f', long, default_value = "text", value_enum)]
    format: OutputFormat,

    /// Alias for JSON output
    #[arg(short = 'j', long)]
    json: bool,

    /// Pretty-print JSON
    #[arg(short = 'p', long)]
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

    /// Enable one-shot system clock synchronization (requires root)
    #[cfg(feature = "sync")]
    #[arg(long)]
    pub sync: bool,

    /// Positional server name or IP
    #[arg(index = 1)]
    positional: Option<String>,

    /// Infinite count mode
    #[arg(short = '8', long)]
    infinite: bool,

    /// Interval between queries in seconds (only with --infinite or --count)
    #[arg(short = 'i', long, default_value_t = 1)]
    interval: u64,
    
    /// Specific count of requests 
    #[arg(short = 'c', long, default_value_t = 1)]
    count: u32,
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();

    // alias --json
    if args.json {
        args.format = OutputFormat::Json;
    }

    // colors
    let want_color = matches!(args.format, OutputFormat::Text)
        && atty::is(Stream::Stdout)
        && std::env::var_os("NO_COLOR").is_none()
        && !args.no_color;
    set_colors_enabled(want_color);

    let term = Term::stdout();
    let timeout = Duration::from_secs(args.timeout);

    // refuse --sync with --compare
    #[cfg(feature = "sync")]
    if args.sync && args.compare.is_some() {
        term.write_line(
            &style("--sync cannot be used with --compare")
                .red()
                .to_string(),
        )
        .ok();
        process::exit(2);
    }

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

                #[cfg(feature = "sync")]
                if args.sync {
                    match sync_from_probe(&res) {
                        Ok(()) => {
                            let _ = term.write_line(&style("Sync applied").green().to_string());
                        }
                        Err(SyncError::Permission(e)) => {
                            term.write_line(&format!("Error: {}", e)).ok();
                            process::exit(12);
                        }
                        Err(SyncError::Sys(e)) => {
                            term.write_line(&format!("Error: {}", e)).ok();
                            process::exit(14);
                        }
                        Err(SyncError::NotSupported) => {
                            term.write_line("Error: sync not supported on this platform")
                                .ok();
                            process::exit(15);
                        }
                    }
                }

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

                #[cfg(feature = "sync")]
                if args.sync {
                    match sync_from_probe(&res) {
                        Ok(()) => {
                            let _ = term.write_line(&style("Sync applied").green().to_string());
                        }
                        Err(SyncError::Permission(e)) => {
                            term.write_line(&format!("Error: {}", e)).ok();
                            process::exit(12);
                        }
                        Err(SyncError::Sys(e)) => {
                            term.write_line(&format!("Error: {}", e)).ok();
                            process::exit(14);
                        }
                        Err(SyncError::NotSupported) => {
                            term.write_line("Error: sync not supported on this platform")
                                .ok();
                            process::exit(15);
                        }
                    }
                }

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
        OutputFormat::Json => match fmt::json::to_json(results, pretty,verbose) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("error serializing: {}", e),
        },
    }
}

fn handle_error(term: &Term, err: RkikError) -> i32 {
    term.write_line(&style(format!("Error: {}", err)).red().to_string())
        .ok();
    match err {
        RkikError::Dns(_) => 2,
        RkikError::Network(ref s) if s == "timeout" => 3,
        _ => 1,
    }
}
