# Usage Bar for Windows

A Windows system tray application that displays real-time API usage for that currently has; [Claude](https://claude.ai),  [z.ai](https://z.ai), and [AmpCode](https://ampcode.com).

## Prerequisites

- Windows 10 v1809+ or Windows 11


## Development

```bash
bun install
bun run dev
```

## Building

```bash
# All installers (NSIS + MSI)
bun run build

# NSIS only
bun run build:nsis

# MSI only
bun run build:msi
```

Built installers are placed in `src-tauri/target/release/bundle/`.

## Usage

1. Launch the app — it appears in the system tray
2. **Left-click** the tray icon to open/close the popup panel
3. **Right-click** for the context menu (Open, Quit)
4. Click **Refresh** to manually poll usage data
5. Click the **settings cog** to configure credentials

### Configuring Z.ai

Open Settings, enter your Z.ai Coding Plan API key, and save. You can also pass the key as an environment variable using `{env:VAR_NAME}` or `$env:VAR_NAME` syntax.

### Configuring Amp

Open Settings, Log into your Amp Account, goto Browser Dev Tools, and enter in your Cookie Session Token.

## Credential Storage

| Provider | Storage | Key |
|----------|---------|-----|
| Claude | `~/.claude/.credentials.json` (shared with Claude Code) | n/a |
| Z.ai | Windows Credential Manager | `usage-bar-zai-credentials` |
| Amp | Windows Credential Manager | `usage-bar-amp-credentials` |

Claude credentials are read automatically — no configuration needed if Claude Code is installed and authenticated.

**All data is stored locally, and only used to check usages.

## Troubleshooting

**Claude usage not showing** — Ensure Claude Code is installed and authenticated. Check that `~/.claude/.credentials.json` exists and contains valid credentials.

**Z.ai / Amp credentials invalid** — Delete the existing entry in Settings and re-enter your key/cookie.

**Network errors** — Verify internet connectivity, then hit Refresh.

## Project Structure

```
src/           TypeScript frontend (UI, polling, components)
src-tauri/     Rust backend (credentials, API services, Tauri commands)
```

## Versioning

This project uses [Changesets](https://github.com/changesets/changesets) for changelog and version management.

```bash
# Add a changeset for your change
bun run changeset

# Bump versions and update CHANGELOG.md
bun run version
```

The `version` command automatically syncs the version from `package.json` into `src-tauri/Cargo.toml`.

### Liscence
MIT
