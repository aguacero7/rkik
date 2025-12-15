# RKIK - Changelog

## [Unreleased]

### Added
- **NTS (Network Time Security) support** - Full RFC 8915 implementation
  - `--nts` flag to enable NTS authentication
  - `--nts-port` to specify custom NTS-KE port (default: 4460)
  - NTS enabled by default in builds (feature flag `nts`)
  - Complete NTS-KE diagnostics in verbose mode:
    - Handshake duration measurement
    - Cookie count and sizes
    - AEAD algorithm negotiation details
    - NTP server address (may differ from NTS-KE server)
  - **TLS Certificate information** (requires rkik-nts v0.3.0+):
    - Subject and Issuer
    - Validity period (valid_from, valid_until)
    - Serial number
    - Subject Alternative Names (SANs)
    - Signature and public key algorithms
    - SHA-256 fingerprint
    - Self-signed certificate detection with warning
  - Full JSON export support for all NTS diagnostics
  - Compatible with all existing features (compare, plugin mode, etc.)

### Changed
- **Default features**: NTS is now included by default alongside `json` and `sync`
- **Dependency updates**:
  - `rkik-nts` upgraded from v0.2.0 to v0.3.0 (adds certificate support)

### Improved
- **Verbose mode enhancements**:
  - Comprehensive NTS-KE diagnostics section
  - TLS certificate details with color-coded output
  - Self-signed certificate warnings
  - Cookie size breakdown
- **JSON output**:
  - Full NTS-KE metadata in verbose JSON mode
  - Certificate information included in JSON exports
  - Backwards compatible with non-NTS queries

### Examples
```bash
# NTS query with full diagnostics
rkik --nts --verbose time.cloudflare.com

# NTS comparison between servers
rkik --nts --compare time.cloudflare.com nts.netnod.se

# JSON export with NTS diagnostics
rkik --nts --verbose --format json --pretty time.cloudflare.com

# Standard NTP still works as before
rkik pool.ntp.org
```

### Security
- NTS provides cryptographic authentication of NTP packets
- TLS certificate verification with chain of trust validation
- Detection and warning for self-signed certificates

---

## [1.2.1] - 2025-11-25

### Fixed
- **Plugin mode improvements**:
  - Fixed buffer flushing before `process::exit()` calls to ensure output is always visible
  - Removed code duplication in UNKNOWN output formatting
  - Added threshold validation: warning and critical must be non-negative, and warning must be less than critical
  - Fixed documentation mismatch: exit code conditions now correctly use `>=` for both code and documentation
  - Replaced `format!("{}")` with more efficient `to_string()` calls

### Changed
- **Exit code logic**: Threshold comparisons now use `>=` consistently (was `>` in code but `>=` in docs)

## [1.2.0] - 2025-10-27

### Added
- **Plugin / Monitoring mode**: new `--plugin` mode that emits a single Centreon/Nagios/Zabbix-compatible line and returns standard plugin exit codes.
- CLI flags: `--warning <MS>` and `--critical <MS>` (both require `--plugin`).

### Changed
- In plugin mode, the human-readable multi-line output is suppressed and only the plugin line is printed.

### Notes
- Thresholds are compared against the absolute clock offset in milliseconds. If the request fails, rkik returns `UNKNOWN` (exit code 3) and prints a plugin-style perfdata line with empty measurement fields.

## [1.1.0] - 2025-09-09

### Added
- **Sync dry-run mode**: `--dry-run` (and its short alias, if enabled) to validate the sync workflow without changing the system clock.

### Changed
- **`sync` feature enabled by default** for builds and packages.
  - To disable: `cargo build --no-default-features --features json`
- **CLI timing flags accept fractional seconds**:
  - `--interval` and `--timeout` now accept values like `0.1`, `0.01`, `0.5`.
  - Effective precision depends on the OS scheduler.

### Fixed
- **Public API cleanup for the sync module**: removed the duplicate import path.
  - Supported: `rkik::sync::{...}`
  - **Removed** (breaking): `rkik::sync::sync::{...}`

### CI / Quality
- **Clippy integrated into CI** with lints treated as errors (`-D warnings`) to enforce code quality.

---

### Migration Notes
- Replace imports from `rkik::sync::sync::*` with `rkik::sync::*`.
- Scripts can now use non-integer intervals and timeouts (e.g., `--interval 0.2`).

### Examples
```bash
# 10 requests at 200 ms intervals
rkik --server time.google.com --count 10 --interval 0.2

# Synchronization in dry-run mode (no clock change)
rkik --server time.google.com --sync --dry-run

# Build without the sync feature (minimal footprint)
cargo build --no-default-features --features json
```

## [1.0.0] – 2025-09-03

### Added
- **Port specification**: query any server at any port (IPv4 or IPv6).
  ```bash
  rkik time.google.com:123
  rkik [2606:4700:f1::123]:123
  ```
- **Sync feature**: optional `--sync` flag to apply time from a remote server to the local system  
  *(Unix only, requires root)*. By default, it won't be compiled in any package.
- **Continuous monitoring**: new flags `--count`, `--infinite`, `--interval` for repeated queries.
- **Library API**: rkik can now be embedded as a library. Output/formatting is cleanly separated from the core.
- **Short output mode**: `-S` / `--short` for minimalist output (text or JSON).

### Changed
- **Refactored codebase**: modular project structure for easier maintenance and library usage.
  ```text
  .
  ├── adapters
  │   ├── mod.rs
  │   ├── ntp_client.rs
  │   └── resolver.rs
  ├── bin
  │   └── rkik.rs
  ├── domain
  │   ├── mod.rs
  │   └── ntp.rs
  ├── error.rs
  ├── fmt
  │   ├── json.rs
  │   ├── mod.rs
  │   └── text.rs
  ├── lib.rs
  ├── services
  │   ├── compare.rs
  │   ├── mod.rs
  │   └── query.rs
  ├── stats.rs
  └── sync
      ├── mod.rs
      └── sync.rs
  ```
  See the [developer guide](https://github.com/aguacero7/rkik/blob/master/docs/developer_guide.md).

- **Error handling**: more detailed and consistent error messages via `RkikError` enum:
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

### Improved
- **JSON integration**:
  - Now powered by `serde_json` (thanks @lucy-dot-dot).
  - `--verbose` adds valuable metadata.
  - `--pretty` or `-p` for pretty-printed JSON.
  - Example:
    ```bash
    rkik -jp time.google.com
    ```
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

- **Convenience flags**:
  - `--json` or `-j`: alias for `--format json`.
  - `--no-color`: disable ANSI styling, always plain text if requested.

---


## [v0.6.1]
### Minor changes
- `--version` flag to display installed rkik's version
You can now display the installed version of rkik using -V or --version.

## [v0.6.0]
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
