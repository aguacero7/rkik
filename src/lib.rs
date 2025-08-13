//! rkik library exposing reusable NTP querying utilities.

pub mod adapters;
pub mod domain;
mod error;
pub mod fmt;
pub mod services;

pub use domain::ntp::{ProbeResult, Target};
pub use error::RkikError;
pub use services::compare::compare_many;
pub use services::query::query_one;
