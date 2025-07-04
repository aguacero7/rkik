# User Guide

This guide helps you install and use **RKIK** to query and compare NTP servers.

## Installation

### Prebuilt Binaries

Download the latest archive from the [releases page](https://github.com/aguacero7/rkik/releases/latest) and extract it:

For linux :
```bash
tar xvfz rkik-linux-x86_64.tar.gz
sudo mv rkik /usr/local/bin
```

You can also use the .exe on windows.

### Install Packages
There is among the release artifacts some pre-built packages for linux distributions which could in the future be part 
in an official repository.

- For Red-Hat based distros (CentOS, Fedora, ...) : `rkik-X.Y.Z-R.x86_64.rpm `

Which can be installed by using
```bash
rpm -U rkik-X.Y.Z-R.x86_64.rpm
# or 
dnf install rkik-X.Y.Z-R.x86_64.rpm
#or 
yum install rkik-X.Y.Z-R.x86_64.rpm
```

- For Debian based distros (Debian, Ubuntu, Kali, ...) : `rkik_X.Y.Z-R_amd64.deb `

You can simply install it with 
```bash
apt install rkik_X.Y.Z-R_amd64.deb
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

