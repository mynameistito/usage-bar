# FRONTEND CODEBASE

**Scope:** TypeScript UI, components, styles
**Framework:** Vanilla TypeScript + Tauri API bindings

## OVERVIEW
Frontend for usage monitoring popup. No framework — uses DOM manipulation and Tauri invoke API.

## STRUCTURE
```
src/
├── main.ts                # Entry point, orchestration, state management
├── index.html             # Popup window template
├── components/            # Reusable UI components
└── styles/                # CSS architecture (Tailwind + custom)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App initialization | `main.ts` - `loadContent()` | Sets up polling, tabs, initial data fetch |
| API calls to Rust | `main.ts` - `invoke()` calls | All backend communication via `@tauri-apps/api/core` |
| UI updates | `main.ts` - `fetch*Usage()` functions | DOM manipulation after data fetch |
| Gauge rendering | `components/UsageGauge.ts` | Circular progress indicators |
| Settings UI | `components/ZaiSettings.ts` | API key management for Z.ai |
| Theming | `styles/theme.css` | Color palette, dark mode support |
| Component styles | `styles/components.css` | Gauge-specific styles |

## CONVENTIONS
- **Component pattern:** Factory functions return `HTMLElement`, not classes
- **State management:** Module-level variables (`activeTab`, `*LastRefresh`)
- **Error handling:** `try/catch` with user-friendly error messages
- **Timing:** `setInterval` for polling (5min) and timestamp updates (30s)
- **Tauri invoke:** Type-safe via generics: `invoke<ReturnType>('command_name')`

## ANTI-PATTERNS (THIS FRONTEND)
- **NEVER** use `alert()` — errors go to DOM error containers
- **DO NOT** create global variables other than state/timer handles
- **NEVER** call Tauri commands synchronously — always `await`
- **DO NOT** bypass cache — all data fetches check Rust-side cache first
- **NEVER** hardcode polling intervals — use `POLL_INTERVAL` constant

## UNIQUE STYLES
- **Tab-based architecture:** Claude/Z.ai views toggled via CSS display
- **Fire-and-forget initialization:** `fetchClaudeTier()` called without await
- **Relative timestamps:** "Updated Xm ago" updated every 30s independently
- **Error suppression:** Z.ai "not configured" errors hide UI silently
- **Dynamic component replacement:** Settings element replaced after async load

## DATA FLOW
1. `DOMContentLoaded` → `loadContent()`
2. Fetch Claude tier (fire-and-forget)
3. Fetch Claude usage (await)
4. Fetch Z.ai usage (await)
5. Create/replace settings component
6. Show content, hide loading spinner
7. Start polling interval (5min)
8. Start timestamp updater (30s)

## INTEGRATION POINTS
- **Tauri commands:** `get_claude_usage`, `get_claude_tier`, `get_zai_usage`, `save_zai_api_key`
- **DOM IDs:** `loading`, `content`, `claude-view`, `zai-view`, tab buttons
- **Error containers:** `claude-error`, `zai-error` with message spans
