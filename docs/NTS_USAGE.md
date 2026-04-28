# NTS (Network Time Security) Usage Guide

## Overview

RKIK now supports **NTS (Network Time Security)**, a cryptographic security mechanism for NTP defined in RFC 8915. NTS provides authentication and integrity protection for time synchronization, ensuring that time data comes from a legitimate source and hasn't been tampered with in transit.

## What is NTS?

Network Time Security consists of two sub-protocols:

1. **NTS-KE (Key Establishment)**: Initial authentication and key exchange over TLS (typically port 4460)
2. **NTS-protected NTP**: Authenticated time synchronization using AEAD encryption and opaque cookies (port 123)

## Building with NTS Support

To enable NTS support, compile RKIK with the `nts` feature:

```bash
cargo build --release --features nts
```

Or install with NTS support:

```bash
cargo install rkik --features nts
```

## Basic Usage

### Simple NTS Query

Query an NTS-enabled server:

```bash
rkik time.cloudflare.com --nts
```

Output:
```
Server: time.cloudflare.com [NTS Authenticated]
IP: 162.159.200.1:123
UTC Time: Tue, 25 Nov 2025 15:51:35 +0000
Local Time: 2025-11-25 16:51:35
Clock Offset: 2.345 ms
Round Trip Delay: 15.678 ms
```

### Verbose Mode

Get detailed NTS authentication information:

```bash
rkik nts.ntp.se --nts -v
```

Output:
```
Server: nts.ntp.se [NTS Authenticated]
IP: 194.58.207.72:123
UTC Time: Tue, 25 Nov 2025 15:53:06 +0000
Local Time: 2025-11-25 16:53:06
Clock Offset: 1.234 ms
Round Trip Delay: 45.678 ms
Stratum: 0
Reference ID: [2a01:3f7:2:44::9]:4123
Timestamp: 1764085986
Authenticated: Yes (NTS)
```

### JSON Output

Get machine-readable JSON output with authentication status:

```bash
rkik nts.ntp.se --nts -j -p
```

Output:
```json
{
  "schema_version": 1,
  "run_ts": "2025-11-25T16:52:58.503603486+00:00",
  "results": [
    {
      "name": "nts.ntp.se",
      "ip": "194.58.207.72",
      "port": 123,
      "offset_ms": 1.234,
      "rtt_ms": 45.678,
      "utc": "2025-11-25T15:53:08.172904492+00:00",
      "local": "2025-11-25 16:53:08",
      "timestamp": null,
      "authenticated": true
    }
  ]
}
```

## Advanced Usage

### Compare Multiple NTS Servers

Compare time from multiple NTS-enabled servers:

```bash
rkik --compare nts.ntp.se time.cloudflare.com --nts -v
```

Output:
```
Comparing -  nts.ntp.se:123 and time.cloudflare.com:123
nts.ntp.se [NTS] [194.58.207.72 v4]: 1.234 ms
  Stratum: 0
  Reference ID: [2a01:3f7:2:44::9]:4123
  Round Trip Delay: 45.678 ms
  Authenticated: Yes (NTS)
time.cloudflare.com [NTS] [162.159.200.1 v4]: 2.345 ms
  Stratum: 0
  Reference ID: [2606:4700:f1::123]:123
  Round Trip Delay: 15.678 ms
  Authenticated: Yes (NTS)
Max drift: 1.111 ms (min: 1.234, max: 2.345, avg: 1.789)
```

### Continuous NTS Monitoring

Monitor NTS time with continuous queries:

```bash
rkik nts.ntp.se --nts -8 -i 5.0
```

This queries the NTS server every 5 seconds indefinitely.

### Custom NTS-KE Port

Specify a custom NTS-KE port (default is 4460):

```bash
rkik time.cloudflare.com --nts --nts-port 8443
```

## Public NTS Servers

Here are some public NTS-enabled NTP servers you can use:

| Server | Location | Provider | NTS-KE Port |
|--------|----------|----------|-------------|
| `time.cloudflare.com` | Global | Cloudflare | 4460 |
| `nts.ntp.se` | Sweden | Netnod | 4460 |
| `ntppool1.time.nl` | Netherlands | NLnet Labs | 4460 |
| `time.txryan.com` | USA | Ryan Sleevi | 4460 |
| `nts.ntp.org.au` | Australia | Australian NTP Pool | 4460 |

## Command-Line Options

### NTS-Specific Options

- `--nts`: Enable NTS (Network Time Security) authentication
- `--nts-port <PORT>`: Specify NTS-KE port number (default: 4460)

### Compatible with Existing Options

All existing RKIK options work with NTS:

- `-v, --verbose`: Show detailed output including authentication status
- `-j, --json`: JSON output with authenticated field
- `-p, --pretty`: Pretty-print JSON
- `-6, --ipv6`: Use IPv6 resolution
- `--timeout <SECONDS>`: Timeout for NTS-KE and NTP operations
- `-8, --infinite`: Continuous NTS queries
- `-i, --interval <SECONDS>`: Interval between queries
- `-c, --count <N>`: Number of queries to perform
- `-C, --compare <SERVERS>...`: Compare multiple NTS servers

## Understanding NTS Output

### Authentication Indicators

When NTS is successfully used, you'll see:

- **Text mode**: `[NTS Authenticated]` badge next to server name
- **Compare mode**: `[NTS]` badge in server list
- **Verbose mode**: `Authenticated: Yes (NTS)` in details
- **JSON mode**: `"authenticated": true` field

### Without NTS

When querying a regular NTP server (without `--nts`):

- **Text mode**: No authentication badge
- **Verbose mode**: `Authenticated: No`
- **JSON mode**: `"authenticated": false`

## NTS Error Kinds

Starting with rkik-nts v0.4.0, RKIK provides detailed NTS validation error reporting with machine-readable error kinds. These help diagnose NTS failures more precisely.

### Security-Critical Errors (Exit Code 2 in Plugin Mode)

| Error Kind | Description |
|------------|-------------|
| `aead_failure` | AEAD authentication failed on the NTP response |
| `missing_authenticator` | NTP response is missing the authenticator extension |
| `unauthenticated_response` | Server returned unauthenticated response after NTS-KE |
| `invalid_unique_id` | Invalid or mismatched Unique Identifier in response |
| `invalid_origin_timestamp` | Origin timestamp check failed (anti-replay protection) |

### Configuration/Connection Errors (Exit Code 3 in Plugin Mode)

| Error Kind | Description |
|------------|-------------|
| `ke_handshake_failed` | NTS-KE handshake failed (TLS or protocol error) |
| `certificate_invalid` | TLS certificate validation failed |
| `missing_cookies` | No cookies received from the NTS server |
| `malformed_extensions` | NTS extensions in response are malformed |
| `timeout` | Connection or operation timed out |
| `network` | Network-level error (connection refused, etc.) |

### JSON Output with Errors

In verbose mode, NTS errors appear in the JSON output:

```json
{
  "name": "time.example.com",
  "authenticated": false,
  "nts": {
    "authenticated": false,
    "error": {
      "kind": "aead_failure",
      "message": "NTS AEAD authentication failed"
    }
  }
}
```

### Text Output with Errors

When NTS validation fails, you'll see:

- **Text mode**: `[NTS Failed] (aead_failure)` badge
- **Compare mode**: `[NTS FAILED]` badge
- **Verbose mode**: Detailed error section with kind and message

## Troubleshooting

### Connection Errors

If you see `NTS-KE failed: Connection reset by peer` or `[ke_handshake_failed]`:

- Check that the server supports NTS
- Verify the NTS-KE port (usually 4460, not 123)
- Ensure your firewall allows outbound TLS connections

### Timeout Errors

If you see `NTS-KE failed: connection timed out` or `[timeout]`:

- Increase the timeout: `--timeout 15`
- Check network connectivity
- Some servers may be temporarily unavailable

### Certificate Errors

If you see `[certificate_invalid]`:

- The server's TLS certificate may be expired or self-signed
- Your system may be missing required root CA certificates
- Check if the server hostname matches the certificate

### AEAD Failures

If you see `[aead_failure]` or `[missing_authenticator]`:

- These indicate potential security issues (tampering or MITM)
- Try a different NTS server
- Check for network proxies that might interfere with NTS

### Unauthenticated Response

If you see `[unauthenticated_response]`:

- The server completed NTS-KE but sent an unauthenticated NTP response
- This could indicate server misconfiguration or an attack
- RKIK rejects such responses for security

### Mixed NTS/NTP Comparison

You cannot mix NTS and non-NTS queries in the same compare operation. Either use `--nts` for all servers or none.

## Security Considerations

### Benefits of NTS

- **Authentication**: Cryptographically verifies server identity
- **Integrity**: Prevents packet tampering
- **Replay protection**: Protects against replay attacks
- **TLS-based**: Leverages proven TLS security

### Limitations

- **No confidentiality**: Time values are not encrypted (only authenticated)
- **Initial latency**: NTS-KE adds TLS handshake overhead
- **Server support**: Not all NTP servers support NTS yet

## Implementation Details

RKIK's NTS support is built on:

- **rkik-nts v0.4.0**: High-level NTS client library with granular error reporting
- **ntpd-rs**: Battle-tested NTS implementation from Pendulum Project
- **rustls**: Modern TLS library for Rust

## Examples

### Basic Time Check with NTS

```bash
# Query Cloudflare's NTS server
rkik time.cloudflare.com --nts

# Same with verbose output
rkik time.cloudflare.com --nts -v
```

### Monitoring Script

```bash
# Monitor NTS time every 10 seconds, output JSON
rkik nts.ntp.se --nts -8 -i 10 -j > nts_monitoring.jsonl
```

### Compare NTS vs Regular NTP

```bash
# First query with NTS
rkik time.cloudflare.com --nts

# Then without NTS
rkik time.cloudflare.com
```

Note the `[NTS Authenticated]` badge difference.

## References

- [RFC 8915: Network Time Security for the Network Time Protocol](https://datatracker.ietf.org/doc/html/rfc8915)
- [rkik-nts GitHub Repository](https://github.com/aguacero7/rkik-nts)
- [Internet Society NTS Announcement](https://www.internetsociety.org/blog/2020/10/nts-rfc-published-new-standard-to-ensure-secure-time-on-the-internet/)
- [RKIK Documentation](https://aguacero7.github.io/rkik/)
