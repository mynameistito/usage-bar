# UI COMPONENTS

**Scope:** Reusable gauge and settings components
**Pattern:** Functional factories returning DOM elements

## OVERVIEW
Three components: usage gauge, MCP gauge (different data shape), Z.ai settings (API key CRUD).

## STRUCTURE
```
components/
├── UsageGauge.ts          # Circular progress gauge (Claude/Z.ai token usage)
├── McpUsageGauge.ts       # Linear gauge (MCP server usage)
└── ZaiSettings.ts         # API key management UI
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Circular gauges | `UsageGauge.ts` - `createUsageGauge()` | Renders SVG, handles color states |
| MCP gauges | `McpUsageGauge.ts` - `createMcpUsageGauge()` | Shows used/total counts |
| Settings form | `ZaiSettings.ts` - `createZaiSettings()` | Input validation, Tauri commands |

## CONVENTIONS
- **Export pattern:** Named export of factory function: `export function createX()`
- **DOM creation:** `document.createElement()` only — no template literals
- **Class naming:** BEM-style: `gauge`, `gauge--warning`, `gauge--critical`
- **Color thresholds:** <70% (green), 70-90% (yellow), >90% (red)
- **SVG math:** `stroke-dasharray` based on circumference (`2 * π * r`)

## ANTI-PATTERNS (COMPONENTS)
- **NEVER** use innerHTML for user content — XSS risk
- **DO NOT** create global styles — component-scoped via specific classes
- **NEVER** assume parent element exists — null-check before append
- **DO NOT** hardcode threshold values — use constants
- **NEVER** embed Tauri calls directly — components receive data, don't fetch

## UNIQUE STYLES
- **SVG-based gauges:** Circle strokes calculated via CSS custom properties
- **Dual gauge types:** Circular (percentage) vs linear (used/total)
- **Async factory:** `ZaiSettings` fetches existing key on creation
- **Self-contained:** Each file includes all necessary CSS via style tags

## COMPONENT APIS

### `createUsageGauge(options)`
```ts
interface UsageGaugeOptions {
  title: string;           // Display name (e.g., "Session")
  utilization: number;     // 0.0 - 1.0
  resetsAt: string;        // ISO datetime or empty
}
```
**Returns:** `<div class="gauge">` with SVG circle

### `createMcpUsageGauge(options)`
```ts
interface McpUsageGaugeOptions {
  title: string;           // Display name
  percentage: number;      // 0.0 - 1.0
  used: number;            // Actual count
  total: number;           // Max count
}
```
**Returns:** `<div class="mcp-gauge">` with linear bar

### `createZaiSettings()`
```ts
// No params — reads existing key via Tauri on init
```
**Returns:** `<div id="zai-settings">` with form/button elements

## VISUAL STATES
- **gauge--success:** <70% utilization (green)
- **gauge--warning:** 70-90% utilization (yellow/orange)
- **gauge--critical:** >90% utilization (red)
- **empty-state:** Shown when no data available

# NPM
Never use NPM, use Bun