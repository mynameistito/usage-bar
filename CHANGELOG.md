# Changelog

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
