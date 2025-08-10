use console::Term;
use std::net::IpAddr;

use crate::ntp::model::NtpResult;

use super::formatter::{print_compare_json, print_compare_text, print_json, print_text};

pub fn write_query(term: &Term, server: &str, result: &NtpResult, format: &str, verbose: bool) {
    match format {
        "json" => print_json(result, server),
        _ => print_text(term, server, result, verbose),
    }
}

pub fn write_compare(term: &Term, results: &[(String, IpAddr, f64)], format: &str) {
    match format {
        "json" => print_compare_json(results),
        _ => print_compare_text(term, results),
    }
}
