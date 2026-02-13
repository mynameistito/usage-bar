use crate::credentials::CredentialManager;
use crate::models::{McpUsage, TokenUsage, ZaiQuotaResponse, ZaiUsageData};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::{debug_error, debug_net, debug_zai};

const ZAI_API_URL: &str = "https://api.z.ai/api/monitor/usage/quota/limit";

pub struct ZaiService;

impl ZaiService {
    pub async fn zai_fetch_quota(client: Arc<reqwest::Client>) -> Result<ZaiUsageData> {
        debug_zai!("zai_fetch_quota: Starting request");
        debug_net!("GET {}", ZAI_API_URL);

        let api_key = CredentialManager::zai_read_api_key()?;
        debug_zai!("Using API key: ***REDACTED***");

        let response = client
            .get(ZAI_API_URL)
            .header("Authorization", &api_key)
            .header("Accept-Language", "en-US,en")
            .header("Content-Type", "application/json")
            .send()
            .await?;

        debug_net!("Response status: {}", response.status());

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                debug_error!("Invalid Z.ai API key");
                Err(anyhow!("z.ai: Invalid API key — please reconfigure"))
            }
            StatusCode::FORBIDDEN => {
                debug_error!("Access denied to Z.ai API");
                Err(anyhow!("z.ai: Access denied"))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                debug_error!("Z.ai rate limit exceeded");
                Err(anyhow!("z.ai: Rate limited — please wait"))
            }
            status if status.is_success() => {
                debug_zai!("Successfully fetched quota data");
                Self::handle_response(response).await
            }
            status if status.is_server_error() => {
                debug_error!("Z.ai server error");
                Err(anyhow!("z.ai: Server error — try again later"))
            }
            _ => {
                debug_error!("Failed to fetch Z.ai quota data");
                Err(anyhow!("z.ai: Failed to fetch usage data"))
            }
        }
    }

    async fn handle_response(response: reqwest::Response) -> Result<ZaiUsageData> {
        let response_text = response.text().await?;

        let quota_response: ZaiQuotaResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse quota response: {}", e))?;

        let mut token_usage: Option<TokenUsage> = None;
        let mut mcp_usage: Option<McpUsage> = None;
        let mut time_limit_total: Option<i32> = None;

        for limit in quota_response.data.limits {
            match limit.limit_type.as_str() {
                "TOKENS_LIMIT" => {
                    token_usage = Some(TokenUsage {
                        percentage: limit.percentage,
                        resets_at: limit.next_reset_time,
                    });
                }
                "TIME_LIMIT" => {
                    time_limit_total = limit.usage;
                    mcp_usage = Some(McpUsage {
                        percentage: limit.percentage,
                        used: limit.current_value.unwrap_or(0),
                        total: limit.usage.unwrap_or(0),
                    });
                }
                _ => {}
            }
        }

        // Infer tier from TIME_LIMIT total (based on Z.ai FAQ):
        // Lite: ~80 prompts per 5 hours
        // Pro: ~400 prompts per 5 hours
        // Max: ~1600 prompts per 5 hours
        let tier_name = time_limit_total.and_then(|total| {
            let tier = if total >= 1400 {
                Some("Max".to_string())
            } else if total >= 300 {
                Some("Pro".to_string())
            } else if total > 0 {
                Some("Lite".to_string())
            } else {
                None
            };
            debug_zai!("Inferred tier: {:?} from time_limit_total={}", tier, total);
            tier
        });

        Ok(ZaiUsageData {
            token_usage,
            mcp_usage,
            tier_name,
        })
    }

    pub fn zai_has_api_key() -> bool {
        CredentialManager::zai_has_api_key()
    }

    pub async fn validate_api_key(client: Arc<reqwest::Client>, api_key: &str) -> Result<()> {
        debug_zai!("validate_api_key: Starting validation");
        let api_key = api_key.trim();

        if api_key.is_empty() {
            debug_error!("API key cannot be empty");
            return Err(anyhow!("API key cannot be empty"));
        }

        // Resolve environment variable if using {env:varname} syntax
        let api_key = CredentialManager::resolve_env_reference(api_key)?;

        if api_key.len() < 10 {
            debug_error!("API key is too short");
            return Err(anyhow!("API key is too short"));
        }

        debug_net!("GET {} (validating key)", ZAI_API_URL);

        let response = client
            .get(ZAI_API_URL)
            .header("Authorization", &api_key)
            .header("Accept-Language", "en-US,en")
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| {
                debug_error!("Network error during validation: {}", e);
                if e.is_timeout() {
                    anyhow!("Connection timed out - check your network")
                } else if e.is_connect() {
                    anyhow!("Could not connect to Z.AI - check your network")
                } else {
                    anyhow!("Network error: {}", e)
                }
            })?;

        debug_net!("Validation response status: {}", response.status());

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                debug_error!("Invalid API key (401)");
                Err(anyhow!("Invalid API key"))
            }
            StatusCode::FORBIDDEN => {
                debug_error!("Access denied - key may lack permissions (403)");
                Err(anyhow!("Access denied - key may lack permissions"))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                debug_error!("Rate limited during validation (429)");
                Err(anyhow!("Rate limited - try again later"))
            }
            status if status.is_server_error() => {
                debug_error!("Z.AI server error (5xx)");
                Err(anyhow!("Z.AI server error - try again later"))
            }
            status if status.is_success() => {
                debug_zai!("API key validation successful");
                let body = response
                    .text()
                    .await
                    .map_err(|e| anyhow!("Failed to read response: {}", e))?;

                if body.contains("\"error\"") {
                    return Err(anyhow!("Invalid API key"));
                }

                if !body.contains("\"limits\"") && !body.contains("\"data\"") {
                    return Err(anyhow!("Unexpected response - key may be invalid"));
                }

                Ok(())
            }
            _ => Err(anyhow!(
                "Failed to validate API key (HTTP {})",
                response.status()
            )),
        }
    }
}
