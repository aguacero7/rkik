# User Guide

This guide helps you install and use **RKIK** to query and compare NTP servers.

## Installation

### Prebuilt Binaries

Download the latest archive from the [releases page](https://github.com/aguacero7/rkik/releases/latest) and extract it:

```bash
tar xvfz rkik-linux-x86_64.tar.gz
sudo mv rkik /usr/local/bin
```

### From Source

```bash
git clone https://github.com/aguacero7/rkik.git
cd rkik
cargo build --release
sudo cp target/release/rkik /usr/local/bin
```

## Configuration

`rkik` has no persistent configuration. Use command line flags to control behaviour.

## Usage

Query an NTP server:

```bash
rkik pool.ntp.org
```

Compare two servers:

```bash
rkik --compare time.google.com time.cloudflare.com
```

Add `--verbose` to show stratum and reference ID, or `--format json` for JSON output.

## Troubleshooting

If you see network errors, ensure your firewall allows NTP traffic (UDP port 123) and that the server hostname resolves.

## FAQ

**Q:** Does RKIK support IPv6?

**A:** Yes. Use the `--ipv6` flag when querying servers that have AAAA records.

