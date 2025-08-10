use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "rkik")]
#[command(about = "Rusty Klock Inspection Kit - NTP Query and Compare Tool")]
#[command(long_about = Some(
    "Query and compare NTP servers from the CLI.\n\
     \n\
     Examples:\n\
       rkik 0.pool.ntp.org\n\
       rkik --server time.google.com --verbose\n\
       rkik --compare ntp1 ntp2 --format json\n\
     \n\
     Supports both IPv4 and IPv6, positional or flagged arguments."
))]
pub struct Args {
    /// Query a single NTP server
    #[arg(short, long)]
    pub server: Option<String>,

    /// Compare two servers
    #[arg(short='C',long, num_args = 2..10)]
    pub compare: Option<Vec<String>>,

    /// Show detailed output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Output format: "text" or "json"
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Use IPv6 resolution only
    #[arg(short = '6', long)]
    pub ipv6: bool,

    /// Positional server name or IP (used if --server not provided)
    #[arg(index = 1)]
    pub positional: Option<String>,
}
