# RUST BACKEND

**Scope:** Tauri v2 app, Windows credential integration, API services
**Language:** Rust (2021 edition)

## OVERVIEW
Tauri v2 desktop app backend. Handles Windows Credential Manager, HTTP requests, caching, and frontend-to-Rust communication.

## STRUCTURE
```
src-tauri/
├── src/                   # Rust source modules
├── Cargo.toml             # Dependencies, build profiles
├── tauri.conf.json        # App config (windows, bundle)
├── build.rs               # Build script (if any)
└── target/                # Rust build output
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App entry point | `src/main.rs` | Tauri setup, tray icon, state management |
| Frontend commands | `src/commands.rs` | `#[tauri::command]` functions |
| Claude API | `src/claude_service.rs` | OAuth token refresh, usage API calls |
| Z.ai API | `src/zai_service.rs` | API key validation, quota fetching |
| Credentials | `src/credentials.rs` | Windows Credential Manager (Win32 API) |
| Data models | `src/models.rs` | Structs for API responses |
| Caching | `src/cache.rs` | Time-based TTL cache |
| Build config | `Cargo.toml` | Dependencies, compiler profiles |
| App metadata | `tauri.conf.json` | Window size, tray, bundle settings |

## CONVENTIONS
- **Error handling:** Return `Result<T, String>` from commands (String = user-facing error)
- **Async runtime:** Tokio multi-threaded, all network calls async
- **Shared state:** Arc-wrapped HTTP client and caches in Tauri state
- **Credential targets:** Hardcoded constants `CLAUDE_TARGET`, `ZAI_TARGET`
- **Debug logging:** `#[cfg(debug_assertions)]` macro, production stripped
- **Cache TTL:** 30 seconds for all API responses

## ANTI-PATTERNS (THIS BACKEND)
- **NEVER** unwrap() in production — use `?` or handle explicitly
- **DO NOT** create new HTTP clients per request — use shared state
- **NEVER** expose OAuth tokens in logs — even debug logs redact sensitive data
- **DO NOT** block async runtime — all I/O must be `.await`
- **NEVER** bypass cache for frequent calls — check cache first

## UNIQUE STYLES
- **Dual credential pattern:** Reads shared Claude creds, manages own Z.ai creds
- **OAuth refresh flow:** Auto-refreshes expired tokens before API calls
- **Win32 raw API:** Direct `windows` crate calls (no credential-manager abstraction)
- **Cache-first pattern:** All Tauri commands check cache before network
- **State injection:** `State<'_, T>` params for shared resources

## DEPENDENCIES
- **tauri 2.0:** Desktop framework, tray icon
- **reqwest 0.11:** HTTP client (rustls-tls only, no OpenSSL)
- **tokio 1.0:** Async runtime (multi-threaded)
- **serde/serde_json:** JSON serialization
- **windows 0.52:** Win32 API bindings (Credentials module)
- **anyhow 1.0:** Error handling (internal-only, not exposed to frontend)

## BUILD PROFILES
- **dev:** Fast compilation (opt-level 0, 256 codegen-units)
- **release:** Optimized (opt-level 3, LTO thin, strip symbols)
- **release-small:** Size-optimized (opt-level s, full LTO, panic=abort)

## TAURI COMMANDS
```rust
claude_get_usage()       // Fetch cached/live Claude usage data
claude_get_tier()        // Fetch Claude plan/tier info
zai_get_usage()          // Fetch cached/live Z.ai quota
zai_check_api_key()      // Validate existing Z.ai key
zai_save_api_key(key)    // Store Z.ai key in Credential Manager
zai_delete_api_key()      // Remove Z.ai key
refresh_all()            // Invalidate all caches
```

## CREDENTIAL STORAGE
- **Claude:** Target = "Claude Code-credentials" (reads Claude Code's entry)
- **Z.ai:** Target = "usage-bar-zai-credentials" (app-specific, read/write)
- **Format:** Claude = JSON with OAuth tokens, Z.ai = plaintext API key

# NPM
Never use NPM, use Bun