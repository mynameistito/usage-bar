#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod models;
mod credentials;
mod claude_service;
mod zai_service;
mod commands;

use cache::ResponseCache;
use models::{UsageData, ZaiUsageData};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, tray::TrayIconBuilder};

pub struct HttpClient(pub Arc<reqwest::Client>);
pub struct ClaudeUsageCache(pub ResponseCache<UsageData>);
pub struct ZaiUsageCache(pub ResponseCache<ZaiUsageData>);

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .setup(|app| {
      // Initialize shared HTTP client
      let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client");
      app.manage(HttpClient(Arc::new(client)));

      // Initialize response caches (30 second TTL)
      app.manage(ClaudeUsageCache(ResponseCache::new(30)));
      app.manage(ZaiUsageCache(ResponseCache::new(30)));

      // Get the window that was automatically created from tauri.conf.json
      if let Some(window) = app.get_webview_window("main") {
        window.set_ignore_cursor_events(false)?;
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
        .on_menu_event(move |app: &tauri::AppHandle, event| {
          match event.id.as_ref() {
            "open" => {
              if let Some(window) = app.get_webview_window("main") {
                window.show().unwrap();
                window.set_focus().unwrap();
              }
            }
            "quit" => {
              app.exit(0);
            }
            _ => {}
          }
        })
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_claude_usage,
      commands::get_claude_tier,
      commands::get_zai_usage,
      commands::get_zai_tier,
      commands::refresh_zai_usage,
      commands::check_zai_api_key,
      commands::validate_zai_api_key,
      commands::save_zai_api_key,
      commands::delete_zai_api_key,
      commands::refresh_all,
      commands::quit_app,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
