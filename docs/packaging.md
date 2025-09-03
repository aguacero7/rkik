# Packaging

## Arch Linux (AUR)
- Development package **`rkik-git`**:
  <https://aur.archlinux.org/packages/rkik-git>
  ```bash
  git clone https://aur.archlinux.org/rkik-git.git
  cd rkik-git && makepkg -si
  ```
  With a helper:
  ```bash
  yay -S rkik-git
  ```

## Nix / NixOS
- Package **`rkik`** in **nixpkgs** (channel dependent). Search:
  <https://search.nixos.org/packages?query=rkik>
  ```bash
  nix shell nixpkgs#rkik
  # or
  nix-env -iA nixpkgs.rkik
  ```
  To add into a NixOS configuration:
  ```nix
  { pkgs, ... }:
  {
    environment.systemPackages = with pkgs; [ rkik ];
  }
  ```

## DEB / RPM
- Packaging metadata is kept in `Cargo.toml` (`package.metadata.deb` / `package.metadata.generate-rpm`).
- `.deb` / `.rpm` artifacts are built in CI and published in Releases.

## From source
- crates.io publication: `cargo publish` (source-only).
- Local build:
  ```bash
  cargo build --release
  ```
