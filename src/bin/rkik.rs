use clap::{Parser, ValueEnum};
use console::{Term, set_colors_enabled, style};
#[cfg(feature = "sync")]
use rkik::sync::{SyncError, get_sys_permissions, sync_from_probe};
use std::io::{self, IsTerminal};
use std::process;
use std::time::Duration;
use tokio::signal;

use rkik::{
    ProbeResult, RkikError, compare_many, fmt, query_one,
    stats::{Stats, compute_stats},
};
use std::collections::HashMap;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Simple,
    JsonShort,
}

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
struct Args {
    /// Query a single NTP server (optional)
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

    /// Alias for simple / short text output
    #[arg(short = 'S', long)]
    short: bool,

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
    #[arg(long, default_value_t = 5.0)]
    timeout: f64,

    /// Enable one-shot system clock synchronization (requires root)
    #[cfg(feature = "sync")]
    #[arg(long)]
    pub sync: bool,

    /// Flag to cancel synchronisation (for testing)
    #[cfg(feature = "sync")]
    #[arg(short = '0', long = "dry-run")]
    pub dry_run: bool,

    /// Positional server name or IP (can include port specification) - Examples: [time.google.com, [2001:4860:4860::8888]:123, 192.168.1.23:123]
    #[arg(index = 1)]
    target: Option<String>,

    /// Infinite count mode
    #[arg(short = '8', long)]
    infinite: bool,

    /// Interval between queries in seconds (only with --infinite or --count)
    #[arg(short = 'i', long, default_value_t = 1.0)]
    interval: f64,

    /// Specific count of requests
    #[arg(short = 'c', long, default_value_t = 1)]
    count: u32,

    /// Enable Centreon/Nagios plugin output (produces machine-parseable output and proper exit codes)
    #[arg(long)]
    pub plugin: bool,

    /// Warning threshold in ms (requires --plugin)
    #[arg(long, requires = "plugin", value_name = "MS")]
    pub warning: Option<f64>,

    /// Critical threshold in ms (requires --plugin)
    #[arg(long, requires = "plugin", value_name = "MS")]
    pub critical: Option<f64>,
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();

    // alias --json
    if args.json {
        args.format = OutputFormat::Json;
    }
    //alias --short
    if args.short {
        args.format = OutputFormat::Simple;
    }
    if args.short && args.json {
        args.format = OutputFormat::JsonShort;
    }
    // colors
    let want_color = (matches!(args.format, OutputFormat::Text)
        || matches!(args.format, OutputFormat::Simple))
        && io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && !args.no_color;
    set_colors_enabled(want_color);

    let term = Term::stdout();
    let timeout = Duration::from_secs_f64(args.timeout);

    if args.infinite && args.count != 1 {
        term.write_line(
            &style("--infinite cannot be used with --count")
                .red()
                .to_string(),
        )
        .ok();
        process::exit(2);
    }
    if (matches!(args.format, OutputFormat::Simple)
        || matches!(args.format, OutputFormat::JsonShort))
        && args.verbose
    {
        term.write_line(
            &style("--verbose has no effect with short format")
                .yellow()
                .to_string(),
        )
        .ok();
    }
    if args.interval != 1.0 && !args.infinite && args.count == 1 {
        term.write_line(
            &style("--interval requires --infinite or --count")
                .red()
                .to_string(),
        )
        .ok();
        process::exit(2);
    }
    #[cfg(feature = "sync")]
    if args.infinite && args.sync {
        term.write_line(
            &style("--sync cannot be used with --infinite")
                .red()
                .to_string(),
        )
        .ok();
        process::exit(2);
    }

    // refuse --plugin with --compare for now
    if args.plugin && args.compare.is_some() {
        term.write_line(
            &style("--plugin cannot be used with --compare")
                .red()
                .to_string(),
        )
        .ok();
        process::exit(2);
    }

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

    let exit_code = match (&args.compare, &args.server, &args.target) {
        (Some(list), _, _) => {
            let mut all: HashMap<String, Vec<ProbeResult>> = HashMap::new();
            let mut n = 0u32;
            loop {
                match compare_many(list, args.ipv6, timeout).await {
                    Ok(results) => {
                        if args.count > 1 || args.infinite {
                            match args.format {
                                OutputFormat::Text => {
                                    if args.verbose {
                                        output(
                                            &term,
                                            &results,
                                            OutputFormat::Text,
                                            args.pretty,
                                            true,
                                        );
                                    } else {
                                        let line = fmt::text::render_short_compare(&results);
                                        term.write_line(&line).ok();
                                    }
                                }
                                OutputFormat::JsonShort => {
                                    for r in &results {
                                        match fmt::json::probe_to_short_json(r) {
                                            Ok(s) => println!("{}", s),
                                            Err(e) => eprintln!("error serializing: {}", e),
                                        }
                                    }
                                }
                                _ => {
                                    output(
                                        &term,
                                        &results,
                                        args.format.clone(),
                                        args.pretty,
                                        args.verbose,
                                    );
                                }
                            }
                        } else {
                            output(
                                &term,
                                &results,
                                args.format.clone(),
                                args.pretty,
                                args.verbose,
                            );
                        }
                        for r in results {
                            all.entry(r.target.name.clone()).or_default().push(r);
                        }
                    }
                    Err(e) => {
                        let code = handle_error(&term, e);
                        process::exit(code);
                    }
                }
                n += 1;
                if !args.infinite && n >= args.count {
                    break;
                }
                if args.infinite {
                    let sleep = tokio::time::sleep(Duration::from_secs_f64(args.interval));
                    tokio::select! {
                        _ = sleep => {},
                        _ = signal::ctrl_c() => { break; }
                    }
                } else {
                    tokio::time::sleep(Duration::from_secs_f64(args.interval)).await;
                }
            }

            if all.values().map(|v| v.len()).sum::<usize>() > list.len() {
                let mut stats_list: Vec<(String, Stats)> = all
                    .into_iter()
                    .map(|(name, vals)| (name, compute_stats(&vals)))
                    .collect();
                stats_list.sort_by(|a, b| a.0.cmp(&b.0));
                match args.format {
                    OutputFormat::Json => {
                        match fmt::json::stats_list_to_json(&stats_list, args.pretty) {
                            Ok(s) => println!("{}", s),
                            Err(e) => eprintln!("error serializing: {}", e),
                        }
                    }
                    _ => {
                        for (name, st) in &stats_list {
                            let line = fmt::text::render_stats(name, st);
                            term.write_line(&line).ok();
                        }
                        let min = stats_list
                            .iter()
                            .map(|(_, s)| s.offset_avg)
                            .fold(f64::INFINITY, f64::min);
                        let max = stats_list
                            .iter()
                            .map(|(_, s)| s.offset_avg)
                            .fold(f64::NEG_INFINITY, f64::max);
                        let drift = max - min;
                        let _ = term.write_line(&format!("Max avg drift: {:.3} ms", drift));
                    }
                }
            }
            0
        }
        (_, Some(server), _) => {
            query_loop(server, &args, &term, timeout).await;
            0
        }
        (_, None, Some(pos)) => {
            query_loop(pos, &args, &term, timeout).await;
            0
        }
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

async fn query_loop(target: &str, args: &Args, term: &Term, timeout: Duration) {
    let mut all = Vec::new();
    let mut n = 0u32;
    loop {
        match query_one(target, args.ipv6, timeout).await {
            Ok(res) => {
                // In plugin mode we suppress the regular human-readable output and only
                // collect results to produce the plugin line at the end.
                if !args.plugin {
                    if args.count > 1 || args.infinite {
                        let format = args.format.clone();
                        match format {
                            OutputFormat::Text => {
                                if args.verbose {
                                    output(
                                        term,
                                        std::slice::from_ref(&res),
                                        OutputFormat::Text,
                                        args.pretty,
                                        true,
                                    );
                                } else {
                                    let line = fmt::text::render_short_probe(&res);
                                    term.write_line(&line).ok();
                                }
                            }
                            OutputFormat::JsonShort => match fmt::json::probe_to_short_json(&res) {
                                Ok(s) => println!("{}", s),
                                Err(e) => eprintln!("error serializing: {}", e),
                            },

                            _ => {
                                output(
                                    term,
                                    std::slice::from_ref(&res),
                                    format,
                                    args.pretty,
                                    args.verbose,
                                );
                            }
                        }
                    } else {
                        output(
                            term,
                            std::slice::from_ref(&res),
                            args.format.clone(),
                            args.pretty,
                            args.verbose,
                        );
                    }
                }
                all.push(res);
            }
            Err(e) => {
                if args.plugin {
                    // Plugin mode: report UNKNOWN and exit with code 3
                    println!("RKIK UNKNOWN - request failed | offset_ms=;{};{};0; rtt_ms=;;;0;",
                        args.warning.map(|v| v.to_string()).unwrap_or_default(),
                        args.critical.map(|v| v.to_string()).unwrap_or_default()
                    );
                    process::exit(3);
                }
                let code = handle_error(term, e);
                process::exit(code);
            }
        }
        n += 1;
        if !args.infinite && n >= args.count {
            break;
        }
        if args.infinite {
            let sleep = tokio::time::sleep(Duration::from_secs_f64(args.interval));
            tokio::select! {
                _ = sleep => {},
                _ = signal::ctrl_c() => { break; }
            }
        } else {
            tokio::time::sleep(Duration::from_secs_f64(args.interval)).await;
        }
    }

    if all.len() > 1 {
        let stats = compute_stats(&all);
        let format = args.format.clone();
        match format {
            OutputFormat::Json => {
                match fmt::json::stats_to_json(&all[0].target.name, &stats, args.pretty) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("error serializing: {}", e),
                }
            }
            _ => {
                let line = fmt::text::render_stats(&all[0].target.name, &stats);
                term.write_line(&line).ok();
            }
        }
    }

    // Plugin mode: produce Centreon/Nagios compatible output and exit with proper code
    if args.plugin {
        if all.is_empty() {
            println!("RKIK UNKNOWN - no result | offset_ms=;{};{};0; rtt_ms=;;;0;",
                args.warning.map(|v| v.to_string()).unwrap_or_default(),
                args.critical.map(|v| v.to_string()).unwrap_or_default()
            );
            process::exit(3);
        }

        let stats = compute_stats(&all);
        let offset = stats.offset_avg;
        let rtt = stats.rtt_avg;
        let host = &all[0].target.name;
        let ip = &all[0].target.ip;

        let warn_str = args.warning.map(|v| format!("{}", v)).unwrap_or_default();
        let crit_str = args.critical.map(|v| format!("{}", v)).unwrap_or_default();

        let abs_offset = offset.abs();
        let mut exit_code = 0i32;
        if let Some(c) = args.critical {
            if abs_offset > c {
                exit_code = 2;
            }
        }
        if exit_code == 0 {
            if let Some(w) = args.warning {
                if abs_offset > w {
                    exit_code = 1;
                }
            }
        }

        let state = match exit_code {
            0 => "OK",
            1 => "WARNING",
            2 => "CRITICAL",
            _ => "UNKNOWN",
        };

        println!(
            "RKIK {} - offset {:.3}ms rtt {:.3}ms from {} ({}) | offset_ms={:.3}ms;{};{};0; rtt_ms={:.3}ms;;;0;",
            state, offset, rtt, host, ip, offset, warn_str, crit_str, rtt
        );

        process::exit(exit_code);
    }

    #[cfg(feature = "sync")]
    if args.sync {
        let mut no_sync = false;
        if !get_sys_permissions() | args.dry_run {
            no_sync = true;
        }
        let probe = average_probe(&all);

        match sync_from_probe(&probe, no_sync) {
            Ok(()) => {
                if !get_sys_permissions() {
                    let _ = term
                        .write_line(&style("Error: need root or CAP_SYS_TIME").red().to_string());
                } else if args.dry_run {
                    let _ = term.write_line(&style("Sync skipped (dry-run)").yellow().to_string());
                } else if args.count <= 1 {
                    let _ = term.write_line(&style("Sync applied").green().to_string());
                } else {
                    let _ = term.write_line(
                        &style(format!(
                            "Average offset Sync applied : {:.3} ms",
                            probe.offset_ms
                        ))
                        .green()
                        .to_string(),
                    );
                }
            }
            Err(SyncError::Permission(e)) => {
                term.write_line(&style(format!("Error: {}", e)).red().to_string())
                    .ok();
                process::exit(12);
            }
            Err(SyncError::Sys(e)) => {
                term.write_line(&style(format!("Error: {}", e)).red().to_string())
                    .ok();
                process::exit(14);
            }
            Err(SyncError::NotSupported) => {
                term.write_line(
                    &style("Error: sync not supported on this platform")
                        .red()
                        .to_string(),
                )
                .ok();
                process::exit(15);
            }
        }
    }
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
        OutputFormat::Json => match fmt::json::to_json(results, pretty, verbose) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("error serializing: {}", e),
        },
        OutputFormat::JsonShort => match fmt::json::to_short_json(results, pretty) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("error serializing: {}", e),
        },
        OutputFormat::Simple => {
            if results.len() == 1 {
                let s = fmt::text::render_simple_probe(&results[0]);
                term.write_line(&s).ok();
            } else {
                let s = fmt::text::render_simple_compare(results);
                term.write_line(&s).ok();
            }
        }
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

#[cfg(feature = "sync")]
fn average_probe(results: &[ProbeResult]) -> ProbeResult {
    let mut avg = results.last().cloned().unwrap();
    avg.offset_ms = results.iter().map(|r| r.offset_ms).sum::<f64>() / results.len() as f64;
    avg.rtt_ms = results.iter().map(|r| r.rtt_ms).sum::<f64>() / results.len() as f64;
    if let Some(min_stratum) = results.iter().map(|r| r.stratum).min() {
        avg.stratum = min_stratum;
    }
    avg
}
