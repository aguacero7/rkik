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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stratum: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
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
pub fn to_json(results: &[ProbeResult], pretty: bool, verbose: bool) -> Result<String, RkikError> {
    #[cfg(feature = "json")]
    {
        let probes = results
            .iter()
            .map(|r| JsonProbe {
                name: r.target.name.clone(),
                ip: r.target.ip.to_string(),
                offset_ms: r.offset_ms,
                rtt_ms: r.rtt_ms,
                utc: r.utc.to_rfc3339(),
                local: r.local.format("%Y-%m-%d %H:%M:%S").to_string(),
                stratum: if verbose { Some(r.stratum) } else { None },
                ref_id: if verbose { Some(r.ref_id.clone()) } else { None },
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

