# USAGE BAR - PROJECT KNOWLEDGE BASE

**Generated:** 2026-02-12
**Commit:** 4f274e9
**Branch:** main

## OVERVIEW
Windows system tray application for monitoring Claude (Anthropic) and Z.ai (GLM-4.7) API usage. Built with Tauri v2, Rust backend, vanilla TypeScript frontend.

## STRUCTURE
```
usage-bar/
├── src/                    # Frontend (TypeScript, HTML, CSS)
├── src-tauri/              # Rust backend + native Windows integration
├── scripts/                # Build utilities (icon generation)
├── dist/                   # Build output (frontend bundle)
└── usage-bar/              # Orphaned iOS project (not integrated)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| API endpoints/services | `src-tauri/src/*_service.rs` | Claude & Z.ai API integration |
| UI components | `src/components/*.ts` | Gauge components, settings |
| Credential storage | `src-tauri/src/credentials.rs` | Windows Credential Manager |
| Tauri commands | `src-tauri/src/commands.rs` | Frontend ↔ Rust bridge |
| Styling | `src/styles/*.css` | Tailwind v4 + custom |
| Build config | `src-tauri/tauri.conf.json`, `vite.config.ts` | App metadata, bundle config |

## CONVENTIONS
- **Package manager:** Bun (not npm/yarn/pnpm)
- **Credential targets:** `Claude Code-credentials` (shared), `usage-bar-zai-credentials` (app-specific)
- **API caching:** 30-second TTL via `ResponseCache`
- **Polling:** 5-minute intervals for usage data
- **Error handling:** Return `Result<T, String>` from Tauri commands
- **Component pattern:** Factory functions (`createUsageGauge()`) returning DOM elements
- **Rust modules:** Flat structure (no `lib.rs`, all modules at `src/*.rs`)

## ANTI-PATTERNS (THIS PROJECT)
- **NEVER** hardcode API keys or tokens
- **DO NOT** bypass Windows Credential Manager for credential storage
- **NEVER** call API endpoints without cache check first
- **DO NOT** block the UI thread — all network calls are async
- **NEVER** expose OAuth tokens in logs — use debug macros conditionally
- **DO NOT** use `unwrap()` in production Rust code

## UNIQUE STYLES
- **Dual-provider architecture:** Claude (OAuth) vs Z.ai (API key)
- **Shared credential pattern:** Reads Claude Code's existing credentials
- **Fire-and-forget pattern:** `fetchClaudeTier()` runs without awaiting
- **Tab-based UI:** Separate views for Claude/Z.ai in single window
- **Tray-first design:** Primary interaction via system tray, not window
- **No frontend framework:** Vanilla TypeScript with DOM manipulation
- **Tailwind v4:** Via `@tailwindcss/vite` plugin (bleeding edge)
- **Direct Win32 API:** Raw `windows` crate for credential storage

## COMMANDS
```bash
# Development
bun run dev                # Start Tauri dev server with hot reload

# Building
bun run build              # Full build with icon generation
bun run build:nsis         # NSIS installer only
bun run build:msi          # MSI installer only

# Utilities
bun run generate-icon      # Generate .ico from source PNG
```

## NOTES
- **No test infrastructure:** Zero test files or CI/CD pipelines
- **High-DPI support:** Scales 100%-250% via CSS/Windows settings
- **Credential sharing:** Leverages `Claude Code-credentials` entry created by Claude Code desktop app
- **Build artifacts:** Final installers in `src-tauri/target/release/bundle/`
- **Debug logging:** Only enabled in debug builds (`#[cfg(debug_assertions)]`)
- **Icon format:** Requires PNG source → ICO conversion for Windows tray
- **Cargo.lock gitignored:** Non-standard for application (should be committed)
