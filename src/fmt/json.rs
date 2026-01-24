use chrono::Utc;
#[cfg(feature = "json")]
use serde::Serialize;

use crate::domain::ntp::ProbeResult;
use crate::error::RkikError;
use crate::stats::Stats;

#[cfg(all(feature = "json", feature = "nts"))]
use crate::adapters::nts_client::{NtsKeData, NtsValidationOutcome};

// NtsValidationOutcome, NtsError, and NtsErrorKind already derive Serialize,
// so we can serialize them directly without wrapper types.

#[cfg(feature = "json")]
#[derive(Serialize)]
pub struct JsonProbe {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stratum: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
    pub utc: String,
    pub local: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    pub authenticated: bool,
    #[cfg(feature = "nts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nts_ke_data: Option<NtsKeData>,
    #[cfg(feature = "nts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nts: Option<NtsValidationOutcome>,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
pub struct JsonRun {
    pub schema_version: u8,
    pub run_ts: String,
    pub results: Vec<JsonProbe>,
}

/// Serialize probe results into JSON string.
#[allow(unused_variables)]
pub fn to_json(results: &[ProbeResult], pretty: bool, verbose: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let probes = results
            .iter()
            .map(|r| {
                #[cfg(feature = "nts")]
                let nts_output = if verbose {
                    r.nts_validation.clone()
                } else {
                    None
                };

                JsonProbe {
                    name: r.target.name.clone(),
                    ip: r.target.ip.to_string(),
                    port: r.target.port,
                    offset_ms: r.offset_ms,
                    rtt_ms: r.rtt_ms,
                    utc: r.utc.to_rfc3339(),
                    local: r.local.format("%Y-%m-%d %H:%M:%S").to_string(),
                    stratum: if verbose { Some(r.stratum) } else { None },
                    ref_id: if verbose {
                        Some(r.ref_id.clone())
                    } else {
                        None
                    },
                    timestamp: if verbose { Some(r.timestamp) } else { None },
                    authenticated: r.authenticated,
                    #[cfg(feature = "nts")]
                    nts_ke_data: if verbose { r.nts_ke_data.clone() } else { None },
                    #[cfg(feature = "nts")]
                    nts: nts_output,
                }
            })
            .collect();

        let run = JsonRun {
            schema_version: 1,
            run_ts: Utc::now().to_rfc3339(),
            results: probes,
        };

        let text = if pretty {
            serde_json::to_string_pretty(&run).map_err(|e| RkikError::Other(e.to_string()))?
        } else {
            serde_json::to_string(&run).map_err(|e| RkikError::Other(e.to_string()))?
        };
        Ok(text)
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
pub struct JsonSimpleProbe {
    pub utc: String,
    pub name: String,
    pub port: u16,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonSimpleRun {
    schema_version: u8,
    run_ts: String,
    results: Vec<JsonSimpleProbe>,
}

/// Serialize simple probe results (timestamp and IP only).
#[allow(unused_variables)]
pub fn simple_to_json(results: &[ProbeResult], pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let probes = results
            .iter()
            .map(|r| JsonSimpleProbe {
                utc: r.utc.to_rfc3339(),
                name: r.target.name.clone(),
                port: r.target.port,
            })
            .collect();

        let run = JsonSimpleRun {
            schema_version: 1,
            run_ts: Utc::now().to_rfc3339(),
            results: probes,
        };

        let text = if pretty {
            serde_json::to_string_pretty(&run).map_err(|e| RkikError::Other(e.to_string()))?
        } else {
            serde_json::to_string(&run).map_err(|e| RkikError::Other(e.to_string()))?
        };
        Ok(text)
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = results;
        let _ = pretty;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonStatsEntry {
    name: String,
    #[serde(flatten)]
    stats: Stats,
}

#[cfg(feature = "json")]
#[derive(Serialize)]
struct JsonStatsSummary {
    schema_version: u8,
    stats: Vec<JsonStatsEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_avg_drift: Option<f64>,
}

/// Serialize statistics into JSON string.
#[allow(unused_variables)]
pub fn stats_list_to_json(list: &[(String, Stats)], pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let stats: Vec<JsonStatsEntry> = list
            .iter()
            .map(|(name, st)| JsonStatsEntry {
                name: name.clone(),
                stats: st.clone(),
            })
            .collect();

        let drift = if stats.len() > 1 {
            let min = stats
                .iter()
                .map(|s| s.stats.offset_avg)
                .fold(f64::INFINITY, f64::min);
            let max = stats
                .iter()
                .map(|s| s.stats.offset_avg)
                .fold(f64::NEG_INFINITY, f64::max);
            Some(max - min)
        } else {
            None
        };

        let summary = JsonStatsSummary {
            schema_version: 1,
            stats,
            max_avg_drift: drift,
        };

        let text = if pretty {
            serde_json::to_string_pretty(&summary).map_err(|e| RkikError::Other(e.to_string()))?
        } else {
            serde_json::to_string(&summary).map_err(|e| RkikError::Other(e.to_string()))?
        };
        Ok(text)
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = list;
        let _ = pretty;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

/// Serialize a single statistics entry.
#[allow(unused_variables)]
pub fn stats_to_json(name: &str, stats: &Stats, pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        stats_list_to_json(&[(name.to_string(), stats.clone())], pretty)
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = name;
        let _ = stats;
        let _ = pretty;
        Err(RkikError::Other("json feature disabled".into()))
    }
}

/// Serialize a single probe into a compact one-line JSON string (no envelope).
pub fn probe_to_short_json(r: &ProbeResult) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let p = JsonSimpleProbe {
            utc: r.utc.to_rfc3339(),
            name: r.target.name.clone(),
            port: r.target.port,
        };
        let s = serde_json::to_string(&p)
            .map_err(|e| RkikError::Other(format!("json encode: {}", e)))?;
        Ok(s)
    }
    #[cfg(not(feature = "json"))]
    {
        Err(RkikError::Other("json feature disabled".into()))
    }
}

/// Serialize a list of probes into a compact JSON array (no envelope).
pub fn to_short_json(results: &[ProbeResult], pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let items: Vec<JsonSimpleProbe> = results
            .iter()
            .map(|r| JsonSimpleProbe {
                utc: r.utc.to_rfc3339(),
                name: r.target.name.clone(),
                port: r.target.port,
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
        Err(RkikError::Other("json feature disabled".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ntp::{ProbeResult, Target};
    use std::net::IpAddr;

    fn sample_probe() -> ProbeResult {
        let utc = chrono::Utc::now();
        let local: chrono::DateTime<chrono::Local> = chrono::DateTime::from(utc);
        ProbeResult {
            target: Target {
                name: "example".into(),
                ip: "127.0.0.1".parse::<IpAddr>().unwrap(),
                port: 123,
            },
            offset_ms: 0.0,
            rtt_ms: 0.5,
            stratum: 1,
            ref_id: "LOCL".into(),
            utc,
            local,
            timestamp: 1,
            authenticated: false,
            #[cfg(feature = "nts")]
            nts_ke_data: None,
            #[cfg(feature = "nts")]
            nts_validation: None,
        }
    }

    #[test]
    fn timestamp_hidden_when_not_verbose() {
        let probe = sample_probe();
        let json = to_json(std::slice::from_ref(&probe), false, false).unwrap();
        assert!(
            !json.contains("timestamp"),
            "timestamp should be omitted when not verbose: {json}"
        );
        let json_verbose = to_json(std::slice::from_ref(&probe), false, true).unwrap();
        assert!(
            json_verbose.contains("\"timestamp\":1"),
            "timestamp should appear when verbose: {json_verbose}"
        );
    }
}
