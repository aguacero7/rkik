use crate::domain::ntp::ProbeResult;
#[cfg(all(feature = "ptp", target_os = "linux"))]
use crate::domain::ptp::PtpProbeResult;
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

#[cfg(all(feature = "ptp", target_os = "linux"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PtpStats {
    pub count: usize,
    pub offset_avg_ns: f64,
    pub offset_min_ns: f64,
    pub offset_max_ns: f64,
    pub mean_path_delay_avg_ns: f64,
}

#[cfg(all(feature = "ptp", target_os = "linux"))]
pub fn compute_ptp_stats(results: &[PtpProbeResult]) -> PtpStats {
    if results.is_empty() {
        return PtpStats {
            count: 0,
            offset_avg_ns: 0.0,
            offset_min_ns: 0.0,
            offset_max_ns: 0.0,
            mean_path_delay_avg_ns: 0.0,
        };
    }

    let count = results.len();
    let offset_avg_ns = results.iter().map(|r| r.offset_ns as f64).sum::<f64>() / count as f64;
    let offset_min_ns = results
        .iter()
        .map(|r| r.offset_ns as f64)
        .fold(f64::INFINITY, f64::min);
    let offset_max_ns = results
        .iter()
        .map(|r| r.offset_ns as f64)
        .fold(f64::NEG_INFINITY, f64::max);
    let mean_path_delay_avg_ns = results
        .iter()
        .map(|r| r.mean_path_delay_ns as f64)
        .sum::<f64>()
        / count as f64;

    PtpStats {
        count,
        offset_avg_ns,
        offset_min_ns,
        offset_max_ns,
        mean_path_delay_avg_ns,
    }
}
