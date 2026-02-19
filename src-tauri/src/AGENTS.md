# CORE RUST MODULES

**Scope:** Business logic, API integration, credential management
**Architecture:** Service-oriented, stateless functions + shared state via Tauri `manage()`

## OVERVIEW
Seven modules: credentials (Win32), claude_service, zai_service, amp_service, cache, models, commands (Tauri bridge).

## STRUCTURE
```
src/
├── main.rs                # Tauri setup, state init (clients + caches), tray menu
├── commands.rs            # Tauri command handlers (frontend bridge)
├── credentials.rs         # Windows Credential Manager wrapper (Win32 raw API)
├── claude_service.rs      # Anthropic API + OAuth refresh flow
├── zai_service.rs         # Z.ai quota API + response parsing
├── amp_service.rs         # Amp: HTML scraping + regex JS object extraction
├── cache.rs               # In-memory TTL cache (Mutex<Option<(Instant, T)>>)
├── logging.rs             # Debug macro definitions + ANSI color constants
└── models.rs              # Serde data structures for all three providers
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| HTTP client setup | `main.rs` - `HttpClient` / `AmpHttpClient` state | Two clients: redirects-on vs redirects-off |
| Cache init (30s TTL) | `main.rs` - `*UsageCache` / `*TierCache` state | 5 separate caches (Claude×2, Z.ai×2, Amp×1) |
| Tray menu | `main.rs` - `TrayIconBuilder` | Open/Quit menu items |
| Claude OAuth refresh | `claude_service.rs` - `check_and_refresh_if_needed()` | Token expiry check + refresh endpoint call |
| Claude usage+tier fetch | `claude_service.rs` - `claude_fetch_usage_and_tier()` | Returns both in one request |
| Z.ai quota parsing | `zai_service.rs` - `zai_fetch_quota()` | Nested JSON → ZaiUsageData |
| Amp HTML scraping | `amp_service.rs` - `amp_fetch_usage()` | GETs `/settings`, regex-extracts `freeTierUsage` JS object |
| Amp auth detection | `amp_service.rs` - redirect checks | Detects 3xx login redirect (redirects disabled on AmpHttpClient) |
| Amp regex extraction | `amp_service.rs` - `extract_number()` / `extract_number_optional()` | Cached regex via `LazyLock` |
| Credential I/O | `credentials.rs` - `read_credential()` / `save_credential()` | Win32 `CredReadW`/`CredWriteW` |
| Cache logic | `cache.rs` - `get()`, `set()`, `clear()` | Thread-safe, TTL checked on read |
| Data shapes | `models.rs` | `UsageData`, `ZaiUsageData`, `AmpUsageData`, tier structs |
| Debug macros | `logging.rs` | `debug_claude!`, `debug_amp!`, `debug_net!`, etc. |

## CONVENTIONS
- **Module visibility:** All modules `pub mod`, most functions `pub fn`
- **Error propagation:** Internal → `anyhow::Result<T>`; Tauri commands → `Result<T, String>`
- **Async:** All network calls are `async fn`, take `Arc<reqwest::Client>`
- **State injection:** Commands receive `State<'_, HttpClient>` etc. — never construct clients in commands
- **Debug logging:** `debug_*!()` macros only, gated on `#[cfg(debug_assertions)]`; NEVER log tokens/cookies

## ANTI-PATTERNS (CORE MODULES)
- **NEVER** call `unwrap()` on external data (API responses, credentials)
- **DO NOT** create HTTP clients per request — use injected `Arc<Client>`
- **NEVER** log OAuth tokens or session cookies — use redacted placeholders
- **DO NOT** hold `MutexGuard` across `await` points
- **NEVER** parse dates manually — pass strings to frontend
- **DO NOT** add a new provider without its own cache type(s) in `main.rs`

## UNIQUE STYLES
- **Two HTTP clients:** `HttpClient` (redirects enabled) for Claude/Z.ai JSON APIs; `AmpHttpClient` (redirects disabled, browser User-Agent) for Amp HTML scraping so 302→login is detectable
- **Dual-layer caching:** `ResponseCache<T>` wraps `Mutex<Option<(Instant, T)>>` — TTL checked on `get()`
- **Amp scraping:** `parse_free_tier_usage()` brace-counts to extract JS object, then regex-matches numeric fields — brittle to ampcode.com HTML changes
- **OAuth token refresh:** Reads Claude Code's own credential entry; refreshes if expired, writes updated token
- **`refresh_all` parallel fetch:** `tokio::join!` on all three providers, returns `Option<T>` per provider
- **Credential sharing:** Claude service reads from Claude Code's credential store directly

## API INTEGRATIONS

### Claude Service
```
GET /api/oauth/usage
  Headers: Authorization: Bearer {token}, anthropic-beta: oauth-2025-04-20

Token Refresh (if expired):
  POST /api/oauth/token
  Body: {"grant_type": "refresh_token", "refresh_token": ...}
```

### Z.ai Service
```
GET /api/monitor/usage/quota/limit
  Headers: Authorization: {apiKey}, Content-Type: application/json

Response: nested JSON with token_usage.mcp_usage paths
```

### Amp Service
```
GET https://ampcode.com/settings
  Headers: Cookie: session={cookie}, Accept: text/html, Referer: ampcode.com

Auth detection: redirect to */login|signin|auth* (redirects disabled → 302→login always surfaces as 3xx)
Data: regex-extract freeTierUsage:{quota, used, hourlyReplenishment, windowHours} from embedded JS
Units: values in cents → divided by 100 for dollar display
resets_at: computed from windowHours aligned to Unix epoch
```

## DATA MODELS (models.rs)
```rust
pub struct UsageData {              // Claude response
  pub five_hour_utilization: f64,
  pub five_hour_resets_at: String,
  pub seven_day_utilization: f64,
  pub seven_day_resets_at: String,
  pub extra_usage_enabled: bool,
  pub extra_usage_monthly_limit: Option<f64>,
  pub extra_usage_used_credits: Option<f64>,
  pub extra_usage_utilization: Option<f64>,
}

pub struct ClaudeTierData { pub plan_name: String, pub rate_limit_tier: String }

pub struct ZaiUsageData { pub token_usage: Option<TokenUsage>, pub mcp_usage: Option<McpUsage>, pub tier_name: Option<String> }
pub struct ZaiTierData { pub plan_name: String }

pub struct AmpUsageData {           // Computed from scraped HTML
  pub quota: f64,                   // dollars (cents/100)
  pub used: f64,
  pub used_percent: f64,            // 0–100, clamped
  pub hourly_replenishment: f64,
  pub window_hours: Option<f64>,
  pub resets_at: Option<i64>,       // epoch millis
}
```

## CREDENTIAL MANAGER (credentials.rs)
- **Targets:** `CLAUDE_TARGET` (Claude Code's own credential), `ZAI_TARGET`, `AMP_TARGET`
- **Win32 functions:** `CredReadW`, `CredWriteW`, `CredDeleteW`
- **Credential blob:** UTF-8 JSON (Claude), plaintext key (Z.ai), plaintext cookie value (Amp)

## CACHE STRATEGY (cache.rs)
```rust
pub struct ResponseCache<T: Clone> {
  data: Mutex<Option<(Instant, T)>>,
  ttl_seconds: u64,
}
// get() → clones on hit, returns None on miss/expired
// set() → overwrites
// clear() → sets to None (force refresh)
```

## COMMAND BRIDGE (commands.rs)
- **Cache-first:** All `get_*` commands check cache before network; `refresh_*` commands call `clear()` first
- **Error mapping:** `anyhow::Error` → `String` via `.to_string()`
- **`open_url`:** Validates `http(s)://` prefix; uses `ShellExecuteW` on Windows with COM init
- **`refresh_all`:** `tokio::join!` across all three providers; partial failures return `None` (not error)

## PACKAGE MANAGER
Never use NPM — use Bun
