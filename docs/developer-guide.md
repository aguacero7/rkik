# RKIK — Developer Guide

## Table of Contents

- [Architecture](#architecture)
- [Code layout](#code-layout)
- [Library API](#library-api)
- [Building and testing](#building-and-testing)
- [Packaging and releases](#packaging-and-releases)
- [Contributing](#contributing)

---

## Architecture

rkik is a Rust CLI with a reusable library. The split is intentional and strict: no `clap`, `console`, or `process::exit` in library code.

The main execution path for an NTP probe:

```
CLI (legacy.rs / rkik.rs)
  └─ services::query::query_one()
       ├─ adapters::resolver::resolve_ip()   (DNS)
       └─ adapters::ntp_client              (rsntp → ProbeResult)
            └─ [--nts] adapters::nts_client (rkik-nts → NtsTimeResult)
```

Compare mode runs all queries concurrently via `futures::join_all`.

**Feature flags:**

| Feature | Default | Description |
|---------|---------|-------------|
| `json` | yes | `serde::Serialize` on domain types; JSON formatters |
| `nts` | yes | NTS support via `rkik-nts` |
| `ptp` | yes (Linux) | PTP/IEEE 1588 mode via `statime` |
| `sync` | yes | System clock sync (Unix, root) |
| `network-tests` | no | Integration tests hitting real servers |

---

## Code layout

```
src/
  bin/rkik.rs          # modern subcommand CLI (clap)
  bin/rkik/
    legacy.rs          # legacy one-shot CLI (backwards compat)
    config_store.rs    # TOML-backed config + presets
  lib.rs               # public API re-exports
  adapters/
    resolver.rs        # DNS resolution
    ntp_client.rs      # rsntp wrapper → ProbeResult
    nts_client.rs      # rkik-nts wrapper (feature nts)
    ptp_client.rs      # statime wrapper (feature ptp, linux)
  domain/
    ntp.rs             # Target, ProbeResult
    ptp.rs             # PtpProbeResult, clock metadata (feature ptp)
  services/
    query.rs           # query_one()
    compare.rs         # compare_many()
    ptp_query.rs       # PTP probe entry point (feature ptp)
  fmt/
    text.rs            # terminal rendering
    json.rs            # JSON serialization
    ptp_text.rs        # PTP text renderer (feature ptp)
    ptp_json.rs        # PTP JSON renderer (feature ptp)
  stats.rs             # Stats, PtpStats, compute_stats()
  sync/                # clock sync (feature sync)
  error.rs             # RkikError
tests/
  integration.rs       # basic lib integration tests
  nts_test.rs          # NTS rendering and validation tests
  ptp_tests.rs         # PTP renderer unit tests
  cli_test.rs          # CLI smoke tests (assert_cmd)
```

---

## Library API

Add to `Cargo.toml`:

```toml
[dependencies]
rkik = { version = "2", default-features = false, features = ["json"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Types

```rust
pub struct Target {
    pub name: String,
    pub ip: std::net::IpAddr,
    pub port: u16,
}

pub struct ProbeResult {
    pub target: Target,
    pub offset_ms: f64,      // signed: positive = local clock ahead
    pub rtt_ms: f64,
    pub stratum: u8,
    pub ref_id: String,
    pub utc: chrono::DateTime<chrono::Utc>,
    pub local: chrono::DateTime<chrono::Local>,
    pub timestamp: i64,
    pub authenticated: bool,
}
```

With the `json` feature, these derive `serde::Serialize`.

### Functions

```rust
pub async fn query_one(
    target: &str,
    ipv6_only: bool,
    timeout: std::time::Duration,
    use_nts: bool,
    nts_ke_port: u16,
) -> Result<ProbeResult, RkikError>;

pub async fn compare_many(
    targets: &[String],
    ipv6_only: bool,
    timeout: std::time::Duration,
    use_nts: bool,
    nts_ke_port: u16,
) -> Result<Vec<ProbeResult>, RkikError>;
```

### Example

```rust
use rkik::{query_one, compare_many};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single probe
    let r = query_one("time.google.com", false, Duration::from_secs(3), false, 4460).await?;
    println!("{}: offset={:.3}ms rtt={:.3}ms", r.target.name, r.offset_ms, r.rtt_ms);

    // Parallel compare
    let targets = vec!["pool.ntp.org".into(), "time.cloudflare.com".into()];
    let results = compare_many(&targets, false, Duration::from_secs(3), false, 4460).await?;
    for p in &results {
        println!("{}: {:.3}ms", p.target.name, p.offset_ms);
    }
    Ok(())
}
```

---

## Building and testing

```bash
# Standard build
cargo build --release

# Run tests (no network)
cargo test

# Run with real NTP servers
cargo test --features network-tests

# Minimal build (no NTS, no PTP, no sync)
cargo build --no-default-features --features json

# Lint
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

The Docker lab (`./scripts/test-env-up.sh`) spins up three NTP daemons and a LinuxPTP grandmaster for local end-to-end testing. See [user-guide.md#local-test-environment](user-guide.md#local-test-environment).

---

## Packaging and releases

Packaging metadata lives in `Cargo.toml` under `[package.metadata.deb]` and `[package.metadata.generate-rpm]`. CI builds `.deb`, `.rpm`, and `tar.gz` artifacts and publishes them to GitHub Releases alongside a `cargo publish` for crates.io.

**Distribution packages:**

| Platform | Package | Link |
|----------|---------|------|
| Arch Linux | `rkik-git` (AUR) | https://aur.archlinux.org/packages/rkik-git |
| Nix / NixOS | `rkik` (nixpkgs) | https://search.nixos.org/packages?query=rkik |
| All Linux | `.deb` / `.rpm` / tar.gz | GitHub Releases |

---

## Contributing

1. Fork the repo and create a feature branch from `master`.
2. Keep the library/CLI split: no `clap`, `console`, or `process::exit` in `src/lib.rs` or anything it re-exports.
3. Run `cargo fmt` and `cargo clippy -- -D warnings` before pushing.
4. Add or update tests for any behaviour change.
5. Open a PR referencing the relevant issue. Commit messages follow the `type: description` convention (`feat:`, `fix:`, `docs:`, `chore:`).

If you're looking for a starting point, check the [good first issue](https://github.com/aguacero7/rkik/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) label.
