#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod amp_service;
mod cache;
mod claude_service;
mod commands;
mod credentials;
mod logging;
mod models;
mod zai_service;

// Re-export logging constants so macros can find them via $crate
pub use logging::{
    COLOR_BLUE, COLOR_BRIGHT_CYAN, COLOR_BRIGHT_RED, COLOR_CYAN, COLOR_GRAY, COLOR_GREEN,
    COLOR_MAGENTA, COLOR_RED, COLOR_RESET, COLOR_YELLOW,
};

use cache::ResponseCache;
use models::{AmpUsageData, ClaudeTierData, UsageData, ZaiTierData, ZaiUsageData};
use std::sync::Arc;
use std::time::Duration;
use tauri::{tray::TrayIconBuilder, Manager};

pub struct HttpClient(pub Arc<reqwest::Client>);
pub struct AmpHttpClient(pub Arc<reqwest::Client>);
pub struct ClaudeUsageCache(pub ResponseCache<UsageData>);
pub struct ClaudeTierCache(pub ResponseCache<ClaudeTierData>);
pub struct ZaiUsageCache(pub ResponseCache<ZaiUsageData>);
pub struct ZaiTierCache(pub ResponseCache<ZaiTierData>);
pub struct AmpUsageCache(pub ResponseCache<AmpUsageData>);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    debug_app!("Usage Bar starting...");

    tauri::Builder::default()
        .setup(|app| {
            debug_app!("Initializing application state");

            // Initialize shared HTTP client (with redirects)
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;
            app.manage(HttpClient(Arc::new(client)));
            debug_app!("HTTP client initialized (timeout: 15s, redirects enabled)");

            // Initialize Amp HTTP client (no redirects for auth detection)
            let amp_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build Amp HTTP client: {}", e))?;
            app.manage(AmpHttpClient(Arc::new(amp_client)));
            debug_app!("Amp HTTP client initialized (timeout: 15s, redirects disabled)");

            // Initialize response caches (30 second TTL)
            app.manage(ClaudeUsageCache(ResponseCache::new(30)));
            app.manage(ClaudeTierCache(ResponseCache::new(30)));
            app.manage(ZaiUsageCache(ResponseCache::new(30)));
            app.manage(ZaiTierCache(ResponseCache::new(30)));
            app.manage(AmpUsageCache(ResponseCache::new(30)));
            debug_app!("Response caches initialized (TTL: 30s)");

            // Get the window that was automatically created from tauri.conf.json
            if let Some(window) = app.get_webview_window("main") {
                window.set_ignore_cursor_events(false)?;

                // Handle window close event for graceful shutdown
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        debug_app!("Window close requested, exiting gracefully");
                        if let Err(e) = window_clone.hide() {
                            debug_error!("Failed to hide window: {}", e);
                        }
                    }
                });

                debug_app!("Main window configured");
            }

            // Create tray icon with menu
            let _tray = TrayIconBuilder::new()
                .menu(&tauri::menu::Menu::with_items(
                    app,
                    &[
                        &tauri::menu::MenuItem::with_id(app, "open", "Open", true, None::<&str>)?,
                        &tauri::menu::PredefinedMenuItem::separator(app)?,
                        &tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                    ],
                )?)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if let Err(e) = window.show() {
                                debug_error!("Failed to show window: {}", e);
                            }
                            if let Err(e) = window.set_focus() {
                                debug_error!("Failed to focus window: {}", e);
                            }
                        }
                    }
                    "quit" => {
                        debug_app!("Quit requested via tray menu");
                        app.exit(0);
                    }
                    _ => {}
                })
                .icon(match app.default_window_icon() {
                    Some(icon) => icon.clone(),
                    None => return Err(anyhow::anyhow!("Missing window icon").into()),
                })
                .build(app)?;

            debug_app!("System tray icon registered");
            debug_app!("Initialization complete");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::claude_get_all,
            commands::claude_get_usage,
            commands::claude_get_tier,
            commands::zai_get_all,
            commands::zai_refresh_all,
            commands::zai_get_usage,
            commands::zai_get_tier,
            commands::zai_refresh_usage,
            commands::zai_check_api_key,
            commands::zai_validate_api_key,
            commands::zai_save_api_key,
            commands::zai_delete_api_key,
            commands::amp_get_usage,
            commands::amp_refresh_usage,
            commands::amp_check_session_cookie,
            commands::amp_validate_session_cookie,
            commands::amp_save_session_cookie,
            commands::amp_delete_session_cookie,
            commands::quit_app,
            commands::refresh_all,
            commands::open_url,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            eprintln!("error while running tauri application: {}", e);
            anyhow::anyhow!("Failed to run application")
        })?;
    Ok(())
}
