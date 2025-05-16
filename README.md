# RKIK - Rusty Klock Inspection Kit

RKIK is a command-line tool for querying NTP servers and comparing clock offsets in Rust. It provides functionalities for querying single servers, comparing two servers, and outputting data in both human-readable and JSON formats.

## Features
- Query a single NTP server and display clock offset, round-trip delay, and stratum.
- Compare the clock offsets between two NTP servers.
- Output in human-readable or JSON format.
- Verbose mode for additional information (stratum).

## Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd rkik
```
# Build the project
```
cargo build --release
```
# Install the binary globally
```
sudo cp target/release/rkik /usr/local/bin
```
# Verify the installation
```
rkik --help
```
#  Usage Examples and Parameters

| Command                           | Description                             | Example                                  |
|---------------------------------- |--------------------------------------- |---------------------------------------- |
| `--server <server>`               | Query a single NTP server               | `rkik --server pool.ntp.org`             |
| `--server <server> --verbose`     | Query a server with verbose output     | `rkik --server time.google.com --verbose` |
| `--server <server> --format json` | Query a server and get JSON output     | `rkik --server time.cloudflare.com --format json` |
| `--compare <server1> <server2>`   | Compare two NTP servers                | `rkik --compare pool.ntp.org time.google.com` |
| `--compare <server1> <server2> --format json` | Compare two servers in JSON format  | `rkik --compare pool.ntp.org time.cloudflare.com --format json` |

