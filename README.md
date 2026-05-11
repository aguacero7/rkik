# rkik - Rusty Klock Inspection Kit

[![Build & Tests](https://github.com/aguacero7/rkik/actions/workflows/ci-test-n-build.yml/badge.svg)](https://github.com/aguacero7/rkik/actions/workflows/ci-test-n-build.yml)
[![crates.io](https://img.shields.io/crates/v/rkik.svg)](https://crates.io/crates/rkik)
[![Packaging status](https://repology.org/badge/tiny-repos/rkik.svg)](https://repology.org/project/rkik/versions)

> Think `dig`, but for time sources. No daemon, no root, no config to touch.

rkik queries NTP and NTS servers and tells you what they say: offset, RTT, stratum, authentication status. One shot, stateless, done.

---

## Why rkik?

- **Passive by design.** It never touches your system clock unless you explicitly ask (`--sync`). Run it anywhere, as anyone.
- **NTS that actually works.** rkik-nts 1.0.0 is the first rust implementation of RFC 8915 verified against real public servers (`time.cloudflare.com`, `nts.ntp.se`). Older tools either skip authentication or implement it wrong.
- **Two protocols, one tool.** NTP over IPv4/IPv6 and NTS with full TLS diagnostics — compare mode, JSON output, and Nagios/Centreon plugin support included.
- The only **NTS-aware monitoring** plugin for Nagios, Centreon and Zabbix
---

## Demo

[![asciicast](https://asciinema.org/a/LBiUOMMoimo3DWHh.svg)](https://asciinema.org/a/LBiUOMMoimo3DWHh)

```
$ rkik --nts time.cloudflare.com -v

Server: time.cloudflare.com [NTS Authenticated]
IP: 162.159.200.1:123
UTC Time: Tue, 28 Apr 2026 14:32:10 +0000
Local Time: 2026-04-28 16:32:10
Clock Offset: 1.243 ms
Round Trip Delay: 14.871 ms

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

---

## Installation

```bash
cargo install rkik
```

**Arch Linux (AUR):**
```bash
yay -S rkik-git
```

**Nix / NixOS:**
```bash
nix shell nixpkgs#rkik
```

**DEB / RPM / tar.gz:** grab the latest from [GitHub Releases](https://github.com/aguacero7/rkik/releases/latest).

```bash
# Debian / Ubuntu
sudo apt install ./rkik_2.2.0-1_amd64.deb

# Fedora / RHEL / Alma / Rocky
sudo dnf install rkik-2.2.0-1.x86_64.rpm
```

[![Packaging status](https://repology.org/badge/vertical-allrepos/rkik.svg)](https://repology.org/project/rkik/versions)

---

## Quick start

```bash
# Basic NTP probe
rkik pool.ntp.org

# NTS — cryptographically authenticated
rkik --nts time.cloudflare.com

# Compare a few servers side by side
rkik --compare pool.ntp.org time.google.com time.cloudflare.com

# Plugin output for Nagios / Centreon / Zabbix
rkik pool.ntp.org --plugin --warning 50 --critical 200

# JSON, prettified
rkik pool.ntp.org --json --pretty
```

---

## NTP

```bash
rkik time.google.com              # basic probe
rkik time.google.com -v           # adds stratum, ref ID
rkik time.google.com --count 10 --interval 2   # repeat 10 times
rkik -6 pool.ntp.org              # IPv6-only resolution
```

Output:
```
Server: time.google.com
IP: 216.239.35.4:123
UTC Time: Tue, 28 Apr 2026 14:32:08 +0000
Local Time: 2026-04-28 16:32:08
Clock Offset: 0.312 ms
Round Trip Delay: 9.449 ms
```

---

## NTS — Network Time Security ★

NTS (RFC 8915) is authenticated NTP — every packet is verified with AEAD encryption, so you know the response hasn't been tampered with.

```bash
rkik --nts time.cloudflare.com          # authenticated probe
rkik --nts time.cloudflare.com -v       # with TLS/KE diagnostics
rkik --nts --compare nts.ntp.se time.cloudflare.com
```

When something goes wrong, rkik tells you what kind of failure it is:

| Kind | Meaning |
|------|---------|
| `aead_failure` | AEAD verification failed — possible tampering |
| `missing_authenticator` | Authenticator extension absent in response |
| `unauthenticated_response` | Server completed NTS-KE but sent plain NTP |
| `ke_handshake_failed` | TLS or NTS-KE handshake failed |
| `timeout` | Connection timed out |

Security-critical failures (`aead_failure`, `missing_authenticator`, `unauthenticated_response`, `invalid_unique_id`, `invalid_origin_timestamp`) exit with code 2 in plugin mode.

See [docs/user-guide.md#nts](docs/user-guide.md#nts--network-time-security) for the full error reference.

---

## Plugin mode

```bash
rkik time.google.com --plugin --warning 50 --critical 200
# → RKIK OK - offset 4.006ms rtt 9.449ms from time.google.com (216.239.35.4) | offset_ms=4.006ms;50;200;0; rtt_ms=9.449ms;;;0;
```

Exit codes: `0` OK · `1` WARNING · `2` CRITICAL · `3` UNKNOWN (request failed or security error)

---

## Debugging a time issue

Your monitoring fires a time alert on `ntp.server.local`. You don't know yet whether the problem is that server or its upstream.

```bash
$ rkik ntp.server.local -v
Clock Offset: 5000.145 ms
Reference ID: 145.238.80.80
```

Five seconds of drift, and you can see the upstream. Check it directly:

```bash
$ rkik 145.238.80.80
Clock Offset: 5001.531 ms
```

Same offset upstream. The problem isn't your server — it's the reference. Time to point it somewhere else.

---

## For developers

rkik is split into a library and a CLI. The library has no `clap`, no `process::exit`, no terminal code — just async functions you can call directly.

```toml
[dependencies]
rkik = { version = "2", default-features = false, features = ["json"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use rkik::query_one;
use std::time::Duration;

let r = query_one("time.google.com", false, Duration::from_secs(3), false, 4460).await?;
println!("{}: offset={:.3}ms", r.target.name, r.offset_ms);
```

Full API reference, architecture overview, and contribution guide: [docs/developer-guide.md](docs/developer-guide.md)

To contribute, see [CONTRIBUTING](docs/developer-guide.md#contributing) — issues labelled [`good first issue`](https://github.com/aguacero7/rkik/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) are a good starting point.
