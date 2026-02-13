# Usage Bar — Feature Roadmap & Ideas

**Generated:** 2026-02-13  
**Based on:** Full codebase analysis (commit 4f274e9, branch main)

---

## Codebase Summary

Usage Bar is a Windows system tray app (Tauri v2 + Rust + vanilla TypeScript) that monitors API usage for **Claude** (Anthropic) and **Z.ai** (GLM-4.7). It reads Claude Code's OAuth credentials from `~/.claude/.credentials.json`, stores Z.ai keys via Windows Credential Manager, and displays usage gauges in a 320×560 popup window. Data is polled every 5 minutes with a 30-second backend cache.

### Current Strengths
- Clean dual-provider architecture with cache-first data flow
- Solid Win32 credential integration (shared Claude Code creds + own Z.ai creds)
- Atomic credential writes, OAuth auto-refresh, poisoned-mutex recovery
- Polished dark UI with Satoshi font, shadcn-inspired theme, staggered animations

### Current Gaps
- No notifications or alerts — purely passive monitoring
- "Cost" section is hardcoded placeholder (`$0.00 · 0 tokens`)
- No usage history or trends — only live snapshot
- No settings panel (Settings button only opens Z.ai modal)
- No test infrastructure, no CI/CD
- No auto-update mechanism
- Single-platform (Windows only, hardcoded `USERPROFILE`, Win32 creds)

---

## Feature Ideas

### 1. ⚡ Usage Threshold Notifications (Windows Toast)

**Priority:** High · **Effort:** Medium  
**Why:** Users currently have no way to know they're close to rate limits without manually opening the app.

**Scope:**
- Add Windows toast notifications via `tauri-plugin-notification` when usage crosses configurable thresholds (e.g., 70%, 90%, 100%)
- Store notification preferences in a local config file (`%APPDATA%/usage-bar/config.json`)
- Rust: new `notification_service.rs` module, new `NotificationConfig` model
- Frontend: Add notification toggle + threshold sliders to a new Settings view
- Cooldown logic to avoid notification spam (e.g., max 1 per threshold per reset period)

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/Cargo.toml` | Add `tauri-plugin-notification` |
| Backend | `src-tauri/src/notification_service.rs` | New — threshold checking + toast dispatch |
| Backend | `src-tauri/src/config.rs` | New — user preferences (JSON file in AppData) |
| Backend | `src-tauri/src/commands.rs` | New commands: `get_settings`, `save_settings` |
| Backend | `src-tauri/src/main.rs` | Register plugin + new state |
| Frontend | `src/components/Settings.ts` | New — settings panel component |
| Frontend | `src/main.ts` | Wire settings button for Claude tab too |

---

### 2. 📊 Usage History & Sparkline Trends

**Priority:** High · **Effort:** Medium-High  
**Why:** Seeing only a live snapshot makes it impossible to understand usage patterns or plan work sessions.

**Scope:**
- Persist usage snapshots to a local SQLite database (via `tauri-plugin-sql` or raw `rusqlite`)
- Store a row per poll: `(timestamp, provider, five_hour_util, seven_day_util, ...)`
- Frontend: render a mini sparkline chart (last 24h or 7d) below each gauge
- Use `<canvas>` or inline SVG polyline for the sparkline — no charting library needed
- Add a "History" expandable section per provider

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/Cargo.toml` | Add `rusqlite` or `tauri-plugin-sql` |
| Backend | `src-tauri/src/history.rs` | New — DB schema, insert/query functions |
| Backend | `src-tauri/src/commands.rs` | New commands: `get_usage_history` |
| Backend | `src-tauri/src/main.rs` | Initialize DB on startup, add state |
| Frontend | `src/components/Sparkline.ts` | New — SVG sparkline component |
| Frontend | `src/main.ts` | Fetch + render history data |

**DB Schema:**
```sql
CREATE TABLE usage_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  provider TEXT NOT NULL,          -- 'claude' | 'zai'
  metric TEXT NOT NULL,            -- 'five_hour' | 'seven_day' | 'token' | 'mcp'
  value REAL NOT NULL,             -- utilization percentage
  recorded_at INTEGER NOT NULL     -- unix timestamp
);
CREATE INDEX idx_history_lookup ON usage_history(provider, metric, recorded_at);
```

---

### 3. 💰 Live Cost Tracking (Replace Placeholder)

**Priority:** High · **Effort:** Low-Medium  
**Why:** The "Cost" section currently shows hardcoded `$0.00` — it's a broken promise in the UI.

**Scope:**
- Option A: If Anthropic's `/usage` endpoint returns cost data, parse and display it
- Option B: Estimate cost from utilization + known tier pricing (approximate but useful)
- Option C: Remove the section entirely if no data source exists (honest > broken)
- If keeping: accumulate daily cost from usage history DB (idea #2 dependency)

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/src/models.rs` | Add cost fields to `UsageData` if API provides them |
| Backend | `src-tauri/src/claude_service.rs` | Parse cost from response |
| Frontend | `src/index.html` | Update cost section markup |
| Frontend | `src/main.ts` | Populate cost data dynamically |

---

### 4. 🔔 Tray Icon Dynamic Badge / Color

**Priority:** Medium · **Effort:** Low  
**Why:** The tray icon is static — users can't glance at it to know their usage status without clicking.

**Scope:**
- Generate tray icons dynamically with color overlays: green (< 70%), yellow (70-90%), red (> 90%)
- Use `tauri::image::Image` to composite a colored dot onto the base icon
- Update icon on every poll cycle based on highest utilization across providers
- Optional: show numeric badge (e.g., "85%") on hover via tray tooltip

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/src/main.rs` | Dynamic tray icon update logic |
| Backend | `src-tauri/src/commands.rs` | Emit tray update after data fetch |
| Assets | `src-tauri/icons/` | Add colored overlay variants or generate at runtime |

---

### 5. ⚙️ General Settings Panel

**Priority:** Medium · **Effort:** Medium  
**Why:** The Settings button currently only opens Z.ai modal, and only on Z.ai tab. No way to configure polling interval, startup behavior, etc.

**Scope:**
- New full-width settings view (replaces content area, back button to return)
- Configurable options:
  - Polling interval (1m, 2m, 5m, 10m)
  - Launch on Windows startup (registry key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
  - Notification preferences (ties into idea #1)
  - Theme selection (dark/light/system — CSS variables already support it)
  - Show/hide providers
- Persist to `%APPDATA%/usage-bar/config.json`

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/src/config.rs` | New — config struct + file I/O |
| Backend | `src-tauri/src/commands.rs` | `get_config`, `save_config`, `set_autostart` |
| Frontend | `src/components/SettingsView.ts` | New — full settings UI |
| Frontend | `src/main.ts` | Settings navigation, apply config on load |

---

### 6. ⌨️ Global Hotkey to Toggle Window

**Priority:** Medium · **Effort:** Low  
**Why:** Opening the tray popup requires precise mouse clicking. A hotkey (e.g., `Ctrl+Shift+U`) would be faster.

**Scope:**
- Use `tauri-plugin-global-shortcut` to register a configurable hotkey
- Toggle window visibility (show/focus if hidden, hide if visible)
- Store hotkey preference in config (idea #5)

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/Cargo.toml` | Add `tauri-plugin-global-shortcut` |
| Backend | `src-tauri/src/main.rs` | Register shortcut in setup |
| Backend | `src-tauri/tauri.conf.json` | Enable global-shortcut permission |

---

### 7. 🔌 Plugin Architecture for Additional Providers

**Priority:** Medium · **Effort:** High  
**Why:** Other AI coding tools (Cursor, Windsurf, GitHub Copilot) also have usage limits. Making providers pluggable future-proofs the app.

**Scope:**
- Define a `Provider` trait in Rust:
  ```rust
  trait UsageProvider {
      fn name(&self) -> &str;
      fn fetch_usage(&self, client: Arc<Client>) -> Result<ProviderUsage>;
      fn has_credentials(&self) -> bool;
  }
  ```
- Refactor `ClaudeService` and `ZaiService` to implement the trait
- Frontend: dynamically generate tabs from provider list
- Config file lists enabled providers

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/src/provider.rs` | New — trait definition |
| Backend | `src-tauri/src/claude_service.rs` | Implement `UsageProvider` |
| Backend | `src-tauri/src/zai_service.rs` | Implement `UsageProvider` |
| Backend | `src-tauri/src/commands.rs` | Generic provider commands |
| Frontend | `src/main.ts` | Dynamic tab generation |

---

### 8. 🔄 Auto-Update Mechanism

**Priority:** Medium · **Effort:** Medium  
**Why:** No CI/CD or update mechanism means users must manually download new versions.

**Scope:**
- Use `tauri-plugin-updater` with GitHub Releases as the update source
- Check for updates on startup (configurable in settings)
- Show update-available badge in the UI
- NSIS/MSI installer already works — just needs a signed update manifest

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/Cargo.toml` | Add `tauri-plugin-updater` |
| Backend | `src-tauri/tauri.conf.json` | Configure updater endpoint |
| Backend | `src-tauri/src/main.rs` | Register updater plugin |
| CI | `.github/workflows/release.yml` | New — build + publish on tag |
| Frontend | `src/main.ts` | Update notification UI |

---

### 9. 🧪 Test Infrastructure

**Priority:** Medium · **Effort:** Medium  
**Why:** Zero tests in the entire project. Any refactor is high-risk.

**Scope:**
- **Rust unit tests:** cache logic, model parsing, tier inference, credential (mocked)
- **Rust integration tests:** service response handling with mock HTTP (via `mockito` or `wiremock`)
- **Frontend tests:** Component factory output validation (via `vitest` + `happy-dom`)
- **CI pipeline:** GitHub Actions workflow for `cargo test` + `bun test`

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/src/cache.rs` | Add `#[cfg(test)] mod tests` |
| Backend | `src-tauri/src/models.rs` | Add deserialization tests |
| Backend | `src-tauri/src/claude_service.rs` | Add tier inference tests |
| Backend | `src-tauri/Cargo.toml` | Add `mockito`/`wiremock` as dev-dependency |
| Frontend | `src/components/*.test.ts` | New — component unit tests |
| Root | `vitest.config.ts` | New — test config |
| CI | `.github/workflows/test.yml` | New — CI pipeline |

---

### 10. 🖥️ Window Positioning & Anchor to Tray

**Priority:** Low · **Effort:** Low  
**Why:** The window currently opens centered on screen. Tray apps should anchor to the tray icon position (like Windows Action Center).

**Scope:**
- On tray icon click, get cursor position or tray icon bounds
- Position window adjacent to the taskbar (bottom-right for bottom taskbar, etc.)
- Handle multi-monitor setups
- Remember last position as fallback

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/src/main.rs` | Window positioning logic in tray click handler |
| Backend | `src-tauri/tauri.conf.json` | Set `visible: false` on startup (show on tray click) |

---

### 11. 📋 Copy Usage Summary to Clipboard

**Priority:** Low · **Effort:** Low  
**Why:** Quick way to share current usage status in team chats or logs.

**Scope:**
- Add a "Copy" button in the header area
- Format: `Claude: Session 45% (resets 2h 30m) · Weekly 23% | Z.ai: Token 60% · MCP 12/400`
- Use `tauri-plugin-clipboard-manager` or `navigator.clipboard.writeText()`

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Frontend | `src/main.ts` | Compose summary string, add copy button handler |
| Frontend | `src/index.html` | Add copy button to header |

---

### 12. 🌗 Light Theme Support

**Priority:** Low · **Effort:** Low  
**Why:** CSS custom property architecture already supports theming — just need a second set of values.

**Scope:**
- Add `:root.light` block in `theme.css` with light color values
- Toggle via settings (idea #5) or follow system preference via `prefers-color-scheme`
- Store preference in config

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Frontend | `src/styles/theme.css` | Add light theme variables |
| Frontend | `src/main.ts` | Apply theme class on load |
| Frontend | `src/index.html` | Remove hardcoded `class="dark"` |

---

### 13. 🖼️ Mini/Compact Mode (Floating Widget)

**Priority:** Low · **Effort:** Medium  
**Why:** A tiny always-on-top widget showing just the most critical gauge would be useful for power users.

**Scope:**
- Second window mode: ~120×40px frameless transparent widget
- Shows highest-utilization gauge as a mini progress bar
- Click to expand to full window
- Draggable, always-on-top, click-through-able (toggle)

**Files to touch:**
| Area | File | Change |
|------|------|--------|
| Backend | `src-tauri/tauri.conf.json` | Add second window config |
| Backend | `src-tauri/src/main.rs` | Manage mini window lifecycle |
| Frontend | `src/mini.html` | New — mini widget HTML |
| Frontend | `src/mini.ts` | New — mini widget logic |
| Frontend | `src/styles/mini.css` | New — compact styles |

---

## Implementation Priority Matrix

| # | Feature | Priority | Effort | Dependencies |
|---|---------|----------|--------|--------------|
| 1 | Threshold Notifications | 🔴 High | Medium | #5 (optional) |
| 3 | Live Cost Tracking | 🔴 High | Low | None |
| 4 | Dynamic Tray Icon | 🟡 Medium | Low | None |
| 6 | Global Hotkey | 🟡 Medium | Low | None |
| 10 | Window Anchor to Tray | 🟢 Low | Low | None |
| 11 | Copy to Clipboard | 🟢 Low | Low | None |
| 12 | Light Theme | 🟢 Low | Low | None |
| 5 | Settings Panel | 🟡 Medium | Medium | None |
| 2 | Usage History + Sparklines | 🔴 High | Medium-High | None |
| 9 | Test Infrastructure | 🟡 Medium | Medium | None |
| 8 | Auto-Update | 🟡 Medium | Medium | CI setup |
| 7 | Plugin Architecture | 🟡 Medium | High | None |
| 13 | Mini/Compact Mode | 🟢 Low | Medium | None |

### Suggested Implementation Order (Phases)

**Phase 1 — Quick Wins (1-2 days each):**
3 → 4 → 6 → 10 → 11 → 12

**Phase 2 — Core Features (3-5 days each):**
1 → 5 → 2

**Phase 3 — Infrastructure (3-5 days each):**
9 → 8

**Phase 4 — Ambitious (1-2 weeks each):**
7 → 13
