use clap::{Parser, ValueEnum};
use console::{Term, set_colors_enabled, style};
#[cfg(feature = "sync")]
use rkik::sync::{SyncError, get_sys_permissions, sync_from_probe};
use std::io::{self, IsTerminal, Write};
use std::process;
use std::time::Duration;
use tokio::signal;

use rkik::{
    ProbeResult, RkikError, compare_many, fmt, query_one,
    stats::{Stats, compute_stats},
    tui::{TuiApp, run_tui},
};
#[cfg(all(feature = "ptp", target_os = "linux"))]
use rkik::{
    PtpProbeResult, PtpQueryOptions, query_many_ptp, query_one_ptp,
    stats::{PtpStats, compute_ptp_stats},
};
use std::collections::HashMap;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Simple,
    JsonShort,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
            OutputFormat::Simple => "simple",
            OutputFormat::JsonShort => "json-short",
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Parser, Debug, Clone)]
#[command(name = "rkik")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
pub struct LegacyArgs {
    /// Query a single NTP server (optional)
    #[arg(short, long)]
    pub server: Option<String>,

    /// Compare multiple servers
    #[arg(short = 'C', long, num_args = 2..)]
    pub compare: Option<Vec<String>>,

    /// Show detailed output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Output format: text or json
    #[arg(short = 'f', long, default_value = "text", value_enum)]
    pub format: OutputFormat,

    /// Alias for JSON output
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Alias for simple / short text output
    #[arg(short = 'S', long)]
    pub short: bool,

    /// Pretty-print JSON
    #[arg(short = 'p', long)]
    pub pretty: bool,

    /// Disable colored output
    #[arg(long = "no-color", alias = "nocolor")]
    pub no_color: bool,

    /// Use IPv6 resolution only
    #[arg(short = '6', long)]
    pub ipv6: bool,

    /// Timeout in seconds
    #[arg(long, default_value_t = 5.0)]
    pub timeout: f64,

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
    pub target: Option<String>,

    /// Infinite count mode
    #[arg(short = '8', long)]
    pub infinite: bool,

    /// Interval between queries in seconds (only with --infinite or --count)
    #[arg(short = 'i', long, default_value_t = 1.0)]
    pub interval: f64,

    /// Specific count of requests
    #[arg(short = 'c', long, default_value_t = 1)]
    pub count: u32,

    /// Enable NTS (Network Time Security) authentication
    #[cfg(feature = "nts")]
    #[arg(long)]
    pub nts: bool,

    /// NTS-KE (Key Exchange) port number
    #[cfg(feature = "nts")]
    #[arg(long, default_value_t = 4460)]
    pub nts_port: u16,

    /// Enable Precision Time Protocol mode (only available on Linux)
    #[cfg(all(feature = "ptp", target_os = "linux"))]
    #[arg(long)]
    pub ptp: bool,

    /// PTP domain number
    #[cfg(all(feature = "ptp", target_os = "linux"))]
    #[arg(long, default_value_t = 0, requires = "ptp")]
    pub ptp_domain: u8,

    /// PTP event port
    #[cfg(all(feature = "ptp", target_os = "linux"))]
    #[arg(long, default_value_t = 319, requires = "ptp")]
    pub ptp_event_port: u16,

    /// PTP general port
    #[cfg(all(feature = "ptp", target_os = "linux"))]
    #[arg(long, default_value_t = 320, requires = "ptp")]
    pub ptp_general_port: u16,

    /// Request hardware timestamping (simulated)
    #[cfg(all(feature = "ptp", target_os = "linux"))]
    #[arg(long, requires = "ptp")]
    pub ptp_hw_timestamp: bool,

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

impl Default for LegacyArgs {
    fn default() -> Self {
        Self {
            server: None,
            compare: None,
            verbose: false,
            format: OutputFormat::Text,
            json: false,
            short: false,
            pretty: false,
            no_color: false,
            ipv6: false,
            timeout: 5.0,
            #[cfg(feature = "sync")]
            sync: false,
            #[cfg(feature = "sync")]
            dry_run: false,
            target: None,
            infinite: false,
            interval: 1.0,
            count: 1,
            #[cfg(feature = "nts")]
            nts: false,
            #[cfg(feature = "nts")]
            nts_port: 4460,
            #[cfg(all(feature = "ptp", target_os = "linux"))]
            ptp: false,
            #[cfg(all(feature = "ptp", target_os = "linux"))]
            ptp_domain: 0,
            #[cfg(all(feature = "ptp", target_os = "linux"))]
            ptp_event_port: 319,
            #[cfg(all(feature = "ptp", target_os = "linux"))]
            ptp_general_port: 320,
            #[cfg(all(feature = "ptp", target_os = "linux"))]
            ptp_hw_timestamp: false,
            plugin: false,
            warning: None,
            critical: None,
        }
    }
}

pub async fn run(mut args: LegacyArgs, _warn_legacy: bool) {
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

    // Validate thresholds for plugin mode
    if args.plugin {
        if let Some(w) = args.warning {
            if w < 0.0 {
                term.write_line(&style("--warning must be non-negative").red().to_string())
                    .ok();
                let _ = io::stdout().flush();
                process::exit(2);
            }
        }
        if let Some(c) = args.critical {
            if c < 0.0 {
                term.write_line(&style("--critical must be non-negative").red().to_string())
                    .ok();
                let _ = io::stdout().flush();
                process::exit(2);
            }
        }
        if let (Some(w), Some(c)) = (args.warning, args.critical) {
            if w >= c {
                term.write_line(
                    &style("--warning must be less than --critical")
                        .red()
                        .to_string(),
                )
                .ok();
                let _ = io::stdout().flush();
                process::exit(2);
            }
        }
    }

    if args.infinite && args.count != 1 {
        term.write_line(
            &style("--infinite cannot be used with --count")
                .red()
                .to_string(),
        )
        .ok();
        let _ = io::stdout().flush();
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
        let _ = io::stdout().flush();
        process::exit(2);
    }
    #[cfg(all(feature = "ptp", feature = "nts", target_os = "linux"))]
    if args.ptp && args.nts {
        term.write_line(
            &style("--ptp cannot be combined with --nts")
                .red()
                .to_string(),
        )
        .ok();
        let _ = io::stdout().flush();
        process::exit(2);
    }
    #[cfg(all(feature = "ptp", feature = "sync", target_os = "linux"))]
    if args.ptp && args.sync {
        term.write_line(
            &style("--ptp cannot be combined with --sync")
                .red()
                .to_string(),
        )
        .ok();
        let _ = io::stdout().flush();
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
        let _ = io::stdout().flush();
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
        let _ = io::stdout().flush();
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
        let _ = io::stdout().flush();
        process::exit(2);
    }

    #[cfg(all(feature = "ptp", target_os = "linux"))]
    if args.ptp {
        let opts = PtpQueryOptions::new(
            args.ptp_domain,
            args.ptp_event_port,
            args.ptp_general_port,
            args.ptp_hw_timestamp,
            args.verbose,
        );
        let exit_code = run_ptp_mode(&args, &term, timeout, opts).await;
        let _ = io::stdout().flush();
        process::exit(exit_code);
    }

    let exit_code = match (&args.compare, &args.server, &args.target) {
        (Some(list), _, _) => {
            // Use TUI mode for compare if --infinite is set and format is text
            if args.infinite && matches!(args.format, OutputFormat::Text) && !args.verbose {
                compare_loop_tui(list, &args, timeout).await;
                return;
            }

            #[cfg(feature = "nts")]
            let (use_nts, nts_port) = (args.nts, args.nts_port);
            #[cfg(not(feature = "nts"))]
            let (use_nts, nts_port) = (false, 4460u16);

            let mut all: HashMap<String, Vec<ProbeResult>> = HashMap::new();
            let mut n = 0u32;
            loop {
                match compare_many(list, args.ipv6, timeout, use_nts, nts_port).await {
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
                        let _ = io::stdout().flush();
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

    let _ = io::stdout().flush();
    process::exit(exit_code);
}

async fn compare_loop_tui(list: &[String], args: &LegacyArgs, timeout: Duration) {
    #[cfg(feature = "nts")]
    let (use_nts, nts_port) = (args.nts, args.nts_port);
    #[cfg(not(feature = "nts"))]
    let (use_nts, nts_port) = (false, 4460u16);

    let interval = Duration::from_secs_f64(args.interval);
    let mut tui_app = TuiApp::new(list.to_vec());

    // Channel to communicate between async query task and TUI
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<Result<ProbeResult, String>>>();

    // Spawn async query task
    let list_clone = list.to_vec();
    let ipv6 = args.ipv6;
    tokio::spawn(async move {
        loop {
            let mut cycle_results = Vec::new();

            match compare_many(&list_clone, ipv6, timeout, use_nts, nts_port).await {
                Ok(results) => {
                    for res in results {
                        cycle_results.push(Ok(res));
                    }
                }
                Err(e) => {
                    // Send error for all servers
                    for server in &list_clone {
                        cycle_results.push(Err(format!("{}: {}", server, e)));
                    }
                }
            }

            if tx.send(cycle_results).is_err() {
                break;
            }

            tokio::time::sleep(interval).await;
        }
    });

    let _ = run_tui(&mut tui_app, |app| {
        // Check for new results
        while let Ok(cycle_results) = rx.try_recv() {
            app.start_new_cycle();

            for result in cycle_results {
                match result {
                    Ok(res) => {
                        app.update_server(&res.target.name, &res);
                    }
                    Err(err_msg) => {
                        // Extract server name from error message
                        if let Some(server_name) = err_msg.split(':').next() {
                            app.update_server_error(server_name, err_msg.clone());
                        }
                    }
                }
            }
        }
        Ok(true)
    });
}

async fn query_loop_tui(target: &str, args: &LegacyArgs, timeout: Duration) {
    #[cfg(feature = "nts")]
    let (use_nts, nts_port) = (args.nts, args.nts_port);
    #[cfg(not(feature = "nts"))]
    let (use_nts, nts_port) = (false, 4460u16);

    let interval = Duration::from_secs_f64(args.interval);
    let mut tui_app = TuiApp::new(vec![target.to_string()]);

    // We need to run TUI in a blocking context, so we'll use a channel
    // to communicate between the async query task and the TUI
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Option<ProbeResult>>();

    // Spawn async query task
    let target_clone = target.to_string();
    let ipv6 = args.ipv6;
    tokio::spawn(async move {
        loop {
            match query_one(&target_clone, ipv6, timeout, use_nts, nts_port).await {
                Ok(res) => {
                    if tx.send(Some(res)).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    if tx.send(None).is_err() {
                        break;
                    }
                }
            }
            tokio::time::sleep(interval).await;
        }
    });

    let _ = run_tui(&mut tui_app, |app| {
        // Check for new results
        while let Ok(result_opt) = rx.try_recv() {
            match result_opt {
                Some(res) => {
                    app.update_server(&res.target.name, &res);
                }
                None => {
                    app.update_server_error(target, "Query failed".to_string());
                }
            }
            app.start_new_cycle();
        }
        Ok(true)
    });
}

async fn query_loop(target: &str, args: &LegacyArgs, term: &Term, timeout: Duration) {
    // Use TUI mode if --infinite is set and output is text
    if args.infinite && matches!(args.format, OutputFormat::Text) && !args.verbose && !args.plugin {
        query_loop_tui(target, args, timeout).await;
        return;
    }
    let mut all = Vec::new();
    let mut n = 0u32;

    #[cfg(feature = "nts")]
    let (use_nts, nts_port) = (args.nts, args.nts_port);
    #[cfg(not(feature = "nts"))]
    let (use_nts, nts_port) = (false, 4460u16);

    loop {
        match query_one(target, args.ipv6, timeout, use_nts, nts_port).await {
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
                    emit_unknown(args.warning, args.critical);
                    let _ = io::stdout().flush();
                    process::exit(3);
                }
                let code = handle_error(term, e);
                let _ = io::stdout().flush();
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

    if all.len() > 1 && !args.plugin {
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
            emit_unknown(args.warning, args.critical);
            let _ = io::stdout().flush();
            process::exit(3);
        }

        let stats = compute_stats(&all);
        let offset = stats.offset_avg;
        let rtt = stats.rtt_avg;
        let host = &all[0].target.name;
        let ip = &all[0].target.ip;

        let warn_str = args.warning.map(|v| v.to_string()).unwrap_or_default();
        let crit_str = args.critical.map(|v| v.to_string()).unwrap_or_default();

        let abs_offset = offset.abs();
        let mut exit_code = 0i32;
        if let Some(c) = args.critical {
            if abs_offset >= c {
                exit_code = 2;
            }
        }
        if exit_code == 0 {
            if let Some(w) = args.warning {
                if abs_offset >= w {
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

        let _ = io::stdout().flush();
        process::exit(exit_code);
    }

    #[cfg(feature = "sync")]
    if args.sync {
        let mut no_sync = false;
        if !get_sys_permissions() || args.dry_run {
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
                let _ = io::stdout().flush();
                process::exit(12);
            }
            Err(SyncError::Sys(e)) => {
                term.write_line(&style(format!("Error: {}", e)).red().to_string())
                    .ok();
                let _ = io::stdout().flush();
                process::exit(14);
            }
            Err(SyncError::NotSupported) => {
                term.write_line(
                    &style("Error: sync not supported on this platform")
                        .red()
                        .to_string(),
                )
                .ok();
                let _ = io::stdout().flush();
                process::exit(15);
            }
        }
    }
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
async fn run_ptp_mode(
    args: &LegacyArgs,
    term: &Term,
    timeout: Duration,
    opts: PtpQueryOptions,
) -> i32 {
    match (&args.compare, &args.server, &args.target) {
        (Some(list), _, _) => ptp_compare_loop(list, args, term, timeout, &opts).await,
        (_, Some(server), _) => {
            ptp_query_loop(server, args, term, timeout, &opts).await;
            0
        }
        (_, None, Some(pos)) => {
            ptp_query_loop(pos, args, term, timeout, &opts).await;
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
    }
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
async fn ptp_query_loop(
    target: &str,
    args: &LegacyArgs,
    term: &Term,
    timeout: Duration,
    opts: &PtpQueryOptions,
) {
    let mut all = Vec::new();
    let mut n = 0u32;
    loop {
        match query_one_ptp(target, args.ipv6, timeout, opts).await {
            Ok(res) => {
                if !args.plugin {
                    if args.count > 1 || args.infinite {
                        let format = args.format.clone();
                        match format {
                            OutputFormat::Text => {
                                if args.verbose {
                                    output_ptp(
                                        term,
                                        std::slice::from_ref(&res),
                                        OutputFormat::Text,
                                        args.pretty,
                                        true,
                                    );
                                } else {
                                    let line = fmt::ptp_text::render_short_probe(&res);
                                    term.write_line(&line).ok();
                                }
                            }
                            OutputFormat::JsonShort => {
                                match fmt::ptp_json::probe_to_short_json(&res) {
                                    Ok(s) => println!("{}", s),
                                    Err(e) => eprintln!("error serializing: {}", e),
                                }
                            }
                            _ => {
                                output_ptp(
                                    term,
                                    std::slice::from_ref(&res),
                                    format,
                                    args.pretty,
                                    args.verbose,
                                );
                            }
                        }
                    } else {
                        output_ptp(
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
                    emit_ptp_unknown(args.warning, args.critical);
                    let _ = io::stdout().flush();
                    process::exit(3);
                }
                let code = handle_error(term, e);
                let _ = io::stdout().flush();
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

    if all.len() > 1 && !args.plugin {
        let stats = compute_ptp_stats(&all);
        match args.format {
            OutputFormat::Json => {
                match fmt::ptp_json::stats_to_json(&all[0].target.name, &stats, args.pretty) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("error serializing: {}", e),
                }
            }
            _ => {
                let line = fmt::ptp_text::render_stats(&all[0].target.name, &stats);
                term.write_line(&line).ok();
            }
        }
    }

    if args.plugin {
        if all.is_empty() {
            emit_ptp_unknown(args.warning, args.critical);
            let _ = io::stdout().flush();
            process::exit(3);
        }
        let stats = compute_ptp_stats(&all);
        let probe = &all[0];
        let exit_code = emit_ptp_plugin(&stats, probe, args);
        let _ = io::stdout().flush();
        process::exit(exit_code);
    }
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
async fn ptp_compare_loop(
    list: &[String],
    args: &LegacyArgs,
    term: &Term,
    timeout: Duration,
    opts: &PtpQueryOptions,
) -> i32 {
    let mut all: HashMap<String, Vec<PtpProbeResult>> = HashMap::new();
    let mut n = 0u32;
    loop {
        match query_many_ptp(list, args.ipv6, timeout, opts).await {
            Ok(results) => {
                if args.count > 1 || args.infinite {
                    match args.format {
                        OutputFormat::Text => {
                            if args.verbose {
                                output_ptp(term, &results, OutputFormat::Text, args.pretty, true);
                            } else {
                                let line = fmt::ptp_text::render_short_compare(&results);
                                term.write_line(&line).ok();
                            }
                        }
                        OutputFormat::JsonShort => {
                            for r in &results {
                                match fmt::ptp_json::probe_to_short_json(r) {
                                    Ok(s) => println!("{}", s),
                                    Err(e) => eprintln!("error serializing: {}", e),
                                }
                            }
                        }
                        _ => {
                            output_ptp(
                                term,
                                &results,
                                args.format.clone(),
                                args.pretty,
                                args.verbose,
                            );
                        }
                    }
                } else {
                    output_ptp(
                        term,
                        &results,
                        args.format.clone(),
                        args.pretty,
                        args.verbose,
                    );
                }
                for res in results {
                    all.entry(res.target.name.clone()).or_default().push(res);
                }
            }
            Err(e) => {
                let code = handle_error(term, e);
                let _ = io::stdout().flush();
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
        let mut stats_list: Vec<(String, PtpStats)> = all
            .into_iter()
            .map(|(name, vals)| (name, compute_ptp_stats(&vals)))
            .collect();
        stats_list.sort_by(|a, b| a.0.cmp(&b.0));
        match args.format {
            OutputFormat::Json => {
                match fmt::ptp_json::stats_list_to_json(&stats_list, args.pretty) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("error serializing: {}", e),
                }
            }
            _ => {
                for (name, st) in &stats_list {
                    let line = fmt::ptp_text::render_stats(name, st);
                    term.write_line(&line).ok();
                }
            }
        }
    }
    0
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
fn emit_ptp_unknown(warning: Option<f64>, critical: Option<f64>) {
    let warn_str = warning.map(|v| v.to_string()).unwrap_or_default();
    let crit_str = critical.map(|v| v.to_string()).unwrap_or_default();
    println!(
        "RKIK UNKNOWN - PTP request failed | offset_ns=;{};{};0; delay_ns=;;;0;",
        warn_str, crit_str
    );
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
fn emit_ptp_plugin(stats: &PtpStats, probe: &PtpProbeResult, args: &LegacyArgs) -> i32 {
    let warn_str = args.warning.map(|v| v.to_string()).unwrap_or_default();
    let crit_str = args.critical.map(|v| v.to_string()).unwrap_or_default();
    let offset = stats.offset_avg_ns;
    let delay = stats.mean_path_delay_avg_ns;
    let host = &probe.target.name;
    let ip = &probe.target.ip;

    let mut exit_code = 0i32;
    if let Some(c) = args.critical {
        if offset.abs() >= c {
            exit_code = 2;
        }
    }
    if exit_code == 0 {
        if let Some(w) = args.warning {
            if offset.abs() >= w {
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
        "RKIK {state} - offset {offset:.0}ns delay {delay:.0}ns from {host} ({ip}) | offset_ns={offset:.0}ns;{warn};{crit};0; delay_ns={delay:.0}ns;;;0;",
        state = state,
        offset = offset,
        delay = delay,
        host = host,
        ip = ip,
        warn = warn_str,
        crit = crit_str
    );

    exit_code
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
fn output_ptp(
    term: &Term,
    results: &[PtpProbeResult],
    fmt: OutputFormat,
    pretty: bool,
    verbose: bool,
) {
    match fmt {
        OutputFormat::Text => {
            if results.len() == 1 {
                let s = fmt::ptp_text::render_probe(&results[0], verbose);
                term.write_line(&s).ok();
            } else {
                let s = fmt::ptp_text::render_compare(results, verbose);
                term.write_line(&s).ok();
            }
        }
        OutputFormat::Json => match fmt::ptp_json::to_json(results, pretty, verbose) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("error serializing: {}", e),
        },
        OutputFormat::JsonShort => match fmt::ptp_json::to_short_json(results, pretty) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("error serializing: {}", e),
        },
        OutputFormat::Simple => {
            if results.len() == 1 {
                let s = fmt::ptp_text::render_simple_probe(&results[0]);
                term.write_line(&s).ok();
            } else {
                let s = fmt::ptp_text::render_simple_compare(results);
                term.write_line(&s).ok();
            }
        }
    }
}

/// Emit a plugin-mode UNKNOWN status line with the provided thresholds
fn emit_unknown(warning: Option<f64>, critical: Option<f64>) {
    let warn_str = warning.map(|v| v.to_string()).unwrap_or_default();
    let crit_str = critical.map(|v| v.to_string()).unwrap_or_default();
    println!(
        "RKIK UNKNOWN - request failed | offset_ms=;{};{};0; rtt_ms=;;;0;",
        warn_str, crit_str
    );
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
