pub mod cli;
pub mod core;
pub mod errors;
pub mod ntp;
pub mod output;

pub use cli::args::Args;
pub use core::{compare, query};
pub use ntp::resolver::resolve_ip as resolve_ip_for_mode;
