use crate::claude_service::ClaudeService;
use crate::credentials::CredentialManager;
use crate::zai_service::ZaiService;
use crate::{ClaudeTierCache, ClaudeUsageCache, HttpClient, ZaiTierCache, ZaiUsageCache};
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::{debug_cache, debug_claude, debug_cred, debug_zai};

#[tauri::command]
pub async fn claude_get_all(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ClaudeUsageCache>,
    tier_cache: State<'_, ClaudeTierCache>,
) -> Result<(crate::models::UsageData, crate::models::ClaudeTierData), String> {
    debug_claude!("claude_get_all called");

    let client = Arc::clone(&client.0);

    if let (Some(usage), Some(tier)) = (usage_cache.0.get(), tier_cache.0.get()) {
        debug_cache!("Returning cached Claude usage and tier data");
        return Ok((usage, tier));
    }

    debug_claude!("Calling check_and_refresh_if_needed...");
    if let Err(e) = ClaudeService::check_and_refresh_if_needed(client.clone()).await {
        debug_claude!("check_and_refresh_if_needed failed: {}", e);
        return Err(e.to_string());
    }
    debug_claude!("check_and_refresh_if_needed succeeded");

    debug_claude!("Calling fetch_usage_and_tier...");
    match ClaudeService::fetch_usage_and_tier(client).await {
        Ok((usage_data, tier_data)) => {
            debug_claude!("fetch_usage_and_tier succeeded, caching results");
            usage_cache.0.set(usage_data.clone());
            tier_cache.0.set(tier_data.clone());
            Ok((usage_data, tier_data))
        }
        Err(e) => {
            debug_claude!("fetch_usage_and_tier failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn claude_get_usage(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ClaudeUsageCache>,
    tier_cache: State<'_, ClaudeTierCache>,
) -> Result<crate::models::UsageData, String> {
    debug_claude!("claude_get_usage called");

    // Check cache first
    if let Some(data) = usage_cache.0.get() {
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

    debug_claude!("Calling fetch_usage_and_tier...");
    match ClaudeService::fetch_usage_and_tier(client).await {
        Ok((usage_data, tier_data)) => {
            debug_claude!("fetch_usage_and_tier succeeded, caching results");
            usage_cache.0.set(usage_data.clone());
            tier_cache.0.set(tier_data);
            Ok(usage_data)
        }
        Err(e) => {
            debug_claude!("fetch_usage_and_tier failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn claude_get_tier(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ClaudeUsageCache>,
    tier_cache: State<'_, ClaudeTierCache>,
) -> Result<crate::models::ClaudeTierData, String> {
    debug_claude!("claude_get_tier called");

    // Check tier cache first
    if let Some(data) = tier_cache.0.get() {
        debug_cache!("Returning cached Claude tier data");
        return Ok(data);
    }

    let client = Arc::clone(&client.0);

    debug_claude!("Calling check_and_refresh_if_needed for tier...");
    if let Err(e) = ClaudeService::check_and_refresh_if_needed(client.clone()).await {
        debug_claude!("check_and_refresh_if_needed failed: {}", e);
        return Err(e.to_string());
    }
    debug_claude!("check_and_refresh_if_needed succeeded");

    debug_claude!("Calling fetch_usage_and_tier for tier...");
    match ClaudeService::fetch_usage_and_tier(client).await {
        Ok((usage_data, tier_data)) => {
            debug_claude!(
                "fetch_usage_and_tier succeeded: plan={}",
                tier_data.plan_name
            );
            // Cache both results to avoid duplicate fetches
            usage_cache.0.set(usage_data);
            tier_cache.0.set(tier_data.clone());
            Ok(tier_data)
        }
        Err(e) => {
            debug_claude!("fetch_usage_and_tier failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn zai_get_all(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ZaiUsageCache>,
    tier_cache: State<'_, ZaiTierCache>,
) -> Result<(crate::models::ZaiUsageData, crate::models::ZaiTierData), String> {
    debug_zai!("zai_get_all called");

    let client = Arc::clone(&client.0);

    if !ZaiService::zai_has_api_key() {
        debug_zai!("Z.ai API key not configured");
        return Err("Z.ai API key not configured".to_string());
    }

    if let (Some(usage), Some(tier)) = (usage_cache.0.get(), tier_cache.0.get()) {
        debug_cache!("Returning cached Z.ai usage and tier data");
        return Ok((usage, tier));
    }

    debug_zai!("Calling ZaiService::fetch_quota...");
    match ZaiService::fetch_quota(client).await {
        Ok(data) => {
            debug_zai!("fetch_quota succeeded, caching result");
            let tier_name = data
                .tier_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            let tier_data = crate::models::ZaiTierData {
                plan_name: tier_name.clone(),
            };
            usage_cache.0.set(data.clone());
            tier_cache.0.set(tier_data.clone());
            Ok((data, tier_data))
        }
        Err(e) => {
            debug_zai!("fetch_quota failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn zai_get_usage(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ZaiUsageCache>,
    tier_cache: State<'_, ZaiTierCache>,
) -> Result<crate::models::ZaiUsageData, String> {
    debug_zai!("zai_get_usage called");

    // Check cache first
    if let Some(data) = usage_cache.0.get() {
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
            // Also populate tier cache from the usage response
            if let Some(tier_name) = &data.tier_name {
                tier_cache.0.set(crate::models::ZaiTierData {
                    plan_name: tier_name.clone(),
                });
            }
            usage_cache.0.set(data.clone());
            Ok(data)
        }
        Err(e) => {
            debug_zai!("fetch_quota failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn zai_refresh_usage(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ZaiUsageCache>,
    tier_cache: State<'_, ZaiTierCache>,
) -> Result<crate::models::ZaiUsageData, String> {
    debug_zai!("zai_refresh_usage called (force refresh)");

    // Clear caches to force a fresh fetch
    usage_cache.0.clear();
    tier_cache.0.clear();

    let client = Arc::clone(&client.0);

    if !ZaiService::zai_has_api_key() {
        debug_zai!("Z.ai API key not configured");
        return Err("Z.ai API key not configured".to_string());
    }

    debug_zai!("Calling ZaiService::fetch_quota...");
    match ZaiService::fetch_quota(client).await {
        Ok(data) => {
            debug_zai!("fetch_quota succeeded, caching result");
            // Also populate tier cache from the usage response
            if let Some(tier_name) = &data.tier_name {
                tier_cache.0.set(crate::models::ZaiTierData {
                    plan_name: tier_name.clone(),
                });
            }
            usage_cache.0.set(data.clone());
            Ok(data)
        }
        Err(e) => {
            debug_zai!("fetch_quota failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn zai_get_tier(
    client: State<'_, HttpClient>,
    usage_cache: State<'_, ZaiUsageCache>,
    tier_cache: State<'_, ZaiTierCache>,
) -> Result<crate::models::ZaiTierData, String> {
    debug_zai!("zai_get_tier called");

    // Check tier cache first
    if let Some(data) = tier_cache.0.get() {
        debug_cache!("Returning cached Z.ai tier data");
        return Ok(data);
    }

    if !ZaiService::zai_has_api_key() {
        debug_zai!("Z.ai API key not configured");
        return Err("Z.ai API key not configured".to_string());
    }

    let client = Arc::clone(&client.0);

    debug_zai!("Calling ZaiService::fetch_quota for tier...");
    match ZaiService::fetch_quota(client).await {
        Ok(data) => {
            let plan_name = data
                .tier_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            debug_zai!("fetch_quota succeeded: plan={}", plan_name);
            // Cache both results to avoid duplicate fetches
            usage_cache.0.set(data);
            let tier_data = crate::models::ZaiTierData { plan_name };
            tier_cache.0.set(tier_data.clone());
            Ok(tier_data)
        }
        Err(e) => {
            debug_zai!("fetch_quota failed: {}", e);
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
pub async fn zai_validate_api_key(
    client: State<'_, HttpClient>,
    api_key: String,
) -> Result<(), String> {
    debug_zai!("zai_validate_api_key called");
    let client = Arc::clone(&client.0);
    ZaiService::validate_api_key(client, &api_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn zai_save_api_key(api_key: String) -> Result<(), String> {
    CredentialManager::zai_write_api_key(&api_key).map_err(|e| e.to_string())
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
    claude_usage_cache: State<'_, ClaudeUsageCache>,
    claude_tier_cache: State<'_, ClaudeTierCache>,
    zai_usage_cache: State<'_, ZaiUsageCache>,
    zai_tier_cache: State<'_, ZaiTierCache>,
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
            match ClaudeService::fetch_usage_and_tier(client.clone()).await {
                Ok((usage_data, tier_data)) => {
                    claude_usage_cache.0.set(usage_data.clone());
                    claude_tier_cache.0.set(tier_data);
                    Ok(Some(usage_data))
                }
                Err(e) => Err(e.to_string()),
            }
        },
        async {
            if ZaiService::zai_has_api_key() {
                match ZaiService::fetch_quota(client.clone()).await {
                    Ok(data) => {
                        if let Some(tier_name) = &data.tier_name {
                            zai_tier_cache.0.set(crate::models::ZaiTierData {
                                plan_name: tier_name.clone(),
                            });
                        }
                        zai_usage_cache.0.set(data.clone());
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
