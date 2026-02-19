# USAGE BAR — PROJECT KNOWLEDGE BASE

**Generated:** 2026-02-19
**Branch:** feat/ampcode

## OVERVIEW
Windows system tray app (Tauri v2) that monitors API usage for Claude Code, Z.ai, and Amp. Rust backend handles auth, caching, and HTTP; vanilla TypeScript frontend renders gauges in a popup window.

## STRUCTURE
```
usage-bar/
├── src/                   # TypeScript frontend (Tauri webview)
│   ├── main.ts            # Orchestration, state, polling, tab logic
│   ├── index.html         # Popup window template
│   ├── components/        # UI components (gauges, settings panel)
│   └── styles/            # CSS (theme, base, components)
├── src-tauri/
│   ├── src/               # Rust backend — services, commands, cache
│   ├── tauri.conf.json    # Window config: 320×560, alwaysOnTop, visible=false
│   └── Cargo.toml         # Deps: tauri 2.10, reqwest 0.11, windows 0.61, regex
├── scripts/               # generate-icon.js (png → ico)
└── package.json           # Bun workspace, vite scripts
```

## WHERE TO LOOK
| Task | Location |
|------|----------|
| App startup + data fetch | `src/main.ts` - `loadContent()` |
| Tab switching (claude/zai/amp) | `src/main.ts` - `switchTab()`, `setupTabSwitching()` |
| Settings modal open/close | `src/main.ts` - `openSettings()`, `closeSettings()` |
| Gauge rendering | `src/components/UsageGauge.ts`, `McpUsageGauge.ts` |
| Credential management UI | `src/components/SettingsView.ts` |
| Tauri command handlers | `src-tauri/src/commands.rs` |
| Claude OAuth + API | `src-tauri/src/claude_service.rs` |
| Amp HTML scraping | `src-tauri/src/amp_service.rs` |
| Z.ai quota API | `src-tauri/src/zai_service.rs` |
| Windows Credential Manager | `src-tauri/src/credentials.rs` |
| In-memory TTL cache | `src-tauri/src/cache.rs` |
| Data structs (serde) | `src-tauri/src/models.rs` |
| Tray setup + state init | `src-tauri/src/main.rs` |
| Window dimensions/CSP | `src-tauri/tauri.conf.json` |

## CONVENTIONS
- **Package manager:** Bun only — NEVER npm or yarn
- **TypeScript checker:** `tsgo` (native preview, not `tsc`)
- **Build:** `bun run dev` (tauri dev) / `bun run build` (tauri build)
- **Frontend:** Vanilla TS + DOM manipulation — no React/Vue/Svelte
- **Styling:** Tailwind CSS v4 via `@tailwindcss/vite`
- **Error propagation (Rust):** `anyhow::Result<T>` internally, `String` at command boundary
- **Debug logging:** `debug_*!()` macros gated on `#[cfg(debug_assertions)]`
- **HTTP clients:** Two separate clients — `HttpClient` (redirects on) for Claude/Z.ai, `AmpHttpClient` (redirects off, browser UA) for Amp
- **Cache TTL:** 30 seconds for all providers

## ANTI-PATTERNS
- **NEVER** use npm/yarn — Bun only
- **NEVER** call `unwrap()` on external data (API responses, credentials)
- **DO NOT** hold Mutex locks across `await` points
- **NEVER** log OAuth tokens or session cookies
- **DO NOT** create new HTTP clients per request — use injected `Arc<Client>`
- **NEVER** use `alert()` in frontend — errors go to DOM error containers
- **DO NOT** add a new provider without both a separate cache + HTTP client state type

## COMMANDS
```bash
bun run dev              # Start tauri dev (hot reload)
bun run build            # Release build (generates icon, bundles)
bun run build:nsis       # NSIS installer
bun run typecheck        # tsgo --noEmit
bun run cargo:check      # cargo check
bun run cargo:clippy     # cargo clippy
bun run cargo:precheck   # check + clippy -D warnings + fmt check
```

## TAURI COMMAND SURFACE
| Category | Commands |
|----------|----------|
| Claude | `claude_get_all`, `claude_get_usage`, `claude_get_tier` |
| Z.ai | `zai_get_all`, `zai_refresh_all`, `zai_get_usage`, `zai_refresh_usage`, `zai_get_tier`, `zai_check_api_key`, `zai_validate_api_key`, `zai_save_api_key`, `zai_delete_api_key` |
| Amp | `amp_get_usage`, `amp_refresh_usage`, `amp_check_session_cookie`, `amp_validate_session_cookie`, `amp_save_session_cookie`, `amp_delete_session_cookie` |
| App | `quit_app`, `refresh_all`, `open_url` |

## NOTES
- Window starts hidden (`visible: false`); shown via `window.show()` after data loads
- Amp data is scraped from `ampcode.com/settings` HTML (no JSON API) — fragile if page structure changes
- `open_url` uses `ShellExecuteW` on Windows; validates `http://`/`https://` prefix before shell call
- Tab selection persisted in `localStorage` under key `"activeTab"`
- Z.ai "not configured" errors silently hide UI (not shown as error state)
