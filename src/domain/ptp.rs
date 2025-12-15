//! PTP (Precision Time Protocol) domain types.
//!
//! This module defines types for PTP (IEEE 1588-2019) time synchronization,
//! which provides sub-microsecond precision timing using hardware timestamping.

use chrono::{DateTime, Local, Utc};

#[cfg(feature = "json")]
use serde::Serialize;

/// Clock Identity (EUI-64) - 8 bytes identifying a PTP clock
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct ClockIdentity(pub [u8; 8]);

impl ClockIdentity {
    /// Create a new ClockIdentity from bytes
    pub fn new(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Format as hex string (e.g., "00:1B:21:AB:CD:EF:00:01")
    pub fn to_hex_string(&self) -> String {
        self.0
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":")
    }
}

impl std::fmt::Display for ClockIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// Port Identity - combination of clock identity and port number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PortIdentity {
    pub clock_identity: ClockIdentity,
    pub port_number: u16,
}

impl std::fmt::Display for PortIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.clock_identity, self.port_number)
    }
}

/// Time Source - origin of the time used by the Grandmaster
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum TimeSource {
    AtomicClock,
    Gps,
    TerrestrialRadio,
    Ptp,
    Ntp,
    HandSet,
    Other,
    InternalOscillator,
}

impl std::fmt::Display for TimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSource::AtomicClock => write!(f, "ATOMIC_CLOCK"),
            TimeSource::Gps => write!(f, "GPS"),
            TimeSource::TerrestrialRadio => write!(f, "TERRESTRIAL_RADIO"),
            TimeSource::Ptp => write!(f, "PTP"),
            TimeSource::Ntp => write!(f, "NTP"),
            TimeSource::HandSet => write!(f, "HAND_SET"),
            TimeSource::Other => write!(f, "OTHER"),
            TimeSource::InternalOscillator => write!(f, "INTERNAL_OSCILLATOR"),
        }
    }
}

/// Clock Quality - describes the quality of a PTP clock
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct ClockQuality {
    /// Clock class (6 = primary reference, 7 = default)
    pub clock_class: u8,
    /// Clock accuracy (higher = better, 0x20 = within 25ns)
    pub clock_accuracy: u8,
    /// Offset scaled log variance (stability measure)
    pub offset_scaled_log_variance: u16,
}

impl ClockQuality {
    /// Get human-readable accuracy description
    pub fn accuracy_description(&self) -> &'static str {
        match self.clock_accuracy {
            0x20 => "within 25 ns",
            0x21 => "within 100 ns",
            0x22 => "within 250 ns",
            0x23 => "within 1 µs",
            0x24 => "within 2.5 µs",
            0x25 => "within 10 µs",
            0x26 => "within 25 µs",
            0x27 => "within 100 µs",
            0x28 => "within 250 µs",
            0x29 => "within 1 ms",
            0x2A => "within 2.5 ms",
            0x2B => "within 10 ms",
            0x2C => "within 25 ms",
            0x2D => "within 100 ms",
            0x2E => "within 250 ms",
            0x2F => "within 1 s",
            0x30 => "within 10 s",
            0x31 => "> 10 s",
            _ => "unknown",
        }
    }

    /// Get human-readable clock class description
    pub fn class_description(&self) -> &'static str {
        match self.clock_class {
            6 => "Primary reference (GPS/Atomic)",
            7 => "Primary reference (default)",
            13 => "Application-specific time source",
            14 => "Alternative PTP profile",
            52 => "Degraded primary reference (holdover within spec)",
            58 => "Degraded primary reference (out of holdover spec)",
            187 => "Default slave-only",
            248 => "Default (no external reference)",
            255 => "Slave-only",
            _ => "Other",
        }
    }
}

/// PTP target specification
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PtpTarget {
    /// Server hostname or IP address
    pub name: String,
    /// Resolved IP address
    pub ip: std::net::IpAddr,
    /// PTP domain number (default: 0)
    pub domain: u8,
    /// Event message port (default: 319)
    pub event_port: u16,
    /// General message port (default: 320)
    pub general_port: u16,
}

/// Packet statistics for PTP exchange
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PacketStats {
    /// Number of Sync messages sent
    pub sync_sent: u32,
    /// Number of Sync messages received
    pub sync_received: u32,
    /// Number of Follow_Up messages received
    pub follow_up_received: u32,
    /// Number of Delay_Req messages sent
    pub delay_req_sent: u32,
    /// Number of Delay_Resp messages received
    pub delay_resp_received: u32,
    /// Number of Announce messages received
    pub announce_received: u32,
}

/// Detailed PTP diagnostics (verbose mode)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PtpDiagnostics {
    /// Master clock port identity
    pub master_port_identity: PortIdentity,
    /// Whether hardware timestamping was used
    pub hardware_timestamping: bool,
    /// Timestamp mode description
    pub timestamp_mode: String,
    /// Steps removed from Grandmaster
    pub steps_removed: u16,
    /// Current UTC offset (TAI - UTC in seconds)
    pub current_utc_offset: i16,
    /// Whether UTC offset is valid
    pub current_utc_offset_valid: bool,
    /// Whether leap second is pending
    pub leap59: bool,
    pub leap61: bool,
    /// Whether time is traceable to primary reference
    pub time_traceable: bool,
    /// Whether frequency is traceable to primary reference
    pub frequency_traceable: bool,
    /// PTP timescale flag (true = PTP, false = ARB)
    pub ptp_timescale: bool,
    /// Packet statistics
    pub packet_stats: PacketStats,
    /// Measurement duration in milliseconds
    pub measurement_duration_ms: f64,
}

/// Result of a PTP time query
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PtpProbeResult {
    /// Target specification
    pub target: PtpTarget,
    /// Clock offset in nanoseconds (positive = local clock ahead)
    pub offset_ns: i64,
    /// Mean path delay in nanoseconds
    pub mean_path_delay_ns: i64,
    /// Grandmaster clock identity
    pub master_identity: ClockIdentity,
    /// Clock quality of Grandmaster
    pub clock_quality: ClockQuality,
    /// Time source of Grandmaster
    pub time_source: TimeSource,
    /// UTC timestamp from PTP
    pub utc: DateTime<Utc>,
    /// Local timestamp
    pub local: DateTime<Local>,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
    /// Detailed diagnostics (only in verbose mode)
    pub diagnostics: Option<PtpDiagnostics>,
}

impl PtpProbeResult {
    /// Get offset in milliseconds (for consistency with NTP results)
    pub fn offset_ms(&self) -> f64 {
        self.offset_ns as f64 / 1_000_000.0
    }

    /// Get mean path delay in milliseconds
    pub fn mean_path_delay_ms(&self) -> f64 {
        self.mean_path_delay_ns as f64 / 1_000_000.0
    }

    /// Get offset in microseconds
    pub fn offset_us(&self) -> f64 {
        self.offset_ns as f64 / 1_000.0
    }

    /// Get mean path delay in microseconds
    pub fn mean_path_delay_us(&self) -> f64 {
        self.mean_path_delay_ns as f64 / 1_000.0
    }

    /// Check if local clock is ahead of PTP time
    pub fn is_ahead(&self) -> bool {
        self.offset_ns > 0
    }

    /// Check if local clock is behind PTP time
    pub fn is_behind(&self) -> bool {
        self.offset_ns < 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_identity_display() {
        let id = ClockIdentity([0x00, 0x1B, 0x21, 0xAB, 0xCD, 0xEF, 0x00, 0x01]);
        assert_eq!(id.to_hex_string(), "00:1B:21:AB:CD:EF:00:01");
    }

    #[test]
    fn test_port_identity_display() {
        let port = PortIdentity {
            clock_identity: ClockIdentity([0x00, 0x1B, 0x21, 0xAB, 0xCD, 0xEF, 0x00, 0x01]),
            port_number: 1,
        };
        assert_eq!(port.to_string(), "00:1B:21:AB:CD:EF:00:01:1");
    }

    #[test]
    fn test_offset_conversions() {
        let result = PtpProbeResult {
            target: PtpTarget {
                name: "test".to_string(),
                ip: "127.0.0.1".parse().unwrap(),
                domain: 0,
                event_port: 319,
                general_port: 320,
            },
            offset_ns: 1_500_000,
            mean_path_delay_ns: 500_000,
            master_identity: ClockIdentity([0; 8]),
            clock_quality: ClockQuality {
                clock_class: 6,
                clock_accuracy: 0x20,
                offset_scaled_log_variance: 0,
            },
            time_source: TimeSource::Gps,
            utc: Utc::now(),
            local: Local::now(),
            timestamp: 0,
            diagnostics: None,
        };

        assert_eq!(result.offset_ms(), 1.5);
        assert_eq!(result.offset_us(), 1500.0);
        assert_eq!(result.mean_path_delay_ms(), 0.5);
        assert_eq!(result.mean_path_delay_us(), 500.0);
        assert!(result.is_ahead());
        assert!(!result.is_behind());
    }

    #[test]
    fn test_clock_quality_descriptions() {
        let quality = ClockQuality {
            clock_class: 6,
            clock_accuracy: 0x20,
            offset_scaled_log_variance: 0,
        };

        assert_eq!(
            quality.class_description(),
            "Primary reference (GPS/Atomic)"
        );
        assert_eq!(quality.accuracy_description(), "within 25 ns");
    }
}
