# Developer Guide

This guide describes the project architecture and how to contribute.

## Project Architecture

RKIK is a Rust command-line application to query and compare NTP servers.

- **Main Logic**: Centralized in `src/lib.rs`, with async logic integrated for multi-server comparison.
- **CLI Parsing**: Done with [`clap`](https://docs.rs/clap), supporting both positional and flagged arguments.
- **NTP Client**:
  - [`rsntp`](https://crates.io/crates/rsntp) provides both sync and async APIs.
  - `SntpClient` is used for single-server queries (sync).
  - `AsyncSntpClient` is used for `--compare` (async).
- **Async Runtime**: Powered by [`tokio`](https://crates.io/crates/tokio).
- **Terminal Output**: Colored output via the `console` crate.

## Code Structure

```
src/
  main.rs        - CLI entry point, dispatches sync or async paths
  lib.rs         - Core logic, including IP resolution, sync query and async comparison
  async_compare.rs - Async implementation of --compare (2+ servers)
```

Unit and CLI tests are in the `tests/` directory.

## Async Mode

The comparison mode (`--compare s1 s2 [s3...]`) is always **asynchronous**, and runs all queries in parallel using `futures::join_all`.

Simple queries (`--server`, positional) remain **synchronous** for simplicity and performance.

## Environment Setup

Ensure you have the latest stable Rust toolchain:

```bash
rustup install stable
rustup default stable
```

Build the project:

```bash
cargo build --release
```

Run all tests (unit + integration):

```bash
cargo test
```

If you want to simulate real NTP requests in tests, enable optional network tests (not enabled by default):

```bash
cargo test --features integration-tests
```

## Contribution Guidelines

1. Fork the repository and create your feature branch.
2. Follow the async design if you contribute to `--compare`.
3. Write tests for your changes (unit or CLI).
4. Run `cargo fmt -- --check` and ensure no changes are needed.
5. Submit a pull request and reference the relevant issue.

## CI/CD

The workflow `ci-test-n-build.yml` runs on each push. It covers:
- `cargo test`, `cargo fmt`, and `cargo clippy`
- Multiple targets: Linux, macOS, Windows
- Checks with stable toolchain

The `release.yml` workflow handles:
- Package builds (`.deb`, `.rpm`, static binaries)
- Cross-compilation for Linux and Windows
- Release asset upload on GitHub

Crates published via `cargo publish` are **source-only**, controlled via `Cargo.toml: package.include`.

Packaging metadata (description, license, deb/rpm configs) is also maintained in `Cargo.toml`.

## Notes

- Prefer `join_all` over `spawn` to keep async logic deterministic.
- The async comparison logic is designed to scale up to ~10 servers without performance drop.