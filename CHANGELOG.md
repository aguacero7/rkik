# RKIK - Changelog 

## Unreleased
- possibility to specify a port to a query
We can now query a server at any port (either in IPv4 or v6). 
```bash
rkik time.google.com:123
rkik [2606:4700:f1::123]:123
```
- Optional --sync flag to actually apply the time from a distant server to our system ( Unix only, needs root )

- New --count --infinite --interval flags for continuous monitoring of the server

- API Integration
Everybody can now really use rkik as a library for their projects. The format / output part is now dissociated with the core of the app.
 
- Short Output mode (`-S`)
  A short output mode has been added displaying only the offset and the IP of the result, for JSON and text. It's the exact same output as the one displayed when using `--count` or `-8`


- More detailled errors 
Errors output is now more detailed, more precise and prettier. It follows this RkikError enuum
```rust
pub enum RkikError {
    /// DNS resolution failure.
    #[error("dns: {0}")]
    Dns(String),
    /// Network related error.
    #[error("network: {0}")]
    Network(String),
    /// Protocol violation.
    #[error("protocol: {0}")]
    Protocol(String),
    /// Underlying IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Other error cases.
    #[error("other: {0}")]
    Other(String),
}
```

- JSON integration improved
We now use serde_json for the format of the JSON output(thanks @lucy-dot-dot), which has changed, making the use of the verbose flag on it actually very useful, you can now use
```bash
rkik -jp time.google.com
```
To display a pretty json output. 
```json
{
  "schema_version": 1,
  "run_ts": "2025-08-26T15:46:54.558275110+00:00",
  "results": [
    {
      "name": "time.google.com",
      "ip": "216.239.35.8",
      "offset_ms": 1.4152181101962924,
      "rtt_ms": 12.369429459795356,
      "utc": "2025-08-26T15:46:54.559491539+00:00",
      "local": "2025-08-26 17:46:54"
    }
  ]
}
```

- `--json` or `-j` flag
 We have added an alias to the `--format json` flag to make it simpler to ask for a json output. You can combine it with `--pretty` or `-p` for a prettier output.

- no color format integration
We've added the `--nocolor` arg for the output to not be stylized, otherwise, il will always be if your terminal can handle it.

- Short output format
You can now use `--format simple` or `-S` / `--short` to display a minimalist output with only the time of the requested server and its IP address.

## Latest version v0.6.1
### Minor changes
- `--version` flag to display installed rkik's version
You can now display the installed version of rkik using -V or --version.

## Version v0.6.0
### Async Comparison Mode

The --compare flag now supports comparing 2 or more NTP servers in parallel, powered by tokio. This results in significantly improved performance and better scalability for auditing drift across multiple time sources.

```bash
rkik --compare time.google.com time.cloudflare.com 0.pool.ntp.org
```
- Async Foundation for Future Use Cases
The asynchronous implementation is now a clean foundation for future monitoring, scheduling, or background tasks using tokio.

- Dynamic Server Count in --compare
No longer limited to 2 servers — the comparison now accepts up to 10 servers and returns a comprehensive view of offsets and drift.

- Improved CLI Argument Parsing
The --compare flag uses num_args = 2..10, enabling natural and flexible command-line usage.

### Improvements
- Full refactor of compare_servers into async logic with join_all.
- Better error reporting during comparison phase (resolvable vs. unreachable servers).
- Refactored architecture to cleanly separate sync and async code paths.
- CLI gracefully switches between sync and async depending on operation mode.


### CLI Ergonomics
Short flags added for faster interaction:
`-C = --compare`
`-v = --verbose`
`-6 = --ipv6`
`-s = --server`
