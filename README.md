# RKIK - Rusty Klock Inspection Kit
![Build & Tests](https://github.com/aguacero7/rkik/actions/workflows/ci-test-n-build.yml/badge.svg)
<br>
Most systems rely on a daemon (like chronyd or ntpd) to synchronize time. But what if you just want to **inspect** the current offset between your system clock and one or more NTP servers — without root, without sync, and without installing anything heavy?

**RKIK** is a Rust-based CLI tool designed for **stateless and passive NTP inspection**, just as `dig` or `ping` are for DNS and ICMP.

**Link to  [Documentation page](https://aguacero7.github.io/rkik/)**

---

## Features

- Query any NTP server (IPv4 or IPv6)
-  Compare offsets between X servers
-  Output formats: human-readable or JSON
-  Verbose mode for advanced metadata
-  Accepts both FQDN and raw IP addresses
-  Argument parsing via `clap` with fallback positional support

---

## Installation

### Linux
```bash
# Download rkik-linux-x86_64.tar.gz on https://github.com/aguacero7/rkik/releases/latest
tar xvfz rkik-linux-x86_64.tar.gz 
cd rkik-linux-x86_64/
sudo mv ./rkik /usr/local/bin
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
apt install <rkik-<X.Y.Z-R>.x86_64.rpm>
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

---

## Usage Examples

| Command                                          | Description                                |
|--------------------------------------------------|--------------------------------------------|
| `rkik pool.ntp.org`                              | Query an NTP server (positional)           |
| `rkik pool.ntp.org -6`                              | Query an NTP server using IPv6 (positional)           |
| `rkik --server pool.ntp.org`                     | Same as above, explicit flag               |
| `rkik --server time.google.com --verbose`        | Verbose query output                       |
| `rkik --server time.cloudflare.com --format json`| JSON output for a single server            |
| `rkik --compare pool.ntp.org time.google.com`    | Compare two servers                        |
| `rkik -C ntp1 ntp2 ntp3 ntp4 --format json`         | Compare servers with JSON output           |

---

## Output Examples

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

## Documentation

See the [docs](docs/README.md) directory for the full user and developer guides.
