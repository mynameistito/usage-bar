# FRONTEND CODEBASE

**Scope:** TypeScript UI, components, styles
**Framework:** Vanilla TypeScript + Tauri API bindings

## OVERVIEW
Frontend for usage monitoring popup. Three-tab UI (Claude / Z.ai / Amp). No framework — DOM manipulation and `@tauri-apps/api` invoke. Settings panel slides in over main content.

## STRUCTURE
```
src/
├── main.ts                # Entry, orchestration, state, tab logic, settings modal
├── index.html             # Popup window template (320×560)
├── components/            # Reusable UI components
└── styles/                # CSS (theme.css, base.css, components.css)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App initialization | `main.ts` - `loadContent()` | Fetches all providers, shows window after load |
| Tab switching | `main.ts` - `switchTab()` | CSS display toggle; tab saved to `localStorage` |
| Settings modal | `main.ts` - `openSettings()` / `closeSettings()` | Slides over content; animated close |
| Connection badges | `main.ts` - `createOrUpdateConnectionBadge()` | Shared badge logic for Z.ai + Amp headers |
| Claude data fetch | `main.ts` - `fetchClaudeData()` | Calls `claude_get_all`, renders 2 gauges + extra usage |
| Z.ai data fetch | `main.ts` - `fetchZaiData()` | Calls `zai_get_all` / `zai_refresh_all` |
| Amp data fetch | `main.ts` - `fetchAmpData()` | Calls `amp_get_usage` / `amp_refresh_usage` |
| Gauge rendering | `components/UsageGauge.ts` | SVG circular progress |
| MCP gauge | `components/McpUsageGauge.ts` | Linear used/total bar |
| Settings UI | `components/SettingsView.ts` | Z.ai API key + Amp session cookie management |
| Theming | `styles/theme.css` | Color palette, dark mode |
| Component styles | `styles/components.css` | Gauge-specific CSS |

## CONVENTIONS
- **Component pattern:** Factory functions return `HTMLElement` — not classes
- **State:** Module-level variables (`activeTab`, `*LastRefresh`, `hasAmpSession`)
- **Error handling:** `try/catch` → DOM error containers, never `alert()`
- **Polling:** `POLL_INTERVAL = 300000` (5min); timestamp updates every 30s
- **Tauri invoke:** Typed generics: `invoke<ReturnType>('command_name')`
- **Z.ai API key cache:** 5s client-side TTL via `cachedZaiApiKeyCheck` (avoids log spam)
- **Settings guard:** `settingsOpening` flag prevents duplicate panel creation

## ANTI-PATTERNS (THIS FRONTEND)
- **NEVER** use `alert()` — errors go to DOM error containers
- **DO NOT** create global variables beyond state/timer handles
- **NEVER** call Tauri commands synchronously — always `await`
- **DO NOT** bypass cache — all data fetches go through Rust-side cache
- **NEVER** hardcode polling interval — use `POLL_INTERVAL` constant
- **DO NOT** import Tauri window API at module top — lazy import in `loadContent()`

## UNIQUE STYLES
- **Tab persistence:** `localStorage.getItem("activeTab")` restores last-used tab
- **Settings as overlay:** `content` hidden via `display:none` while settings open; restored on close with slide-out animation
- **Amp conditional init:** `fetchAmpData()` only called on startup if session cookie exists
- **`Promise.allSettled`:** Used for all multi-provider fetches (never `Promise.all`) so one failure doesn't block others
- **Z.ai "not configured" suppression:** Error silently hides UI instead of showing error state
- **`refresh_all` command:** Single Rust command fetches all three providers in parallel via `tokio::join!`

## DATA FLOW (STARTUP)
1. `DOMContentLoaded` → `loadContent()`
2. Check Z.ai API key + Amp cookie (parallel)
3. Update connection badges
4. `Promise.allSettled([fetchClaudeData, fetchZaiData, fetchAmpData?])`
5. Hide spinner, show content
6. Setup tabs, restore saved tab
7. `window.show()` (Tauri)
8. Start polling (5min) + timestamp updater (30s)

## INTEGRATION POINTS
- **Tauri commands:** See root AGENTS.md command surface table
- **DOM IDs:** `loading`, `content`, `app`, `claude-view`, `zai-view`, `amp-view`, tab buttons, `settings-view`
- **Error containers:** `claude-error`, `zai-error`, `amp-error` with message spans
- **Connection status:** `zai-connected-status`, `amp-connected-status`

## PACKAGE MANAGER
Never use NPM — use Bun
