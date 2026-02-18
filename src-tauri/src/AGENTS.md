# CORE RUST MODULES

**Scope:** Business logic, API integration, credential management
**Architecture:** Service-oriented, stateless functions + shared state

## OVERVIEW
Six modules: credentials (Win32), claude_service, zai_service, cache, models, commands (Tauri bridge).

## STRUCTURE
```
src/
├── main.rs                # Tauri setup, state initialization, tray menu
├── commands.rs            # Tauri command handlers (frontend bridge)
├── credentials.rs         # Windows Credential Manager wrapper
├── claude_service.rs      # Anthropic API + OAuth refresh flow
├── zai_service.rs        # Z.ai API + quota parsing
├── cache.rs              # In-memory TTL cache implementation
└── models.rs             # Data structures (API responses)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| HTTP client setup | `main.rs` - `HttpClient` state | Shared reqwest::Client, 15s timeout |
| Cache init | `main.rs` - `*UsageCache` state | 30-second TTL for both providers |
| Tray menu | `main.rs` - `TrayIconBuilder` | Open/Quit menu items |
| Claude OAuth flow | `claude_service.rs` - `check_and_refresh_if_needed()` | Token refresh logic |
| Z.ai quota parsing | `zai_service.rs` - `fetch_usage()` | Complex API response parsing |
| Credential I/O | `credentials.rs` - `read_credential()`, `save_credential()` | Win32 raw API |
| Cache logic | `cache.rs` - `get()`, `set()` | Instant + Mutex |
| Data shapes | `models.rs` - `UsageData`, `ZaiUsageData` | Serde structs |

## CONVENTIONS
- **Module visibility:** All modules `pub mod`, most functions `pub fn` for testing
- **Error propagation:** Internal functions return `anyhow::Result<T>`, commands convert to `String`
- **Async functions:** All network calls are `async fn`, take `Arc<reqwest::Client>`
- **Credential passwords:** Always `None` (default credentials, not user-password)
- **JSON parsing:** Strict via `serde_json::from_str` — fails fast on malformed data

## ANTI-PATTERNS (CORE MODULES)
- **NEVER** call `unwrap()` on external data (API responses, credentials)
- **DO NOT** create HTTP clients per request — use injected Arc<Client>
- **NEVER** log OAuth tokens — use `debug_log!` macro only
- **DO NOT** hold locks across await points — MutexGuard scope limited
- **NEVER** parse dates manually — use string passthrough to frontend

## UNIQUE STYLES
- **Dual-layer caching:** Cache struct wraps Mutex<Option<(Instant, T)>>
- **OAuth token refresh:** Checks expiry, calls refresh endpoint, updates credential
- **Credential sharing:** Reads Claude Code's credential entry directly
- **Service pattern:** Each provider has static methods, no instances needed
- **State injection:** Commands receive `State<'_, T>` for client/cache

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

Response shape: nested object with token_usage.mcp_usage paths
```

## DATA MODELS (models.rs)
```rust
pub struct UsageData {           // Claude response
  pub five_hour_utilization: f64,
  pub five_hour_resets_at: String,
  pub seven_day_utilization: f64,
  pub seven_day_resets_at: String,
  pub extra_usage_enabled: bool,
  pub extra_usage_monthly_limit: Option<f64>,
  pub extra_usage_used_credits: Option<f64>,
  pub extra_usage_utilization: Option<f64>,
}

pub struct ClaudeTierData {      // Claude plan info
  pub plan_name: String,
  pub rate_limit_tier: String,
}

pub struct ZaiUsageData {        // Z.ai response (reconstructed)
  pub token_usage: Option<TokenUsage>,
  pub mcp_usage: Option<McpUsage>,
}
```

## CREDENTIAL MANAGER (credentials.rs)
- **Target constants:** `CLAUDE_TARGET`, `ZAI_TARGET`
- **Win32 functions:** `CredReadW`, `CredWriteW` via `windows` crate
- **Credential blob:** UTF-8 JSON string (Claude), plaintext key (Z.ai)
- **Error handling:** Returns `String` error messages for frontend display

## CACHE STRATEGY (cache.rs)
```rust
pub struct ResponseCache<T> {
  data: Mutex<Option<(Instant, T)>>,
  ttl_seconds: u64,
}

// Thread-safe: get() clones, set() overwrites
// TTL checked on read, auto-expires after N seconds
```

## COMMAND BRIDGE (commands.rs)
- **Debug logging:** Conditional `#[cfg(debug_assertions)]` macro
- **Cache-first:** All commands check cache before network call
- **Error mapping:** anyhow::Error → String via `to_string()`
- **State access:** `State<'_, HttpClient>`, `State<'_, *UsageCache>`


# NPM
Never use NPM, use Bun