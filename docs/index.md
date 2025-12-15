# RKIK Documentation

RKIK (Rusty Klock Inspection Kit) is a Rust CLI and library to **probe NTP servers**, **compare** their offsets, and **export** results in text or JSON.
Version 1.0.0 ships a clean library API, continuous modes, a **compact JSON** format, and `host[:port]` targets (including `[IPv6]:port`).

## Highlights in 1.0.0
- **Library API**: `query_one(&str, bool, Duration) -> Result<ProbeResult, RkikError>` and
  `compare_many(&[String], bool, Duration) -> Result<Vec<ProbeResult>, RkikError>`.
- **Formats**: `text`, `json`, `simple` (short text), `json-short` (compact).
- **Continuous modes**: `--count`, `--infinite`, `--interval` (JSON Lines recommended).
- **IPv6-only**: `-6` forces AAAA resolution.
- **Disable colors**: `--no-color` (alias `--nocolor`).
- **Targets with port**: `pool.ntp.org:123`, `[2a00:1450::200e]:123`.

## Install RKIK
- **AUR (Arch/Manjaro, dev)**: `rkik-git` — <https://aur.archlinux.org/packages/rkik-git>
  ```bash
  git clone https://aur.archlinux.org/rkik-git.git
  cd rkik-git && makepkg -si
  ```
  With a helper:
  ```bash
  yay -S rkik-git
  ```

- **Nix / NixOS**: package `rkik` (channel dependent). Search:
  <https://search.nixos.org/packages?query=rkik>
  ```bash
  # ephemeral shell
  nix shell nixpkgs#rkik
  # user profile install
  nix-env -iA nixpkgs.rkik
  ```

- **Binaries / RPM / DEB**: See GitHub Releases.
- **From source**:
  ```bash
  cargo build --release
  ```

## Quick start
```bash
rkik --server time.google.com --json
rkik --compare time.cloudflare.com,time.google.com --format json
rkik 0.fr.pool.ntp.org --format json-short
rkik -6 --server 2.pool.ntp.org -j
rkik --server time.cloudflare.com --count 2 --interval 1 --short
```

## Contents
- [User Guide](user_guide.md)
- [Developer Guide](developer_guide.md)
- [API Reference](api_reference.md)
- [Packaging](packaging.md)
- [PTP Implementation Design](PTP_IMPLEMENTATION_DESIGN.md)
- [Local Test Environment](TEST_ENV.md)
