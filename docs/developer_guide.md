# Developer Guide

This guide describes the project architecture and how to contribute.

## Project Architecture

RKIK is a Rust command-line application. The main logic lives in `src/lib.rs`, while `src/main.rs` contains the entry point.

- **Argument Parsing**: Implemented with [`clap`](https://docs.rs/clap).
- **NTP Client**: Provided by the [`rsntp`](https://crates.io/crates/rsntp) crate.
- **Output**: Uses the `console` crate for coloured terminal output.

## Code Structure

```
src/
  main.rs  - CLI entry point
  lib.rs   - Core functions: resolve_ip, query_server, compare_servers
```

Unit and CLI tests are in the `tests/` directory.

## Environment Setup

Ensure you have the latest stable Rust toolchain:

```bash
rustup install stable
rustup default stable
```

Run tests with:

```bash
cargo test
```

## Contribution Guidelines

1. Fork the repository and create your feature branch.
2. Write tests for your changes.
3. Ensure `cargo fmt` runs without modifying files.
4. Submit a pull request and reference the relevant issue.

## CI/CD

The workflow named `ci-test-n-build.yml` runs tests, lints, formats code and checks on dependencies on each push. All of that on 3 different OS using 
matrix.

There is another workflow `release.yaml` which is meant to build packages, binaries and artifacts and upload them to the 
release page.

Packaging scripts for `.deb` and `.rpm` are defined in `Cargo.toml`.

