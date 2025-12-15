#![cfg(feature = "ptp")]

use chrono::Utc;
#[cfg(feature = "json")]
use serde::Serialize;

use crate::domain::ptp::{PtpDiagnostics, PtpProbeResult};
use crate::error::RkikError;
use crate::stats::PtpStats;

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonTarget<'a> {
    name: &'a str,
    ip: String,
    event_port: u16,
    general_port: u16,
    domain: u8,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonClockQuality {
    clock_class: u8,
    clock_accuracy: u8,
    offset_scaled_log_variance: u16,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonPacketStats {
    sync_sent: u32,
    sync_received: u32,
    follow_up_received: u32,
    delay_req_sent: u32,
    delay_resp_received: u32,
    announce_received: u32,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonDiagnostics {
    master_port_identity: String,
    hardware_timestamping: bool,
    timestamp_mode: String,
    steps_removed: u16,
    current_utc_offset: i16,
    current_utc_offset_valid: bool,
    leap59: bool,
    leap61: bool,
    time_traceable: bool,
    frequency_traceable: bool,
    ptp_timescale: bool,
    measurement_duration_ms: f64,
    packet_stats: JsonPacketStats,
}

#[cfg(feature = "json")]
impl From<&PtpDiagnostics> for JsonDiagnostics {
    fn from(diag: &PtpDiagnostics) -> Self {
        JsonDiagnostics {
            master_port_identity: diag.master_port_identity.to_string(),
            hardware_timestamping: diag.hardware_timestamping,
            timestamp_mode: diag.timestamp_mode.clone(),
            steps_removed: diag.steps_removed,
            current_utc_offset: diag.current_utc_offset,
            current_utc_offset_valid: diag.current_utc_offset_valid,
            leap59: diag.leap59,
            leap61: diag.leap61,
            time_traceable: diag.time_traceable,
            frequency_traceable: diag.frequency_traceable,
            ptp_timescale: diag.ptp_timescale,
            measurement_duration_ms: diag.measurement_duration_ms,
            packet_stats: JsonPacketStats {
                sync_sent: diag.packet_stats.sync_sent,
                sync_received: diag.packet_stats.sync_received,
                follow_up_received: diag.packet_stats.follow_up_received,
                delay_req_sent: diag.packet_stats.delay_req_sent,
                delay_resp_received: diag.packet_stats.delay_resp_received,
                announce_received: diag.packet_stats.announce_received,
            },
        }
    }
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonProbe<'a> {
    target: JsonTarget<'a>,
    offset_ns: i64,
    mean_path_delay_ns: i64,
    master_identity: String,
    clock_quality: JsonClockQuality,
    time_source: String,
    utc: String,
    local: String,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<JsonDiagnostics>,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonRun<'a> {
    schema_version: u8,
    protocol: &'static str,
    run_ts: String,
    results: Vec<JsonProbe<'a>>,
}

/// Serialize full PTP results into JSON.
pub fn to_json(
    results: &[PtpProbeResult],
    pretty: bool,
    verbose: bool,
) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let probes: Vec<JsonProbe<'_>> = results
            .iter()
            .map(|r| JsonProbe {
                target: JsonTarget {
                    name: &r.target.name,
                    ip: r.target.ip.to_string(),
                    event_port: r.target.event_port,
                    general_port: r.target.general_port,
                    domain: r.target.domain,
                },
                offset_ns: r.offset_ns,
                mean_path_delay_ns: r.mean_path_delay_ns,
                master_identity: r.master_identity.to_string(),
                clock_quality: JsonClockQuality {
                    clock_class: r.clock_quality.clock_class,
                    clock_accuracy: r.clock_quality.clock_accuracy,
                    offset_scaled_log_variance: r.clock_quality.offset_scaled_log_variance,
                },
                time_source: r.time_source.to_string(),
                utc: r.utc.to_rfc3339(),
                local: r.local.to_string(),
                timestamp: r.timestamp,
                diagnostics: if verbose {
                    r.diagnostics.as_ref().map(JsonDiagnostics::from)
                } else {
                    None
                },
            })
            .collect();

        let run = JsonRun {
            schema_version: 1,
            protocol: "ptp",
            run_ts: Utc::now().to_rfc3339(),
            results: probes,
        };

        if pretty {
            serde_json::to_string_pretty(&run).map_err(|e| RkikError::Other(e.to_string()))
        } else {
            serde_json::to_string(&run).map_err(|e| RkikError::Other(e.to_string()))
        }
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = results;
        let _ = pretty;
        let _ = verbose;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonSimpleProbe<'a> {
    utc: String,
    name: &'a str,
    domain: u8,
}

/// Serialize a list of probes into a compact JSON array.
pub fn to_short_json(results: &[PtpProbeResult], pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let items: Vec<JsonSimpleProbe<'_>> = results
            .iter()
            .map(|r| JsonSimpleProbe {
                utc: r.utc.to_rfc3339(),
                name: &r.target.name,
                domain: r.target.domain,
            })
            .collect();
        if pretty {
            serde_json::to_string_pretty(&items)
                .map_err(|e| RkikError::Other(format!("json encode: {}", e)))
        } else {
            serde_json::to_string(&items)
                .map_err(|e| RkikError::Other(format!("json encode: {}", e)))
        }
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = results;
        let _ = pretty;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

/// Serialize a single probe into a compact JSON line.
pub fn probe_to_short_json(result: &PtpProbeResult) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let probe = JsonSimpleProbe {
            utc: result.utc.to_rfc3339(),
            name: &result.target.name,
            domain: result.target.domain,
        };
        serde_json::to_string(&probe).map_err(|e| RkikError::Other(format!("json encode: {}", e)))
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = result;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonStatsEntry {
    name: String,
    #[serde(flatten)]
    stats: PtpStats,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonStatsSummary {
    schema_version: u8,
    stats: Vec<JsonStatsEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_avg_drift_ns: Option<f64>,
}

/// Serialize a list of statistics entries to JSON.
pub fn stats_list_to_json(list: &[(String, PtpStats)], pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let entries: Vec<JsonStatsEntry> = list
            .iter()
            .map(|(name, stats)| JsonStatsEntry {
                name: name.clone(),
                stats: stats.clone(),
            })
            .collect();

        let drift = if entries.len() > 1 {
            let min = entries
                .iter()
                .map(|s| s.stats.offset_avg_ns)
                .fold(f64::INFINITY, f64::min);
            let max = entries
                .iter()
                .map(|s| s.stats.offset_avg_ns)
                .fold(f64::NEG_INFINITY, f64::max);
            Some(max - min)
        } else {
            None
        };

        let summary = JsonStatsSummary {
            schema_version: 1,
            stats: entries,
            max_avg_drift_ns: drift,
        };

        if pretty {
            serde_json::to_string_pretty(&summary).map_err(|e| RkikError::Other(e.to_string()))
        } else {
            serde_json::to_string(&summary).map_err(|e| RkikError::Other(e.to_string()))
        }
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = list;
        let _ = pretty;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

/// Serialize a single statistics entry.
pub fn stats_to_json(name: &str, stats: &PtpStats, pretty: bool) -> Result<String, RkikError> {
    stats_list_to_json(&[(name.to_string(), stats.clone())], pretty)
}
