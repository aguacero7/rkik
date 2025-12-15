# User Guide

This guide explains how to install and use **RKIK** to probe and compare NTP servers.

## Installation

### AUR (Arch Linux and derivatives)
A development package **`rkik-git`** is available:
```bash
git clone https://aur.archlinux.org/rkik-git.git
cd rkik-git
makepkg -si
```
With a helper (example `yay`):
```bash
yay -S rkik-git
```

### Nix / NixOS
RKIK is packaged in nixpkgs as **`rkik`** (availability/version depends on the channel):
```bash
# temporary shell
nix shell nixpkgs#rkik
# install into user profile
nix-env -iA nixpkgs.rkik
```

### Binaries / Linux packages
Archives and `.deb`/`.rpm` are published in GitHub Releases.
```bash
# Debian/Ubuntu
sudo apt install ./rkik_X.Y.Z-R_amd64.deb

# Fedora/RHEL/Alma/Rocky
sudo dnf install rkik-X.Y.Z-R.x86_64.rpm
```

### From source
```bash
git clone https://github.com/aguacero7/rkik.git
cd rkik
cargo build --release
sudo install -m 0755 target/release/rkik /usr/local/bin/rkik
```

## Usage

### Probe a single server
```bash
rkik pool.ntp.org
rkik --server time.cloudflare.com
```

### Compare multiple servers (asynchronous)
```bash
rkik --compare time.google.com,time.cloudflare.com,0.de.pool.ntp.org
rkik --compare time1 time2 time3 --format json
```

### IPv6-only
```bash
rkik -6 --server 2.pool.ntp.org -j
```

### Output formats
- `--format text` (default) — human-readable.
- `--format json` — detailed, stable.
- `--format simple` — minimal text (timestamp, name/port).
- `--format json-short` — compact (`{"utc": "...", "name": "...", "port": 123}`).
  Aliases: `-j/--json`, `-S/--short`.

### Continuous mode
```bash
# two measurements, 1s apart
rkik --server time.cloudflare.com --count 2 --interval 1 --short

# infinite loop (Ctrl-C to stop)
rkik --server time.google.com --infinite --format json
```
For ingestion into a SIEM/log pipeline, prefer `--format json` and collect **one JSON object per line**.

### Targets `host[:port]`
```bash
rkik time.google.com:123
rkik [2606:4700:f1::123]:123
```

### Colors
Disable all coloring:
```bash
rkik --no-color
# or environment variable
NO_COLOR=1 rkik ...
```

## Precision Time Protocol (PTP)

RKIK can inspect IEEE 1588 masters using the new `--ptp` mode (feature `ptp`, enabled by default).

```bash
# Basic probe against domain 0 (default ports 319/320)
rkik --ptp 192.0.2.1

# Custom domain and ports (useful for lab setups)
rkik --ptp --ptp-domain 24 --ptp-event-port 3319 --ptp-general-port 3320 127.0.0.1

# Verbose JSON output with diagnostics
rkik --ptp --verbose --format json --pretty ptp.lab.local
```

Available flags:
- `--ptp` — enable PTP mode (mutually exclusive with `--nts`/`--sync`)
- `--ptp-domain` — domain number (default `0`)
- `--ptp-event-port` / `--ptp-general-port` — override UDP ports (defaults `319`/`320`)
- `--ptp-hw-timestamp` — request hardware timestamping (reported in diagnostics)

PTP shares the existing features (compare mode, plugin mode, JSON/short outputs). Offsets are displayed in nanoseconds and detailed master metadata appears in verbose mode.

## Local Test Environment

A ready-made Docker Compose lab with three NTP servers and a LinuxPTP grandmaster is included.

```bash
./scripts/test-env-up.sh           # start containers
rkik 127.0.0.1:3123                # query lab NTP primary
rkik --ptp --ptp-event-port 3319 127.0.0.1   # query lab PTP master
./scripts/test-env-down.sh         # stop containers
```

See [docs/TEST_ENV.md](TEST_ENV.md) for topology details, port mapping, and troubleshooting tips.

## Troubleshooting
- **Resolution failed**: check DNS / try `-6` if needed.
- **Timeout**: open UDP/123.
- **Inconsistent offsets**: verify local clock and repeatability.

## FAQ
- **IPv6 supported?** Yes, `-6`.
- **More than 2 servers?** Yes, `--compare` accepts N≥2, all in parallel.
- **Adjust the system clock?** Yes, `--sync` requirements: (Unix, root)
