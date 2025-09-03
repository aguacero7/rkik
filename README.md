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
-  Output formats: human-readable or JSON - both shortable (`-S`)
-  Verbose mode for advanced metadata
-  Accepts both FQDN and raw IPv4/6 addresses
-  Continuous diag with either infinite or static count
-  Port specification

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


### Compile sync feature
To enable rkik to apply queried time to your system, you must include sync feature to rkik's compilation
```bash
cargo build --release --features sync
```

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
| `rkik -C ntp1 ntp2 -c 2 -i 1 --nocolor`         | Compare 2 servers twice with an interval of 1s and display a nocolor output           |
| `rkik -S time.google.com --sync`         | Query a server and apply returned time to system (sync feature, UNIX only, requires root, is not installed by default)           |


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

Usage: rkik [OPTIONS] [TARGET]

Arguments:
  [TARGET]  Positional server name or IP (can include port specification) - Examples: [time.google.com, [2001:4860:4860::8888]:123, 192.168.1.23:123]

Options:
  -s, --server <SERVER>                 Query a single NTP server (optional)
  -C, --compare <COMPARE> <COMPARE>...  Compare multiple servers
  -v, --verbose                         Show detailed output
  -f, --format <FORMAT>                 Output format: text or json [default: text] [possible values: text, json, simple, json-short]
  -j, --json                            Alias for JSON output
  -S, --short                           Alias for simple / short text output
  -p, --pretty                          Pretty-print JSON
      --no-color                        Disable colored output
  -6, --ipv6                            Use IPv6 resolution only
      --timeout <TIMEOUT>               Timeout in seconds [default: 5]
  -8, --infinite                        Infinite count mode
  -i, --interval <INTERVAL>             Interval between queries in seconds (only with --infinite or --count) [default: 1]
  -c, --count <COUNT>                   Specific count of requests [default: 1]
  -h, --help                            Print help
  -V, --version                         Print version
```

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
We will chick whether this reference has an offset with us or not with `rkik 145.238.80.80`
```bash
Server: 145.238.80.80
IP: 145.238.80.80:123
UTC Time: Wed, 3 Sep 2025 13:09:57 +0000
Local Time: 2025-09-03 15:09:57
Clock Offset: 5001.531 ms
Round Trip Delay: 8.804 ms
```
It does, we now can assure the problem is external to our server, we may now connect on the system to change its reference with another server.