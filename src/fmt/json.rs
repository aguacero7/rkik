use chrono::Utc;
#[cfg(feature = "json")]
use serde::Serialize;

use crate::domain::ntp::ProbeResult;
use crate::error::RkikError;

#[cfg(feature = "json")]
#[derive(Serialize)]
pub struct JsonProbe {
    pub name: String,
    pub ip: String,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    pub stratum: u8,
    pub ref_id: String,
    pub utc: String,
    pub local: String,
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
pub fn to_json(results: &[ProbeResult], pretty: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let probes = results
            .iter()
            .map(|r| JsonProbe {
                name: r.target.name.clone(),
                ip: r.target.ip.to_string(),
                offset_ms: r.offset_ms,
                rtt_ms: r.rtt_ms,
                stratum: r.stratum,
                ref_id: r.ref_id.clone(),
                utc: r.utc.to_rfc3339(),
                local: r.local.format("%Y-%m-%d %H:%M:%S").to_string(),
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
        Err(RkikError::Other("json feature disabled".into()))
    }
}
