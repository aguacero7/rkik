# User Guide

This guide helps you install and use **RKIK** to query and compare NTP servers.

## Installation

### Prebuilt Binaries

Download the latest archive from the [releases page](https://github.com/aguacero7/rkik/releases/latest) and extract it:

For Linux:

```bash
tar xvfz rkik-linux-x86_64.tar.gz
sudo mv rkik /usr/local/bin
```

For Windows:

Use the `.exe` from the archive and place it in your `PATH`.

### Install Packages

Prebuilt Linux packages are provided and may later be added to official repositories.

- **RPM (RedHat, Fedora, Alma, Rocky)**

```bash
rpm -U rkik-X.Y.Z-R.x86_64.rpm
# or
dnf install rkik-X.Y.Z-R.x86_64.rpm
# or
yum install rkik-X.Y.Z-R.x86_64.rpm
```

- **DEB (Debian, Ubuntu, Kali, etc.)**

```bash
sudo apt install ./rkik_X.Y.Z-R_amd64.deb
```

### From Source

```bash
git clone https://github.com/aguacero7/rkik.git
cd rkik
cargo build --release
sudo cp target/release/rkik /usr/local/bin
```

## Configuration

`rkik` does not require any configuration files.  
All behavior is controlled via command-line arguments.

## Usage

### Query a single server (sync)

```bash
rkik pool.ntp.org
```

Or using the flag:

```bash
rkik --server time.cloudflare.com
```

This runs a single request, using the synchronous NTP client. Use `--verbose` to get more detail.

### Compare multiple servers (async)

```bash
rkik --compare time.google.com time.cloudflare.com 0.de.pool.ntp.org
```

RKIK will resolve and query all servers **in parallel** and output their offset relative to the local system clock.

You can pass 2 or more servers. Example output:

```bash
Comparing (async): 3 servers
time.google.com [142.251.129.16 v4]: 1.034 ms
time.cloudflare.com [162.159.200.1 v4]: 0.867 ms
0.de.pool.ntp.org [85.214.110.245 v4]: 2.348 ms
Max drift: 1.481 ms (min: 0.867, max: 2.348, avg: 1.416)
```

You can also format as JSON:

```bash
rkik --compare time1 time2 time3 --format json
```

### Additional Flags

- `--verbose` — shows NTP stratum and reference ID.
- `--ipv6` — forces AAAA resolution (IPv6 only).
- `--format json` — machine-readable output.

## Use Cases

- Spot-check your system clock drift:
  ```bash
  rkik --compare time.google.com time.cloudflare.com
  ```

- Monitor NTP consistency across providers or ISPs.
- Validate reachability and IPv6 NTP responses:
  ```bash
  rkik --compare --ipv6 time.cloudflare.com 2a00:fb01::1
  ```

- Scripted JSON output for integration with logging or SIEM systems.

## Troubleshooting

- **"Failed to resolve hostname"**  
  → Check DNS resolution or try adding `--ipv6` if needed.

- **"Timeout or no response"**  
  → Ensure your firewall allows **UDP port 123**.

- **"Offset too large"**  
  → Your system clock may be significantly out of sync.

## FAQ

**Q:** Does RKIK support IPv6?  
**A:** Yes. Use `--ipv6` to force AAAA queries.

**Q:** Can I compare more than 2 servers?  
**A:** Yes. `--compare` supports 2 or more servers. All are queried in parallel.

**Q:** Can RKIK adjust my system clock?  
**A:** No. RKIK only queries servers. It never changes your time.

## Licensing

RKIK is open-source, licensed under the MIT license.