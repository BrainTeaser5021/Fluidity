# Fluidity OS Starter Repository

This bundle contains the starter scaffolding for **Fluidity**, a Fedora Kinoite (`bootc`) + KWin Wayland + Rust desktop shell distribution.

## Contents
- `Containerfile`: Builds an atomic, rollback-safe OCI OS image based on `ghcr.io/ublue-os/kinoite-main:latest` with Flatpak, Distrobox, KWin, and Fluidity's Rust shell/launcher.
- `fluidity-launcher/`: Rust (`iced_layershell` + `nucleo-matcher` + `mistralrs`) dynamic launcher that indexes only user applications (excluding system files and settings) and routes natural-language questions to a local 0.5B–1.5B model.
- `kwin-puddle-effect/`: Plasma 6 KWin shader effect scaffolding for the downward puddle-dissolve window-close animation.

## Quick Build Steps (on a Linux host with Podman & Rust)
1. Build the launcher:
   ```bash
   cd fluidity-launcher && cargo build --release
   ```
2. Build the Fluidity OCI OS image:
   ```bash
   podman build -t localhost/fluidity-os:latest -f Containerfile .
   ```
3. Rebase an existing Fedora Kinoite installation onto your local image:
   ```bash
   sudo bootc switch --transport containers-storage localhost/fluidity-os:latest
   ```
