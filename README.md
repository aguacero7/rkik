# RKIK - Rusty Klock Inspection Kit
![CI](https://github.com/aguacero7/rkik/actions/workflows/ci.yml/badge.svg)
**RKIK** is a command-line tool for querying NTP servers and comparing clock offsets, written in Rust.

It allows you to:
- Query a single NTP server
- Compare two NTP servers
- Display human-readable or JSON output
- Use positional or flagged arguments (`--server`, or directly passing the hostname/IP)
- Enable verbose output for advanced details (stratum, reference ID)

---

## 🚀 Features

- ✅ Query any NTP server (IPv4 or IPv6)
- ✅ Compare offsets between two servers
- ✅ Output formats: human-readable or JSON
- ✅ Verbose mode for advanced metadata
- ✅ Accepts both FQDN and raw IP addresses
- ✅ Argument parsing via `clap` with fallback positional support

---

## 🛠 Installation

```bash
git clone <repository-url>
cd rkik
cargo build --release
sudo cp target/release/rkik /usr/local/bin
rkik --help
```

---

## 🧪 Usage Examples

| Command                                          | Description                                |
|--------------------------------------------------|--------------------------------------------|
| `rkik pool.ntp.org`                              | Query an NTP server (positional)           |
| `rkik --server pool.ntp.org`                     | Same as above, explicit flag               |
| `rkik --server time.google.com --verbose`        | Verbose query output                       |
| `rkik --server time.cloudflare.com --format json`| JSON output for a single server            |
| `rkik --compare pool.ntp.org time.google.com`    | Compare two servers                        |
| `rkik --compare ntp1 ntp2 --format json`         | Compare servers with JSON output           |

---

## 📦 Output Examples

**Human-readable:**
```
Server: time.google.com
IP: 216.239.35.0
UTC Time: Mon, 27 May 2024 13:45:00 +0000
Local Time: 2024-05-27 15:45:00
Clock Offset: -1.203 ms
Round Trip Delay: 2.320 ms
```

**JSON:**
```json
{
  "server": "time.google.com",
  "ip": "216.239.35.0",
  "utc_time": "2024-05-27T13:45:00Z",
  "local_time": "2024-05-27 15:45:00",
  "offset_ms": -1.203,
  "rtt_ms": 2.320,
  "stratum": 1,
  "reference_id": "GOOG"
}
```

---
