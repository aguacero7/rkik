# RKIK - Rusty Klock Inspection Kit
![Build & Tests](https://github.com/aguacero7/rkik/actions/workflows/ci-test-n-build.yml/badge.svg)
<br>
Most systems rely on a daemon (like chronyd or ntpd) to synchronize time. But what if you just want to **inspect** the current offset between your system clock and one or more time servers - without root, without installing anything heavy and in a simple CLI command ?

**RKIK** is a Rust-based CLI tool designed for **stateless and passive inspection of NTP, NTS, and PTP sources**, just as `dig` or `ping` are for DNS and ICMP.

**Link to  [Documentation page](https://aguacero7.github.io/rkik/)**

---

## Key features

- **Protocol coverage**
  - NTP probes over IPv4/IPv6
  - **NTS (RFC 8915)** authenticated sessions with full TLS/NTS-KE diagnostics
- **PTP (IEEE 1588-2019)** measurements (Linux-only), including master identity, clock quality, packet stats, and diagnostics in both text and JSON formats
- **Flexible output**: human-readable, verbose, simple/short, JSON, or compact JSON lines
- **Compare / monitoring**: asynchronous comparison across any number of targets, plugin/Nagios output with thresholds, and continuous/infinite sampling modes
- **Ergonomics**: `host[:port]` parsing (including `[IPv6]:port`), colorized or plain text, JSON pretty-print, and optional one-shot system sync
- **Developer-friendly**: reusable library API, async-friendly design, and a Docker lab for local NTP/PTP testing

---
## Installation

### Linux
```bash
# Download rkik-linux-x86_64.tar.gz on https://github.com/aguacero7/rkik/releases/latest
tar xvfz rkik-linux-x86_64.tar.gz 
cd rkik-linux-x86_64/
sudo cp ./rkik /usr/local/bin
```
### Red-hat Like Systems (CentOS, Fedora, RHEL, Alma,..)
```bash
# Download rkik-<X.Y.Z-R>.x86_64.rpm  on https://github.com/aguacero7/rkik/releases/latest
rpm -U <rkik-<X.Y.Z-R>.x86_64.rpm>
# OR
dnf install <rkik-<X.Y.Z-R>.x86_64.rpm>
# OR
yum install <rkik-<X.Y.Z-R>.x86_64.rpm>
```
### Debian-like Systems
```bash
# Download rkik-<X.Y.Z-R>.x86_64.deb  on https://github.com/aguacero7/rkik/releases/latest
apt install <rkik-<X.Y.Z-R>.x86_64.deb>
```
### Cargo
```bash
cargo install rkik
```


### From Source : 
```bash
git clone <repository-url>
cd rkik
cargo build --release
sudo cp target/release/rkik /usr/local/bin
rkik --help
```
## Packaging status
[![Packaging status](https://repology.org/badge/vertical-allrepos/rkik.svg)](https://repology.org/project/rkik/versions)
---

## Usage Examples

| Command                                          | Description                                |
|--------------------------------------------------|--------------------------------------------|
| `rkik -V`                              | Display rkik installed version           |
| `rkik pool.ntp.org`                              | Query an NTP server (positional)           |
| `rkik pool.ntp.org -6`                              | Query an NTP server using IPv6 (positional)           |
| `rkik pool.ntp.org:123`                     | Same as above, explicit specification of a port               |
| `rkik --server time.google.com -v`        | Verbose query output                       |
| `rkik --server time.cloudflare.com -jp`| JSON output for a single server            |
| `rkik --compare pool.ntp.org time.google.com`    | Compare two servers                        |
| `rkik time.google.com -8 -j`         | Continuously query a server and display a raw json output (useful for monitoring scripts)          |
| `rkik es.pool.ntp.org -S `         | Query a server and display a short minimalist output           |
| `rkik -C ntp1 ntp2 -c 2 -i 0.1 --nocolor`         | Compare 2 servers twice with an interval of 100ms and display a nocolor output           |
| `rkik -S time.google.com --sync`         | Query a server and apply returned time to system (sync feature -> requires root or specific permissions)          |
| `rkik time.cloudflare.com --nts` | Query an NTS-enabled server with cryptographic authentication |
| `rkik time.cloudflare.com --nts -v` | NTS query with full diagnostics (handshake, cookies, TLS certificate) |
| `rkik --compare nts.ntp.se time.cloudflare.com --nts -v` | Compare multiple NTS servers with verbose output |
| `rkik --nts --format json --pretty time.cloudflare.com` | NTS query with JSON output including certificate details |
| `rkik --ptp 192.168.1.100` | Query a PTP master with default domain/ports |
| `rkik --ptp --ptp-domain 24 --ptp-event-port 3319 127.0.0.1` | Probe a lab grandmaster on custom ports (see Docker lab) |
| `rkik --ptp --compare 192.168.1.100 192.168.1.101 --format json` | Compare multiple PTP masters with JSON output |


---

## NTS (Network Time Security)

RKIK fully supports **NTS (RFC 8915)**, providing cryptographically authenticated NTP queries.

### Key Features
- **Cryptographic authentication** of NTP packets
- **TLS certificate verification** with chain of trust
- **Complete diagnostics** in verbose mode:
  - NTS-KE handshake duration
  - Cookie management (count and sizes)
  - AEAD algorithm details
  - Full TLS certificate information
- **JSON export** with all metadata
- **Compatible** with all existing features (compare, plugin mode, etc.)

### Quick Start
```bash
# Basic NTS query
rkik --nts time.cloudflare.com

# Full diagnostics with certificate details
rkik --nts --verbose time.cloudflare.com

# Compare multiple NTS servers
rkik --nts --compare time.cloudflare.com nts.netnod.se
```

### NTS Verbose Output Example
```
Server: time.cloudflare.com [NTS Authenticated]
IP: 162.159.200.123:123
UTC Time: Mon, 15 Dec 2025 10:57:17 +0000
Local Time: 2025-12-15 11:57:17
Clock Offset: 14.496 ms
Round Trip Delay: 500.000 ms
Stratum: 0
Reference ID: 162.159.200.1:123
Authenticated: Yes (NTS)

=== NTS-KE Diagnostics ===
Handshake Duration: 95.168 ms
Cookies Received: 8 cookies
AEAD Algorithm: AEAD_AES_SIV_CMAC_256
NTP Server: 162.159.200.1:123
Cookie Sizes:
  Cookie 1: 96 bytes
  Cookie 2: 96 bytes
  ... (truncated)

=== TLS Certificate ===
Subject: CN=time.cloudflare.com
Issuer: C=US, O=DigiCert Inc, OU=www.digicert.com, CN=GeoTrust TLS ECC CA G1
Valid: Feb 10 00:00:00 2025 +00:00 to Mar 12 23:59:59 2026 +00:00
Fingerprint (SHA-256):
  4b060f4d02d65f9cb50ab27239c024426c097d238a2c9e602c896288ed462119
SANs:
  - time.cloudflare.com
Signature Algorithm: 1.2.840.10045.4.3.2
Public Key Algorithm: 1.2.840.10045.2.1
```

For detailed NTS documentation, see [docs/NTS_USAGE.md](docs/NTS_USAGE.md)

---

## PTP (Precision Time Protocol)

RKIK ships a lightweight IEEE 1588-2019 client mode to extract master clock metadata, offsets, and diagnostics. The implementation relies on `statime`/`statime-linux` and therefore currently targets **Linux builds only** (hardware timestamping is reported but optional). When compiling for other operating systems the CLI automatically hides all `--ptp` switches.

### CLI flags

| Flag | Description |
|------|-------------|
| `--ptp` | Enable PTP mode (mutually exclusive with `--nts` and `--sync`) |
| `--ptp-domain <N>` | Domain number (default `0`) |
| `--ptp-event-port <PORT>` / `--ptp-general-port <PORT>` | Override UDP ports (default `319` / `320`) — handy for lab environments |
| `--ptp-hw-timestamp` | Request hardware timestamping; the diagnostics output reports whether HW/SW timestamps were used |

All other CLI niceties continue to work: compare mode, plugin mode, JSON/simple renderers, and verbose diagnostics.

### Quick start

```bash
# Basic probe
rkik --ptp 192.0.2.10

# Custom domain/ports (Docker lab example)
rkik --ptp --ptp-domain 24 --ptp-event-port 3319 --ptp-general-port 3320 127.0.0.1

# Verbose JSON output
rkik --ptp --verbose --format json --pretty ptp.lan
```

### Sample verbose output

```
Server: 192.0.2.10 [PTP Master]
IP: 192.0.2.10:319/320
Domain: 0
UTC Time: Mon, 15 Dec 2025 12:00:00.000000000 +0000
Local Time: 2025-12-15 13:00:00.000000000
Clock Offset: 125 ns (0.125 us)
Mean Path Delay: 450 ns (0.450 us)
Master Clock: 00:1b:21:ff:fe:8a:bc:de
Clock Class: 6 (Primary Reference)
Clock Accuracy: 0x20 (within 25 ns)
Time Source: GNSS

=== PTP Diagnostics ===
Master Port: 00:1b:21:ff:fe:8a:bc:de:1
Timestamp Mode: hardware timestamping (simulated)
Hardware Timestamping: Yes
Steps Removed: 0
Current UTC Offset: 37s (valid: true)
Traceable: time=true, freq=true
Packet Statistics:
  Sync RX: 5
  Delay Resp RX: 5
  Announce RX: 2
  Delay Req TX: 5
Measurement Duration: 1.234 ms
```

PTP data is also available through the library API (`PtpProbeResult`, `PtpQueryOptions`) for custom tooling.

---

## Local Test Environment

A ready-made Docker Compose setup is available in [`docs/TEST_ENV.md`](docs/TEST_ENV.md). It spins up three isolated NTP daemons plus a LinuxPTP grandmaster so you can exercise all RKIK modes locally:

```bash
# Start services
./scripts/test-env-up.sh

# Probe an NTP target on the remapped port
rkik 127.0.0.1:3123

# Probe the PTP master
rkik --ptp --ptp-event-port 3319 --ptp-general-port 3320 127.0.0.1

# Tear everything down
./scripts/test-env-down.sh
```

See the documentation for more customization and troubleshooting tips.

---

## Output Examples

**Human-readable:**
```
Server: time.google.com
IP: 216.239.35.4:123
UTC Time: Wed, 3 Sep 2025 09:44:43 +0000
Local Time: 2025-09-03 11:44:43
Clock Offset: -6776478.958 ms
Round Trip Delay: 33.192 ms
```

**JSON:**
```json
{
  "schema_version": 1,
  "run_ts": "2025-09-03T11:37:57.240321504+00:00",
  "results": [
    {
      "name": "time.google.com",
      "ip": "216.239.35.4",
      "port": 123,
      "offset_ms": -6774948.655983666,
      "rtt_ms": 46.76786344498396,
      "utc": "2025-09-03T09:45:02.291616786+00:00",
      "local": "2025-09-03 11:45:02",
      "timestamp": null
    }
  ]
}

```

---

## Arguments supported 
```bash
Rusty Klock Inspection Kit - NTP Query and Compare Tool

```
Usage: rkik <COMMAND>

Commands:
  ntp <target>            Run an NTP probe loop (default if no subcommand is supplied)
  compare <targets...>    Compare >= 2 servers concurrently
  ptp <target>            IEEE-1588/PTP probe (Linux-only feature flag)
  sync <target>           One-shot synchronization helper
  diag <target>           Verbose diagnostic run (single shot)
  config <subcommand>     Inspect or update defaults (timeout/format/ipv6)
  preset <subcommand>     Manage reusable argument presets
  help                    Print help for the chosen command

Each probe-oriented subcommand accepts the familiar options (`--count`, `--interval`, `--format`, `--plugin`, `--warning`, `--critical`, `--nts`, `--ipv6`, etc.). The legacy one-shot CLI still works: running `rkik <target>` without a subcommand automatically falls back to the historical parser (with a deprecation warning).

### Configuration & presets
- Defaults (timeout/format/ipv6) and presets live in `$RKIK_CONFIG_DIR/config.toml`. If the env var is unset, rkik falls back to the platform config directory (e.g. `~/.config/rkik`).
- Update defaults via `rkik config set default-timeout 3.5`, inspect with `rkik config list`, and clear with `rkik config clear default-format`.
- Store reusable scenarios with `rkik preset add nightly -- ntp pool.ntp.org --count 5`. Run them via `rkik preset run nightly`, which spawns rkik with the recorded arguments.
- JSON payloads follow the same verbosity rules as the CLI: add `-v/--verbose` to include `stratum`, `ref_id`, and `timestamp` fields; without it these keys are omitted for a leaner output.


--- 

## Example Use case
Sometimes, the monitoring system shows up with a ntp error on a server.
You don't know if the problem comes from this server or its reference.
Then you try `rkik ntp.server.local -v`
```bash
Server: ntp.server.local
IP: 192.168.1.123:123
UTC Time: Wed, 3 Sep 2025 13:06:15 +0000
Local Time: 2025-09-03 15:06:15
Clock Offset: 5000.145 ms
Round Trip Delay: 27.420 ms
Stratum: 2
Reference ID: 145.238.80.80
```

At this moment, we can assure there is an offset between our system and the distant server, we can also know which is the reference of that server.
We will check whether this reference has an offset with us or not with `rkik 145.238.80.80`
```bash
Server: 145.238.80.80
IP: 145.238.80.80:123
UTC Time: Wed, 3 Sep 2025 13:09:57 +0000
Local Time: 2025-09-03 15:09:57
Clock Offset: 5001.531 ms
Round Trip Delay: 8.804 ms
```
It does, we now can assure the problem is external to our server, we may now connect on the system to change its reference with another server.

## Plugin mode (Centreon / Nagios / Zabbix) 

Since version 1.2, rkik provides a monitoring plugin mode compatible with Centreon, Nagios and Zabbix. When enabled, rkik emits a single machine-parseable line (plugin format) and returns standard plugin exit codes.

Usage
```bash
# Query a server and output a single plugin line. Thresholds are in milliseconds.
rkik time.google.com --plugin --warning 400 --critical 1000
```

Flags
- `--plugin` : enable plugin mode.
- `--warning <MS>` : warning threshold in milliseconds (requires `--plugin`).
- `--critical <MS>` : critical threshold in milliseconds (requires `--plugin`).

Output format
- A single line is emitted, example:

RKIK OK - offset 4.006ms rtt 9.449ms from time.google.com (216.239.35.4) | offset_ms=4.006ms;400;1000;0; rtt_ms=9.449ms;;;0;
