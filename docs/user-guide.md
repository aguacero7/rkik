# RKIK — User Guide

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Output Formats](#output-formats)
- [NTS — Network Time Security](#nts--network-time-security)
- [PTP — Precision Time Protocol](#ptp--precision-time-protocol)
- [Plugin Mode (Nagios / Centreon / Zabbix)](#plugin-mode-nagios--centreon--zabbix)
- [Local Test Environment](#local-test-environment)
- [Troubleshooting](#troubleshooting)

---

## Installation

### Cargo (recommended)

```bash
cargo install rkik
```

### Arch Linux (AUR)

```bash
# With an AUR helper
yay -S rkik-git

# Manually
git clone https://aur.archlinux.org/rkik-git.git
cd rkik-git && makepkg -si
```

### Nix / NixOS

```bash
nix shell nixpkgs#rkik
# or permanently
nix-env -iA nixpkgs.rkik
```

NixOS configuration:

```nix
{ pkgs, ... }:
{ environment.systemPackages = with pkgs; [ rkik ]; }
```

### Linux packages (DEB / RPM / tar.gz)

Download from [GitHub Releases](https://github.com/aguacero7/rkik/releases/latest):

```bash
# Debian / Ubuntu
sudo apt install ./rkik_X.Y.Z-R_amd64.deb

# Fedora / RHEL / Alma / Rocky
sudo dnf install rkik-X.Y.Z-R.x86_64.rpm
```

### From source

```bash
git clone https://github.com/aguacero7/rkik.git
cd rkik
cargo build --release
sudo install -m 0755 target/release/rkik /usr/local/bin/rkik
```

---

## Basic Usage

### Single server probe

```bash
rkik pool.ntp.org
rkik time.google.com:123
rkik [2606:4700:f1::123]:123   # IPv6 with explicit port
```

### Compare multiple servers (parallel)

```bash
rkik --compare pool.ntp.org time.google.com time.cloudflare.com
rkik --compare time1 time2 time3 --format json
```

### IPv6-only resolution

```bash
rkik -6 pool.ntp.org
```

### Continuous monitoring

```bash
# Fixed number of probes
rkik time.cloudflare.com --count 10 --interval 2

# Infinite loop (Ctrl-C to stop)
rkik time.google.com --infinite --format json

# Continuous compare — output one JSON object per line for SIEM ingestion
rkik --compare pool.ntp.org time.google.com --infinite --format json
```

### Sync system clock (requires root, Unix only)

```bash
sudo rkik time.google.com --sync
sudo rkik time.google.com --sync --dry-run   # preview without applying
```

### Colors

```bash
rkik --nocolor pool.ntp.org
NO_COLOR=1 rkik pool.ntp.org
```

### Version info

```bash
rkik --version          # short version
rkik --version -v       # verbose: features, platform, Rust compiler
```

---

## Output Formats

| Flag | Format | Description |
|------|--------|-------------|
| *(default)* | `text` | Human-readable, colorized |
| `-j` / `--json` | `json` | Full JSON, stable schema |
| `-S` / `--short` | `simple` | Minimal text (name, offset) |
| `--format json-short` | `json-short` | Compact JSON one-liner |
| `--format csv` | `csv` | RFC 4180 compliant CSV output |
| `-p` / `--pretty` | — | Pretty-print JSON (use with `-j`) |
| `-v` / `--verbose` | — | Adds stratum, ref ID, diagnostics |

### CSV output

```bash
# Basic usage
rkik --format csv time.google.com

# Compare mode
rkik --format csv time.google.com pool.ntp.org

# Piping to file
rkik --format csv time.google.com > results.csv
```

### Error output

Text mode:
```
Error: time.example.com - dns: No IP address found for 'time.example.com'
```

JSON mode:
```json
{ "kind": "dns", "message": "No IP address found for 'time.example.com'", "target": "time.example.com" }
```

---

## NTS — Network Time Security

NTS (RFC 8915) provides cryptographic authentication for NTP. Every response is verified with AEAD encryption — spoofed or tampered packets are rejected.

> **rkik uses rkik-nts v1.0.0**, a complete self-contained RFC 8915 implementation. This is the first CLI tool with working NTS authentication verified against real public servers (`time.cloudflare.com`, `nts.ntp.se`).

### Basic NTS query

```bash
rkik --nts time.cloudflare.com
```

Output:
```
Server: time.cloudflare.com [NTS Authenticated]
IP: 162.159.200.1:123
UTC Time: Tue, 28 Apr 2026 14:32:10 +0000
Local Time: 2026-04-28 16:32:10
Clock Offset: 1.243 ms
Round Trip Delay: 14.871 ms
```

### Verbose NTS diagnostics

```bash
rkik --nts time.cloudflare.com -v
```

```
Server: time.cloudflare.com [NTS Authenticated]
...
=== NTS-KE Diagnostics ===
Handshake Duration: 87.342 ms
Cookies Received:   8 cookies
AEAD Algorithm:     AEAD_AES_SIV_CMAC_256
NTP Server:         162.159.200.1:123

=== TLS Certificate ===
Subject:    CN=time.cloudflare.com
Issuer:     C=US, O=DigiCert Inc, CN=GeoTrust TLS ECC CA G1
Valid:      2025-02-10 → 2026-03-12
Fingerprint (SHA-256): 4b060f4d...462119
SANs:       time.cloudflare.com
```

### Compare NTS servers

```bash
rkik --nts --compare nts.ntp.se time.cloudflare.com -v
```

### Custom NTS-KE port

```bash
rkik --nts --nts-port 8443 time.example.com
```

### NTS JSON output

```bash
rkik --nts time.cloudflare.com -jp
```

```json
{
  "schema_version": 1,
  "results": [{
    "name": "time.cloudflare.com",
    "offset_ms": 1.243,
    "rtt_ms": 14.871,
    "authenticated": true
  }]
}
```

### Public NTS servers

| Server | Provider | Location |
|--------|----------|----------|
| `time.cloudflare.com` | Cloudflare | Global |
| `nts.ntp.se` | Netnod | Sweden |
| `ntppool1.time.nl` | NLnet Labs | Netherlands |
| `nts.ntp.org.au` | Australian NTP Pool | Australia |

### NTS error kinds

When NTS validation fails, rkik reports a machine-readable error kind in brackets:

```
NTS-KE failed: connection timed out [timeout]
```

**Security-critical** (plugin exit code 2):

| Kind | Meaning |
|------|---------|
| `aead_failure` | AEAD verification failed — response may be tampered |
| `missing_authenticator` | Authenticator extension missing in response |
| `unauthenticated_response` | Server completed NTS-KE but sent unauthenticated NTP |
| `invalid_unique_id` | Unique Identifier mismatch (replay attack protection) |
| `invalid_origin_timestamp` | Origin timestamp check failed |

**Configuration / connection** (plugin exit code 3):

| Kind | Meaning |
|------|---------|
| `ke_handshake_failed` | TLS or NTS-KE handshake failed |
| `certificate_invalid` | TLS certificate validation failed |
| `missing_cookies` | No cookies received from server |
| `malformed_extensions` | NTS extension fields malformed |
| `timeout` | Connection timed out |
| `network` | Network-level error |

In JSON verbose mode:
```json
{
  "nts": {
    "authenticated": false,
    "error": { "kind": "aead_failure", "message": "NTS AEAD authentication failed" }
  }
}
```

### NTS troubleshooting

**`[ke_handshake_failed]`** — verify the server supports NTS and port 4460 is reachable outbound.

**`[timeout]`** — try `--timeout 15`; some servers have higher latency on the KE step.

**`[certificate_invalid]`** — check system root CAs or server certificate expiry.

**`[aead_failure]` / `[missing_authenticator]`** — potential MITM or proxy interference; try a different server.

---

## PTP — Precision Time Protocol

PTP mode (IEEE 1588-2019) is available on **Linux only** and requires the `ptp` feature (enabled by default).

```bash
# Basic probe (domain 0, ports 319/320)
rkik --ptp 192.0.2.10

# Custom domain and ports (lab setup)
rkik --ptp --ptp-domain 24 --ptp-event-port 3319 --ptp-general-port 3320 127.0.0.1

# Verbose JSON output
rkik --ptp --verbose --format json --pretty ptp.lan

# Compare two PTP masters
rkik --ptp --compare 192.168.1.100 192.168.1.101
```

### PTP flags

| Flag | Description |
|------|-------------|
| `--ptp` | Enable PTP mode (mutually exclusive with `--nts`, `--sync`) |
| `--ptp-domain <N>` | Domain number (default `0`) |
| `--ptp-event-port <PORT>` | Event port (default `319`) |
| `--ptp-general-port <PORT>` | General port (default `320`) |
| `--ptp-hw-timestamp` | Request hardware timestamping |

PTP supports all existing output modes: compare, plugin, JSON, short, stats.

---

## Plugin Mode (Nagios / Centreon / Zabbix)

Since v1.2, rkik emits Nagios-compatible plugin output with perfdata.

```bash
rkik time.google.com --plugin --warning 50 --critical 200
```

Output:
```
RKIK OK - offset 4.006ms rtt 9.449ms from time.google.com (216.239.35.4) | offset_ms=4.006ms;50;200;0; rtt_ms=9.449ms;;;0;
```

### Flags

| Flag | Description |
|------|-------------|
| `--plugin` | Enable plugin mode (suppresses normal output) |
| `--warning <MS>` | Warning threshold in milliseconds |
| `--critical <MS>` | Critical threshold in milliseconds |

### Exit codes

| Code | State | Condition |
|------|-------|-----------|
| `0` | OK | `\|offset\| < warning` (or no thresholds) |
| `1` | WARNING | `warning ≤ \|offset\| < critical` |
| `2` | CRITICAL | `\|offset\| ≥ critical` |
| `3` | UNKNOWN | Request failed |

For NTS failures, security-critical errors (`aead_failure`, `missing_authenticator`, `unauthenticated_response`, `invalid_unique_id`, `invalid_origin_timestamp`) return exit code `2`; configuration/connection errors return `3`.

### Error output (UNKNOWN)

```
RKIK UNKNOWN - request failed | offset_ms=;50;200;0; rtt_ms=;;;0;
```

---

## Local Test Environment

A Docker Compose sandbox ships with the repo — three NTP daemons and a LinuxPTP grandmaster, no production infrastructure needed.

### Requirements

- Docker Engine 20.10+ (or Podman with Docker compatibility)
- `docker compose` plugin
- Internet access for containers to reach upstream clocks

### Services

| Service | Host port | Description |
|---------|-----------|-------------|
| `ntp_primary` | `3123/udp` | NTP following `time.google.com` |
| `ntp_secondary` | `4123/udp` | NTP following `time.cloudflare.com` |
| `ntp_pool` | `5123/udp` | General pool node |
| `ptp_master` | `3319/udp`, `3320/udp` | LinuxPTP grandmaster (domain 24) |

### Usage

```bash
# Start (builds linuxptp image on first run)
./scripts/test-env-up.sh

# NTP probes
rkik 127.0.0.1:3123
rkik --compare 127.0.0.1:3123 127.0.0.1:4123 127.0.0.1:5123 -c 5 -i 0.5

# PTP probe
rkik --ptp --ptp-domain 24 --ptp-event-port 3319 --ptp-general-port 3320 127.0.0.1

# Tear down
./scripts/test-env-down.sh
```

Override the compose binary: `COMPOSE_BIN="podman compose" ./scripts/test-env-up.sh`

### Troubleshooting

- **Port conflict**: edit host-side bindings in `dev/test-env/docker-compose.yml`.
- **macOS / Windows**: LinuxPTP requires UDP multicast — enable experimental multicast in Docker Desktop or use `network_mode: host`.
- **Corporate proxy**: configure Docker to route container traffic through the proxy.

---

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `dns:` error | Check DNS / try `-6` for IPv6 |
| `network: connection timed out` | Open UDP/123 outbound; try `--timeout 10` |
| `[ke_handshake_failed]` (NTS) | Check port 4460 is reachable; verify server NTS support |
| Inconsistent offsets | Verify local clock discipline; increase `--count` for averaging |
| Output garbled | Add `--nocolor` or set `NO_COLOR=1` |
