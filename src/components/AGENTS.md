# UI COMPONENTS

**Scope:** Reusable gauge and settings components
**Pattern:** Functional factories returning DOM elements

## OVERVIEW
Four components: circular usage gauge, linear MCP gauge, unified settings panel (Z.ai + Amp credentials).

## STRUCTURE
```
components/
├── UsageGauge.ts          # Circular SVG progress gauge (Claude/Z.ai/Amp token usage)
├── McpUsageGauge.ts       # Linear gauge (MCP server used/total)
└── SettingsView.ts        # Settings panel: Z.ai API key + Amp session cookie CRUD
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Circular gauges | `UsageGauge.ts` - `createUsageGauge()` | SVG, color states, reset timestamp |
| MCP gauges | `McpUsageGauge.ts` - `createMcpUsageGauge()` | Shows used/total counts |
| Settings panel | `SettingsView.ts` - `createSettingsView()` | Receives `SettingsCallbacks` + initial state |
| Z.ai key input | `SettingsView.ts` - `createZaiInputState()` | Validates then saves; env var bypass |
| Amp cookie input | `SettingsView.ts` - `createAmpInputState()` | Validates against live URL then saves |
| Section rebuild | `SettingsView.ts` - `rebuildZaiSection()` / `rebuildAmpSection()` | In-place DOM swap on state change |

## CONVENTIONS
- **Export:** Named export of factory function: `export function createX()`
- **DOM creation:** `document.createElement()` only — no template literals, no innerHTML for user content
- **BEM classes:** `gauge`, `gauge--warning`, `gauge--critical`; `btn`, `btn-primary`, `btn-ghost`, `btn-destructive`
- **Color thresholds:** <70% (green/success), 70-90% (yellow/warning), >90% (red/critical)
- **SVG math:** `stroke-dasharray` based on circumference (`2 * π * r`)
- **Env var syntax:** `{env:VAR}` or `$env:VAR` — detected by `isEnvVarSyntax()`, skips API validation

## ANTI-PATTERNS (COMPONENTS)
- **NEVER** use innerHTML for user-supplied content — XSS risk
- **DO NOT** create global styles — component-scoped via class selectors
- **NEVER** embed Tauri calls directly in gauge components — they receive data, don't fetch
- **DO NOT** hardcode color thresholds — use consistent percentage constants
- **NEVER** assume parent element exists — null-check before append

## UNIQUE STYLES
- **SettingsView pattern:** Receives all callbacks + initial state from `main.ts`; never calls Tauri directly
- **Section rebuild vs re-render:** `rebuildZaiSection()` / `rebuildAmpSection()` clear and replace section children in-place (avoids full panel re-create)
- **Env var auto-show:** Input switches from `type=password` to `type=text` automatically when `{env:` detected
- **Validation flow:** validate → save → `onChanged()` callback → rebuild section
- **Eye icon toggle:** Password visibility button uses inline SVG, re-renders icon on toggle
- **App version:** `SettingsView.ts` imports `package.json` to render version in about section

## COMPONENT APIS

### `createUsageGauge(options)`
```ts
interface UsageGaugeOptions {
  title: string;           // e.g., "Session", "Weekly", "Free Tier Usage"
  utilization: number;     // 0.0–1.0
  resetsAt: string;        // ISO datetime string or "" (hides reset label)
}
```
**Returns:** `<div class="gauge">` with SVG circle

### `createMcpUsageGauge(options)`
```ts
interface McpUsageGaugeOptions {
  title: string;
  percentage: number;      // 0.0–1.0
  used: number;
  total: number;
}
```
**Returns:** `<div class="mcp-gauge">` with linear bar

### `createSettingsView(callbacks, hasZaiApiKey, hasAmpCookie)`
```ts
interface SettingsCallbacks {
  checkZaiApiKey: () => Promise<boolean>;
  validateZaiApiKey: (apiKey: string) => Promise<void>;
  saveZaiApiKey: (apiKey: string) => Promise<void>;
  deleteZaiApiKey: () => Promise<void>;
  onZaiKeyChanged: () => Promise<void>;
  checkAmpSessionCookie: () => Promise<boolean>;
  validateAmpSessionCookie: (cookie: string) => Promise<void>;
  saveAmpSessionCookie: (cookie: string) => Promise<void>;
  deleteAmpSessionCookie: () => Promise<void>;
  onAmpCookieChanged: () => Promise<void>;
  openUrl: (url: string) => Promise<void>;
  onClose: () => void;
}
```
**Returns:** `<div id="settings-view">` — full settings overlay

## VISUAL STATES
- **gauge--success / status-success:** <70% (green dot)
- **gauge--warning:** 70–90% (yellow)
- **gauge--critical:** >90% (red)
- **empty-state:** No data available

## PACKAGE MANAGER
Never use NPM — use Bun
