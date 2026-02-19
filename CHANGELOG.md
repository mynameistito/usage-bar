# Changelog

## 1.0.4

### Patch Changes

- 5748c91: Add automated release pipeline that builds and attaches Windows x64 and ARM64 NSIS/MSI installers to GitHub Releases. Also fixes version sync to keep `tauri.conf.json` in step with `package.json` and `Cargo.toml`.

## 1.0.3

### Patch Changes

- df23b1c: Use pre-generated PNG/ICO icons derived from SVG logo, with larger, more prominent usage bars. Fix CI Rust cache never being saved by adding `save-always: true` to `Swatinem/rust-cache@v2`.

## 1.0.2

### Patch Changes

- 814dfc1: Add CI workflow that runs TypeScript type checking (via tsgo), linting, and Rust checks (cargo check, clippy, fmt, tests) on pull requests

## 1.0.1

### Patch Changes

- d5c7d49: Set up Changesets, release workflow, clean up README and add assets (logo).
- 51ffe1b: Fix release CI failing due to lefthook pre-commit hooks running cargo:precheck on Ubuntu, which requires Linux GTK/glib system libraries that are not installed on the runner.

All notable changes to this project will be documented in this file.

## 1.0.0 — 2026-02-19

### Features

- **Amp (ampcode.com) integration** — new provider panel displaying Amp token usage, quota, and period reset time; session cookie stored in Windows Credential Manager
- **General Settings panel** — unified slide-up settings view replaces the former Z.ai-only modal; manages both Z.ai API key and Amp session cookie from one place
- **Z.ai quota monitoring** — displays used/limit token counts and tier for Z.ai (GLM) accounts
- **Claude usage monitoring** — reads OAuth tokens shared with Claude Code; no additional authentication required
- **Environment variable support** — Z.ai API key can be supplied as `$ENV_VAR` syntax instead of a raw value
- **Windows Credential Manager storage** — Z.ai and Amp credentials are persisted securely via Win32 `CredReadW`/`CredWriteW`; Claude credentials are read from `~/.claude/.credentials.json`
- **Response caching** — per-provider TTL cache minimises redundant API calls (5-minute poll interval)
- **Cargo build profiles** — optimised `dev`, `release`, and `release-small` profiles for fast iteration and small production binaries
