# Developer Guide

This guide describes the project architecture and contribution workflow, we are highly open to any contribution !!

## Project Architecture

RKIK is a Rust CLI with a **reusable library**.

- **CLI entrypoint**: `src/bin/rkik.rs` (clap parsing, text/JSON rendering, color control, continuous modes).
- **Library**: `src/lib.rs` re-exports the public API and organizes modules:
  - `adapters/`: DNS resolver and NTP client (`rsntp`).
  - `domain/ntp.rs`: `Target`, `ProbeResult` (derive `Serialize` under the `json` feature).
  - `services/query.rs`: `pub async fn query_one(...) -> Result<ProbeResult, RkikError>`.
  - `services/compare.rs`: `pub async fn compare_many(...) -> Result<Vec<ProbeResult>, RkikError>` (parallel via `futures::join_all`).
  - `fmt/json.rs`, `fmt/text.rs`: serialization and terminal rendering (not used by the library’s public API).
  - `stats/`: min/max/avg rollups.
  - `sync/` (feature `sync`): system time application (Unix only).
- **Runtime**: multi-threaded `tokio`.

### Principles
- **No CLI deps in the public API** (no `clap`, `console`, or `process::exit` in library functions).
- **Errors**: `thiserror` + `Result<_, RkikError>`.
- **Tracing**: `tracing::instrument` on I/O boundaries.

## Code Layout

```text
src/
  bin/rkik.rs        # CLI (flags, colors, formats, loops)
  lib.rs             # API re-exports
  adapters/          # DNS + rsntp
  domain/            # domain types (ProbeResult, Target, ...)
  services/          # query_one / compare_many
  fmt/               # text/json output (CLI-side)
  stats/             # Stats
  sync/              # 'sync' feature (Unix)
```

## Async mode
- `--compare` is **asynchronous** and runs all queries in parallel via `join_all`.
- One-shot queries are exposed as **async** via `query_one`.
- Timeout is passed as a `Duration` parameter.

## Environment
```bash
rustup install stable
rustup default stable
cargo build --release
```

## Tests
- **Library** (`tests/v1_lib.rs`) — `tokio::test` (+ `--features network-tests` to talk to real NTP servers).
- **CLI** (`tests/v1_cli.rs`) — `assert_cmd`/`predicates`.
- Lint/format:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features -D warnings
  ```

## CI/CD
- Multi-OS test/build workflow (Linux/macOS/Windows): `cargo test`, `fmt`, `clippy`.
- Release: `.deb`, `.rpm`, binaries; `cargo publish` (source-only).

## Contribution
1. Fork + feature branch.
2. Preserve the library/CLI split and the async design.
3. Keep `cargo fmt` and `clippy` clean.
4. Submit a PR referencing the relevant issue.
