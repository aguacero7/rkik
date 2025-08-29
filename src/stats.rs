use crate::domain::ntp::ProbeResult;
#[cfg(feature = "json")]
use serde::Serialize;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Stats {
    pub count: usize,
    pub offset_avg: f64,
    pub offset_min: f64,
    pub offset_max: f64,
    pub rtt_avg: f64,
}

pub fn compute_stats(results: &[ProbeResult]) -> Stats {
    let count = results.len();
    let offset_avg = results.iter().map(|r| r.offset_ms).sum::<f64>() / count as f64;
    let offset_min = results
        .iter()
        .map(|r| r.offset_ms)
        .fold(f64::INFINITY, f64::min);
    let offset_max = results
        .iter()
        .map(|r| r.offset_ms)
        .fold(f64::NEG_INFINITY, f64::max);
    let rtt_avg = results.iter().map(|r| r.rtt_ms).sum::<f64>() / count as f64;
    Stats {
        count,
        offset_avg,
        offset_min,
        offset_max,
        rtt_avg,
    }
}
