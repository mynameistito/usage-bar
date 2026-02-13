use crate::claude_service::ClaudeService;
use crate::credentials::CredentialManager;
use crate::zai_service::ZaiService;
use crate::{ClaudeUsageCache, HttpClient, ZaiUsageCache};
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::{debug_cache, debug_claude, debug_cred, debug_zai};

#[tauri::command]
pub async fn claude_get_usage(
    client: State<'_, HttpClient>,
    cache: State<'_, ClaudeUsageCache>,
) -> Result<crate::models::UsageData, String> {
    debug_claude!("claude_get_usage called");

    // Check cache first
    if let Some(data) = cache.0.get() {
        debug_cache!("Returning cached Claude usage data");
        return Ok(data);
    }

    let client = Arc::clone(&client.0);

    debug_claude!("Calling check_and_refresh_if_needed...");
    if let Err(e) = ClaudeService::check_and_refresh_if_needed(client.clone()).await {
        debug_claude!("check_and_refresh_if_needed failed: {}", e);
        return Err(e.to_string());
    }
    debug_claude!("check_and_refresh_if_needed succeeded");

    debug_claude!("Calling fetch_usage...");
    match ClaudeService::fetch_usage(client).await {
        Ok(data) => {
            debug_claude!("fetch_usage succeeded, caching result");
            cache.0.set(data.clone());
            Ok(data)
        }
        Err(e) => {
            debug_claude!("fetch_usage failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn claude_get_tier(
    client: State<'_, HttpClient>,
) -> Result<crate::models::ClaudeTierData, String> {
    debug_claude!("claude_get_tier called");
    let client = Arc::clone(&client.0);

    debug_claude!("Calling check_and_refresh_if_needed for tier...");
    if let Err(e) = ClaudeService::check_and_refresh_if_needed(client.clone()).await {
        debug_claude!("check_and_refresh_if_needed failed: {}", e);
        return Err(e.to_string());
    }
    debug_claude!("check_and_refresh_if_needed succeeded");

    debug_claude!("Calling fetch_tier...");
    match ClaudeService::fetch_tier(client).await {
        Ok(data) => {
            debug_claude!("fetch_tier succeeded: plan={}", data.plan_name);
            Ok(data)
        }
        Err(e) => {
            debug_claude!("fetch_tier failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn zai_get_usage(
    client: State<'_, HttpClient>,
    cache: State<'_, ZaiUsageCache>,
) -> Result<crate::models::ZaiUsageData, String> {
    debug_zai!("zai_get_usage called");

    // Check cache first
    if let Some(data) = cache.0.get() {
        debug_cache!("Returning cached Z.ai usage data");
        return Ok(data);
    }

    let client = Arc::clone(&client.0);

    if !ZaiService::zai_has_api_key() {
        debug_zai!("Z.ai API key not configured");
        return Err("Z.ai API key not configured".to_string());
    }

    debug_zai!("Calling ZaiService::fetch_quota...");
    match ZaiService::fetch_quota(client).await {
        Ok(data) => {
            debug_zai!("fetch_quota succeeded, caching result");
            cache.0.set(data.clone());
            Ok(data)
        }
        Err(e) => {
            debug_zai!("fetch_quota failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn refresh_zai_usage(
    client: State<'_, HttpClient>,
    cache: State<'_, ZaiUsageCache>,
) -> Result<crate::models::ZaiUsageData, String> {
    debug_zai!("refresh_zai_usage called (force refresh)");

    // Clear cache to force a fresh fetch
    cache.0.clear();

    let client = Arc::clone(&client.0);

    if !ZaiService::zai_has_api_key() {
        debug_zai!("Z.ai API key not configured");
        return Err("Z.ai API key not configured".to_string());
    }

    debug_zai!("Calling ZaiService::fetch_quota...");
    match ZaiService::fetch_quota(client).await {
        Ok(data) => {
            debug_zai!("fetch_quota succeeded, caching result");
            cache.0.set(data.clone());
            Ok(data)
        }
        Err(e) => {
            debug_zai!("fetch_quota failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_zai_tier(
    client: State<'_, HttpClient>,
) -> Result<crate::models::ZaiTierData, String> {
    debug_zai!("get_zai_tier called");

    if !ZaiService::zai_has_api_key() {
        debug_zai!("Z.ai API key not configured");
        return Err("Z.ai API key not configured".to_string());
    }

    let client = Arc::clone(&client.0);

    debug_zai!("Calling ZaiService::fetch_tier...");
    match ZaiService::fetch_tier(client).await {
        Ok(data) => {
            debug_zai!("fetch_tier succeeded: plan={}", data.plan_name);
            Ok(data)
        }
        Err(e) => {
            debug_zai!("fetch_tier failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn zai_check_api_key() -> bool {
    debug_cred!("zai_check_api_key called");
    let has_key = ZaiService::zai_has_api_key();
    debug_cred!("has_api_key: {}", has_key);
    has_key
}

#[tauri::command]
pub async fn validate_zai_api_key(
    client: State<'_, HttpClient>,
    api_key: String,
) -> Result<(), String> {
    debug_zai!("validate_zai_api_key called");
    let client = Arc::clone(&client.0);
    ZaiService::validate_api_key(client, &api_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn zai_save_api_key(api_key: String) -> Result<(), String> {
    CredentialManager::write_zai_api_key(&api_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn zai_delete_api_key() -> Result<(), String> {
    CredentialManager::zai_delete_api_key().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn refresh_all(
    _app: AppHandle,
    client: State<'_, HttpClient>,
    claude_cache: State<'_, ClaudeUsageCache>,
    zai_cache: State<'_, ZaiUsageCache>,
) -> Result<
    (
        Option<crate::models::UsageData>,
        Option<crate::models::ZaiUsageData>,
    ),
    String,
> {
    let client = Arc::clone(&client.0);

    // Fetch both APIs in parallel using tokio::join!
    let (claude_result, zai_result) = tokio::join!(
        async {
            if let Err(e) = ClaudeService::check_and_refresh_if_needed(client.clone()).await {
                return Err(e.to_string());
            }
            match ClaudeService::fetch_usage(client.clone()).await {
                Ok(data) => {
                    claude_cache.0.set(data.clone());
                    Ok(Some(data))
                }
                Err(e) => Err(e.to_string()),
            }
        },
        async {
            if ZaiService::zai_has_api_key() {
                match ZaiService::fetch_quota(client.clone()).await {
                    Ok(data) => {
                        zai_cache.0.set(data.clone());
                        Ok(Some(data))
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Ok(None)
            }
        }
    );

    Ok((claude_result?, zai_result?))
}
