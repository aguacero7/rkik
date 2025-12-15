#[path = "rkik/config_store.rs"]
mod config_store;
#[path = "rkik/legacy.rs"]
mod legacy;

use clap::{Args as ClapArgs, CommandFactory, Parser, Subcommand, ValueEnum};
use config_store::{ConfigError, ConfigStore, Defaults, PresetRecord};
use legacy::{LegacyArgs, OutputFormat};
use std::env;
use std::process::{self, Command as ProcessCommand};

#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a standard NTP probe loop
    Ntp(NtpCommand),
    /// Compare multiple servers concurrently
    Compare(CompareCommand),
    /// Run PTP queries (Linux only)
    #[cfg(all(feature = "ptp", target_os = "linux"))]
    Ptp(PtpCommand),
    /// One-shot synchronization workflow
    #[cfg(feature = "sync")]
    Sync(SyncCommand),
    /// Diagnostic helpers for a single target
    Diag(DiagCommand),
    /// Inspect or update rkik configuration
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Manage reusable presets
    #[command(subcommand)]
    Preset(PresetCommand),
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct ProbeOptions {
    /// Number of requests to perform
    #[arg(short = 'c', long, value_name = "COUNT")]
    count: Option<u32>,

    /// Interval between requests (s)
    #[arg(short = 'i', long, value_name = "SECONDS")]
    interval: Option<f64>,

    /// Timeout per request (s)
    #[arg(long, value_name = "SECONDS")]
    timeout: Option<f64>,

    /// Run until Ctrl+C
    #[arg(short = '8', long)]
    infinite: bool,

    /// Force IPv6 resolution
    #[arg(short = '6', long)]
    ipv6: bool,
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct OutputOptions {
    /// Verbose / human-friendly output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Output format
    #[arg(short = 'f', long, value_enum)]
    format: Option<OutputFormat>,

    /// Shortcut for --format json
    #[arg(short = 'j', long)]
    json: bool,

    /// Shortcut for --format simple
    #[arg(short = 'S', long)]
    short: bool,

    /// Pretty-print JSON
    #[arg(short = 'p', long)]
    pretty: bool,

    /// Disable colors
    #[arg(long = "no-color", alias = "nocolor")]
    no_color: bool,
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct PluginOptions {
    /// Emit Centreon/Nagios plugin output
    #[arg(long)]
    plugin: bool,

    /// Warning threshold (ms for NTP, ns for PTP)
    #[arg(long, requires = "plugin", value_name = "WARN")]
    warning: Option<f64>,

    /// Critical threshold (ms for NTP, ns for PTP)
    #[arg(long, requires = "plugin", value_name = "CRIT")]
    critical: Option<f64>,
}

#[cfg(feature = "nts")]
#[derive(ClapArgs, Debug, Clone, Default)]
struct NtsOptions {
    /// Enable Network Time Security
    #[arg(long)]
    nts: bool,

    /// NTS-KE port
    #[arg(long, default_value_t = 4460)]
    nts_port: u16,
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct NtpCommand {
    #[command(flatten)]
    common: ProbeOptions,

    #[command(flatten)]
    output: OutputOptions,

    #[command(flatten)]
    plugin: PluginOptions,

    #[cfg(feature = "nts")]
    #[command(flatten)]
    nts: NtsOptions,

    /// Target host (hostname or IP)
    #[arg(value_name = "TARGET")]
    target: Option<String>,
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct CompareCommand {
    #[command(flatten)]
    common: ProbeOptions,

    #[command(flatten)]
    output: OutputOptions,

    #[cfg(feature = "nts")]
    #[command(flatten)]
    nts: NtsOptions,

    /// Servers to compare
    #[arg(value_name = "TARGET", num_args = 2..)]
    targets: Vec<String>,
}

#[cfg(feature = "sync")]
#[derive(ClapArgs, Debug, Clone, Default)]
struct SyncCommand {
    #[command(flatten)]
    common: ProbeOptions,

    #[command(flatten)]
    output: OutputOptions,

    /// Skip actually setting the time
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Target to synchronize with
    #[arg(value_name = "TARGET")]
    target: String,
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct DiagCommand {
    #[command(flatten)]
    common: ProbeOptions,

    /// Target to diagnose
    #[arg(value_name = "TARGET")]
    target: String,
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
#[derive(ClapArgs, Debug, Clone, Default)]
struct PtpCommand {
    #[command(flatten)]
    common: ProbeOptions,

    #[command(flatten)]
    output: OutputOptions,

    #[command(flatten)]
    plugin: PluginOptions,

    /// Target grandmaster/bridge
    #[arg(value_name = "TARGET")]
    target: String,

    /// PTP domain
    #[arg(long, default_value_t = 0)]
    domain: u8,

    /// Event port (UDP)
    #[arg(long, default_value_t = 319)]
    event_port: u16,

    /// General port (UDP)
    #[arg(long, default_value_t = 320)]
    general_port: u16,

    /// Request hardware timestamping
    #[arg(long)]
    hw_timestamp: bool,
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Show the configuration file path
    Path,
    /// List stored defaults
    List,
    /// Get default value
    Get {
        #[arg(value_enum)]
        key: ConfigKey,
    },
    /// Set default value
    Set {
        #[arg(value_enum)]
        key: ConfigKey,
        value: String,
    },
    /// Clear a default value
    Clear {
        #[arg(value_enum)]
        key: ConfigKey,
    },
}

#[derive(Subcommand, Debug)]
enum PresetCommand {
    /// List saved presets
    List,
    /// Store a preset from trailing arguments (use -- to separate)
    Add {
        name: String,
        #[arg(trailing_var_arg = true, value_name = "ARGS")]
        args: Vec<String>,
    },
    /// Remove a preset
    Remove { name: String },
    /// Show stored arguments
    Show { name: String },
    /// Execute a preset by spawning rkik with the stored arguments
    Run { name: String },
}

#[derive(ValueEnum, Clone, Debug)]
enum ConfigKey {
    #[value(alias = "default-timeout")]
    Timeout,
    #[value(alias = "default-format")]
    Format,
    #[value(alias = "default-ipv6")]
    Ipv6Only,
}

enum Mode {
    Modern,
    Legacy,
    Help(Vec<String>),
}

#[tokio::main]
async fn main() {
    match detect_mode() {
        Mode::Help(path) => {
            if let Err(err) = print_help_for(&path) {
                eprintln!("Error: {}", err);
                process::exit(2);
            }
        }
        Mode::Legacy => {
            let args = LegacyArgs::parse();
            legacy::run(args, true).await;
        }
        Mode::Modern => {
            let mut config = load_config();
            let cli = Cli::parse();
            if let Some(cmd) = cli.command {
                if let Err(err) = dispatch_command(cmd, &mut config).await {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            } else if let Err(err) = print_help_for(&[]) {
                eprintln!("Error: {}", err);
                process::exit(2);
            }
        }
    }
}

async fn dispatch_command(cmd: Command, config: &mut ConfigStore) -> Result<(), String> {
    match cmd {
        Command::Ntp(opts) => {
            let legacy_args = build_ntp_args(opts, config.defaults())?;
            legacy::run(legacy_args, false).await;
        }
        Command::Compare(opts) => {
            if opts.targets.len() < 2 {
                return Err("Provide at least two targets to compare".into());
            }
            let legacy_args = build_compare_args(opts, config.defaults())?;
            legacy::run(legacy_args, false).await;
        }
        #[cfg(all(feature = "ptp", target_os = "linux"))]
        Command::Ptp(opts) => {
            let legacy_args = build_ptp_args(opts, config.defaults());
            legacy::run(legacy_args, false).await;
        }
        #[cfg(feature = "sync")]
        Command::Sync(opts) => {
            let legacy_args = build_sync_args(opts, config.defaults())?;
            legacy::run(legacy_args, false).await;
        }
        Command::Diag(opts) => {
            let legacy_args = build_diag_args(opts, config.defaults());
            legacy::run(legacy_args, false).await;
        }
        Command::Config(cmd) => handle_config(cmd, config)?,
        Command::Preset(cmd) => handle_preset(cmd, config)?,
    }
    Ok(())
}

fn build_ntp_args(cmd: NtpCommand, defaults: &Defaults) -> Result<LegacyArgs, String> {
    let mut args = LegacyArgs::default();
    if let Some(target) = cmd.target {
        args.target = Some(target);
    } else {
        return Err("Provide a target (e.g. rkik ntp pool.ntp.org)".into());
    }
    apply_probe_options(&mut args, &cmd.common, defaults);
    apply_output_options(&mut args, &cmd.output, defaults)?;
    apply_plugin_options(&mut args, &cmd.plugin);
    #[cfg(feature = "nts")]
    {
        args.nts = cmd.nts.nts;
        args.nts_port = cmd.nts.nts_port;
    }
    Ok(args)
}

fn build_compare_args(cmd: CompareCommand, defaults: &Defaults) -> Result<LegacyArgs, String> {
    if cmd.targets.len() < 2 {
        return Err("Comparison requires at least two targets".into());
    }
    let mut args = LegacyArgs::default();
    args.compare = Some(cmd.targets);
    apply_probe_options(&mut args, &cmd.common, defaults);
    apply_output_options(&mut args, &cmd.output, defaults)?;
    #[cfg(feature = "nts")]
    {
        args.nts = cmd.nts.nts;
        args.nts_port = cmd.nts.nts_port;
    }
    Ok(args)
}

#[cfg(feature = "sync")]
fn build_sync_args(cmd: SyncCommand, defaults: &Defaults) -> Result<LegacyArgs, String> {
    let mut args = LegacyArgs::default();
    args.target = Some(cmd.target);
    args.sync = true;
    args.dry_run = cmd.dry_run;
    apply_probe_options(&mut args, &cmd.common, defaults);
    apply_output_options(&mut args, &cmd.output, defaults)?;
    Ok(args)
}

fn build_diag_args(cmd: DiagCommand, defaults: &Defaults) -> LegacyArgs {
    let mut args = LegacyArgs::default();
    args.target = Some(cmd.target);
    args.verbose = true;
    args.count = 1;
    args.interval = cmd.common.interval.unwrap_or(1.0);
    args.timeout = cmd.common.timeout.or(defaults.timeout).unwrap_or(5.0);
    args.ipv6 = cmd.common.ipv6 || defaults.ipv6_only.unwrap_or(false);
    args.format = OutputFormat::Text;
    args.pretty = false;
    args.infinite = false;
    args.plugin = false;
    args.no_color = false;
    args
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
fn build_ptp_args(cmd: PtpCommand, defaults: &Defaults) -> LegacyArgs {
    let mut args = LegacyArgs::default();
    args.target = Some(cmd.target);
    args.ptp = true;
    args.ptp_domain = cmd.domain;
    args.ptp_event_port = cmd.event_port;
    args.ptp_general_port = cmd.general_port;
    args.ptp_hw_timestamp = cmd.hw_timestamp;
    apply_probe_options(&mut args, &cmd.common, defaults);
    apply_output_options(&mut args, &cmd.output, defaults).ok();
    apply_plugin_options(&mut args, &cmd.plugin);
    args
}

fn apply_probe_options(args: &mut LegacyArgs, opts: &ProbeOptions, defaults: &Defaults) {
    args.count = opts.count.unwrap_or(1);
    args.interval = opts.interval.unwrap_or(1.0);
    args.timeout = opts.timeout.or(defaults.timeout).unwrap_or(5.0);
    args.infinite = opts.infinite;
    args.ipv6 = opts.ipv6 || defaults.ipv6_only.unwrap_or(false);
}

fn apply_output_options(
    args: &mut LegacyArgs,
    opts: &OutputOptions,
    defaults: &Defaults,
) -> Result<(), String> {
    args.verbose = opts.verbose;
    args.pretty = opts.pretty;
    args.no_color = opts.no_color;
    let mut format = opts
        .format
        .clone()
        .or_else(|| {
            defaults
                .format
                .as_deref()
                .and_then(|s| OutputFormat::from_str(s, false).ok())
        })
        .unwrap_or(OutputFormat::Text);
    if opts.json {
        format = OutputFormat::Json;
    } else if opts.short {
        format = OutputFormat::Simple;
    }
    args.format = format;
    Ok(())
}

fn apply_plugin_options(args: &mut LegacyArgs, opts: &PluginOptions) {
    args.plugin = opts.plugin;
    args.warning = opts.warning;
    args.critical = opts.critical;
}

fn handle_config(cmd: ConfigCommand, config: &mut ConfigStore) -> Result<(), String> {
    match cmd {
        ConfigCommand::Path => {
            println!("{}", config.path().display());
        }
        ConfigCommand::List => {
            let defaults = config.defaults();
            println!("timeout = {}", display_opt_float(defaults.timeout));
            println!(
                "format = {}",
                defaults.format.as_deref().unwrap_or("<unset>")
            );
            println!(
                "ipv6_only = {}",
                defaults
                    .ipv6_only
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "<unset>".into())
            );
        }
        ConfigCommand::Get { key } => match key {
            ConfigKey::Timeout => println!("{}", display_opt_float(config.defaults().timeout)),
            ConfigKey::Format => println!(
                "{}",
                config.defaults().format.as_deref().unwrap_or("<unset>")
            ),
            ConfigKey::Ipv6Only => println!(
                "{}",
                config
                    .defaults()
                    .ipv6_only
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "<unset>".into())
            ),
        },
        ConfigCommand::Set { key, value } => {
            apply_config_value(config, key, Some(value))?;
            persist_config(config)?;
        }
        ConfigCommand::Clear { key } => {
            apply_config_value(config, key, None)?;
            persist_config(config)?;
        }
    }
    Ok(())
}

fn handle_preset(cmd: PresetCommand, config: &mut ConfigStore) -> Result<(), String> {
    match cmd {
        PresetCommand::List => {
            if config.presets().is_empty() {
                println!("(no presets)");
            } else {
                for (name, preset) in config.presets() {
                    println!("{name}: {}", preset.args.join(" "));
                }
            }
        }
        PresetCommand::Add { name, args } => {
            if args.is_empty() {
                return Err("Provide arguments after --".into());
            }
            config.add_preset(name.clone(), args);
            persist_config(config)?;
            println!("Preset '{name}' stored");
        }
        PresetCommand::Remove { name } => {
            if config.remove_preset(&name) {
                persist_config(config)?;
                println!("Removed preset '{name}'");
            } else {
                return Err(format!("Preset '{name}' not found"));
            }
        }
        PresetCommand::Show { name } => match config.preset(&name) {
            Some(PresetRecord { args }) => println!("{}", args.join(" ")),
            None => return Err(format!("Preset '{name}' not found")),
        },
        PresetCommand::Run { name } => {
            let preset = config
                .preset(&name)
                .ok_or_else(|| format!("Preset '{name}' not found"))?;
            run_preset(preset)?;
            return Ok(());
        }
    }
    Ok(())
}

fn detect_mode() -> Mode {
    let mut args = env::args_os();
    args.next(); // skip binary
    match args.next() {
        None => Mode::Modern,
        Some(first) => {
            let first_str = first.to_string_lossy();
            if first_str == "help" {
                let rest = args
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect::<Vec<_>>();
                Mode::Help(rest)
            } else if is_new_keyword(&first_str)
                || matches!(first_str.as_ref(), "-h" | "--help" | "-V" | "--version")
            {
                Mode::Modern
            } else {
                Mode::Legacy
            }
        }
    }
}

fn is_new_keyword(s: &str) -> bool {
    matches!(
        s,
        "ntp" | "compare" | "ptp" | "sync" | "diag" | "config" | "preset"
    )
}

fn load_config() -> ConfigStore {
    match ConfigStore::load() {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Warning: could not load rkik config: {}", err);
            ConfigStore::empty()
        }
    }
}

fn print_help_for(path: &[String]) -> Result<(), String> {
    let mut command = Cli::command();
    if path.is_empty() {
        command.print_help().map_err(|e| e.to_string())?;
        println!();
        return Ok(());
    }

    let mut current = &mut command;
    for segment in path {
        current = current
            .find_subcommand_mut(segment)
            .ok_or_else(|| format!("Unknown command '{segment}'"))?;
    }

    current.print_help().map_err(|e| e.to_string())?;
    println!();
    Ok(())
}

fn apply_config_value(
    config: &mut ConfigStore,
    key: ConfigKey,
    value: Option<String>,
) -> Result<(), String> {
    match key {
        ConfigKey::Timeout => {
            let parsed = value
                .as_deref()
                .map(|v| {
                    v.parse::<f64>()
                        .map_err(|_| format!("Invalid timeout: {v}"))
                })
                .transpose()?;
            config.update_timeout(parsed);
        }
        ConfigKey::Format => {
            let parsed = value
                .as_deref()
                .map(|v| {
                    OutputFormat::from_str(v, false)
                        .map(|fmt| fmt.as_str().to_string())
                        .map_err(|_| {
                            "Unknown format. Use text, json, json-short, or simple.".to_string()
                        })
                })
                .transpose()?;
            config.update_format(parsed);
        }
        ConfigKey::Ipv6Only => {
            let parsed = value
                .as_deref()
                .map(|v| v.parse::<bool>().map_err(|_| format!("Invalid bool: {v}")))
                .transpose()?;
            config.update_ipv6(parsed);
        }
    }
    Ok(())
}

fn persist_config(config: &ConfigStore) -> Result<(), String> {
    config.save().map_err(|e| match e {
        ConfigError::Io(err) => err.to_string(),
        ConfigError::Parse(err) => err.to_string(),
        ConfigError::Invalid(msg) => msg,
    })
}

fn run_preset(preset: &PresetRecord) -> Result<(), String> {
    if preset.args.is_empty() {
        return Err("Preset is empty".into());
    }
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let status = ProcessCommand::new(exe)
        .args(&preset.args)
        .status()
        .map_err(|e| e.to_string())?;
    process::exit(status.code().unwrap_or(1));
}

fn display_opt_float(value: Option<f64>) -> String {
    value
        .map(|v| format!("{:.3}", v))
        .unwrap_or_else(|| "<unset>".into())
}
