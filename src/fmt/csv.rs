use crate::domain::ntp::ProbeResult;
use crate::error::RkikError;
use std::fmt::Write as FmtWrite;

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn to_csv(results: &[ProbeResult]) -> Result<String, RkikError> {
    let mut out = String::new();
    writeln!(&mut out, "target,stratum,offset_ms,delay_ms,timestamp")
        .map_err(|e| RkikError::Other(e.to_string()))?;

    for r in results {
        let target = escape_csv(&r.target.name);
        writeln!(
            &mut out,
            "{},{},{:.3},{:.3},{}",
            target, r.stratum, r.offset_ms, r.rtt_ms, r.timestamp
        )
        .map_err(|e| RkikError::Other(e.to_string()))?;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ntp::Target;
    use std::net::IpAddr;

    fn sample_probe(
        name: &str,
        stratum: u8,
        offset_ms: f64,
        rtt_ms: f64,
        timestamp: i64,
    ) -> ProbeResult {
        let utc = chrono::Utc::now();
        let local = chrono::DateTime::from(utc);
        ProbeResult {
            target: Target {
                name: name.into(),
                ip: "127.0.0.1".parse::<IpAddr>().unwrap(),
                port: 123,
            },
            offset_ms,
            rtt_ms,
            stratum,
            ref_id: "LOCL".into(),
            utc,
            local,
            timestamp,
            authenticated: false,
            #[cfg(feature = "nts")]
            nts_ke_data: None,
            #[cfg(feature = "nts")]
            nts_validation: None,
        }
    }

    #[test]
    fn single_target_produces_header_and_one_row() {
        let p = sample_probe("time.google.com", 1, 1.234, 15.678, 1680000000);
        let csv = to_csv(&[p]).unwrap();
        let lines: Vec<&str> = csv.trim_end().split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "target,stratum,offset_ms,delay_ms,timestamp");
        assert_eq!(lines[1], "time.google.com,1,1.234,15.678,1680000000");
    }

    #[test]
    fn multiple_targets_produces_header_and_multiple_rows() {
        let p1 = sample_probe("time.google.com", 1, 1.234, 15.678, 1680000000);
        let p2 = sample_probe("pool.ntp.org", 2, -2.500, 20.000, 1680000001);
        let csv = to_csv(&[p1, p2]).unwrap();
        let lines: Vec<&str> = csv.trim_end().split('\n').collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "target,stratum,offset_ms,delay_ms,timestamp");
        assert_eq!(lines[1], "time.google.com,1,1.234,15.678,1680000000");
        assert_eq!(lines[2], "pool.ntp.org,2,-2.500,20.000,1680000001");
    }

    #[test]
    fn fields_with_special_characters_are_escaped() {
        let p = sample_probe("server,with\"quotes\nand,commas", 3, 0.0, 0.0, 0);
        let csv = to_csv(&[p]).unwrap();
        assert_eq!(
            csv,
            "target,stratum,offset_ms,delay_ms,timestamp\n\"server,with\"\"quotes\nand,commas\",3,0.000,0.000,0\n"
        );
    }
}
