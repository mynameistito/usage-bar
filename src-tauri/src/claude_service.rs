use crate::credentials::CredentialManager;
use crate::models::{ClaudeTierData, TierResponse, TokenRefreshResponse, UsageData, UsageResponse};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{debug_claude, debug_error, debug_net};

const OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const USAGE_API_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const TOKEN_REFRESH_URL: &str = "https://console.anthropic.com/v1/oauth/token";

pub struct ClaudeService;

impl ClaudeService {
    fn now_millis() -> Result<i64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .map_err(|e| anyhow!("System clock error: {}", e))
    }

    pub async fn fetch_usage(client: Arc<reqwest::Client>) -> Result<UsageData> {
        debug_claude!("fetch_usage: Starting request");
        debug_net!("GET {}", USAGE_API_URL);

        let token = CredentialManager::read_claude_access_token()?;
        debug_claude!("Using access token (expires_at: N/A)");

        let response = client
            .get(USAGE_API_URL)
            .header("Authorization", format!("Bearer {}", token))
            .header("anthropic-beta", "oauth-2025-04-20")
            .send()
            .await?;

        debug_net!("Response status: {}", response.status());

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                debug_claude!("Unauthorized: Attempting token refresh");
                Self::refresh_token(client.clone()).await?;
                let token = CredentialManager::read_claude_access_token()?;
                let retry_response = client
                    .get(USAGE_API_URL)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("anthropic-beta", "oauth-2025-04-20")
                    .send()
                    .await?;

                debug_net!("Retry response status: {}", retry_response.status());

                // Check retry response status before handling
                match retry_response.status() {
                    status if status.is_success() => {
                        debug_claude!("Successfully fetched usage data after retry");
                        Self::handle_response(retry_response).await
                    }
                    StatusCode::UNAUTHORIZED => {
                        debug_error!("Still unauthorized after token refresh");
                        Err(anyhow!("Authentication failed — please log in again"))
                    }
                    StatusCode::FORBIDDEN => {
                        debug_error!("Access denied after token refresh");
                        Err(anyhow!("Access denied — check your permissions"))
                    }
                    StatusCode::TOO_MANY_REQUESTS => {
                        debug_error!("Rate limited after token refresh");
                        Err(anyhow!("Rate limited — please wait and try again"))
                    }
                    status if status.is_server_error() => {
                        debug_error!("Server error after token refresh");
                        Err(anyhow!("Server error — try again later"))
                    }
                    _ => {
                        debug_error!("Failed to fetch usage data after token refresh");
                        Err(anyhow!("Failed to fetch usage data"))
                    }
                }
            }
            status if status.is_success() => {
                debug_claude!("Successfully fetched usage data");
                Self::handle_response(response).await
            }
            StatusCode::FORBIDDEN => {
                debug_error!("Access denied — check your permissions");
                Err(anyhow!("Access denied — check your permissions"))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                debug_error!("Rate limited — please wait and try again");
                Err(anyhow!("Rate limited — please wait and try again"))
            }
            status if status.is_server_error() => {
                debug_error!("Server error — try again later");
                Err(anyhow!("Server error — try again later"))
            }
            _ => {
                debug_error!("Failed to fetch usage data");
                Err(anyhow!("Failed to fetch usage data"))
            }
        }
    }

    async fn handle_response(response: reqwest::Response) -> Result<UsageData> {
        let response_text = response.text().await?;

        let usage_response: UsageResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse usage response: {}", e))?;

        let extra_usage = usage_response.extra_usage;

        Ok(UsageData {
            five_hour_utilization: usage_response.five_hour.utilization,
            five_hour_resets_at: Some(usage_response.five_hour.resets_at),
            seven_day_utilization: usage_response.seven_day.utilization,
            seven_day_resets_at: Some(usage_response.seven_day.resets_at),
            extra_usage_enabled: extra_usage.as_ref().map(|e| e.is_enabled).unwrap_or(false),
            extra_usage_monthly_limit: extra_usage.as_ref().and_then(|e| e.monthly_limit),
            extra_usage_used_credits: extra_usage.as_ref().and_then(|e| e.used_credits),
            extra_usage_utilization: extra_usage.as_ref().and_then(|e| e.utilization),
        })
    }

    pub async fn refresh_token(client: Arc<reqwest::Client>) -> Result<()> {
        debug_claude!("refresh_token: Starting token refresh");
        debug_net!("POST {}", TOKEN_REFRESH_URL);

        let credentials = CredentialManager::read_claude_credentials()?;

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &credentials.claude_ai_oauth.refresh_token),
            ("client_id", OAUTH_CLIENT_ID),
        ];

        let response = client
            .post(TOKEN_REFRESH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        debug_net!("Response status: {}", response.status());

        if !response.status().is_success() {
            let error_text = response.text().await?;
            debug_error!("Token refresh failed: {}", error_text);
            return Err(anyhow!("Token refresh failed: {}", error_text));
        }

        let refresh_response: TokenRefreshResponse = response.json().await?;

        let now = Self::now_millis()?;
        let expires_at = now + (refresh_response.expires_in * 1000);

        debug_claude!(
            "Token refresh successful (new expiry in {}ms)",
            refresh_response.expires_in * 1000
        );

        CredentialManager::update_claude_token(
            &refresh_response.access_token,
            &refresh_response.refresh_token,
            expires_at,
        )?;

        Ok(())
    }

    pub fn is_token_expired() -> bool {
        match CredentialManager::read_claude_credentials() {
            Ok(credentials) => {
                if let Some(expires_at) = credentials.claude_ai_oauth.expires_at {
                    match Self::now_millis() {
                        Ok(now) => {
                            let buffer: i64 = 60 * 1000; // 60 second buffer
                            let expired = now + buffer >= expires_at;
                            debug_claude!(
                                "Token expiry check: now={}, expires_at={}, expired={}",
                                now,
                                expires_at,
                                expired
                            );
                            expired
                        }
                        Err(_) => {
                            debug_error!("System clock error, treating token as expired");
                            true
                        }
                    }
                } else {
                    debug_claude!("Token has no expiry date, treating as expired");
                    true
                }
            }
            Err(_) => {
                debug_claude!("Failed to read credentials, treating as expired");
                true
            }
        }
    }

    pub async fn check_and_refresh_if_needed(client: Arc<reqwest::Client>) -> Result<()> {
        if Self::is_token_expired() {
            debug_claude!("Token expired or expiring soon, refreshing");
            Self::refresh_token(client).await?;
        } else {
            debug_claude!("Token is still valid, skipping refresh");
        }
        Ok(())
    }

    pub async fn fetch_tier(client: Arc<reqwest::Client>) -> Result<ClaudeTierData> {
        debug_claude!("fetch_tier: Starting request");
        debug_net!("GET {}", USAGE_API_URL);

        let token = CredentialManager::read_claude_access_token()?;

        let response = client
            .get(USAGE_API_URL)
            .header("Authorization", format!("Bearer {}", token))
            .header("anthropic-beta", "oauth-2025-04-20")
            .send()
            .await?;

        debug_net!("Response status: {}", response.status());

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                debug_claude!("Unauthorized: Attempting token refresh");
                Self::refresh_token(client.clone()).await?;
                let token = CredentialManager::read_claude_access_token()?;
                let retry_response = client
                    .get(USAGE_API_URL)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("anthropic-beta", "oauth-2025-04-20")
                    .send()
                    .await?;

                debug_net!("Retry response status: {}", retry_response.status());

                // Check retry response status before handling
                match retry_response.status() {
                    status if status.is_success() => {
                        debug_claude!("Successfully fetched tier data after retry");
                        Self::handle_tier_response(retry_response).await
                    }
                    StatusCode::UNAUTHORIZED => {
                        debug_error!("Still unauthorized after token refresh");
                        Err(anyhow!("Authentication failed — please log in again"))
                    }
                    StatusCode::FORBIDDEN => {
                        debug_error!("Access denied after token refresh");
                        Err(anyhow!("Access denied — check your permissions"))
                    }
                    StatusCode::TOO_MANY_REQUESTS => {
                        debug_error!("Rate limited after token refresh");
                        Err(anyhow!("Rate limited — please wait and try again"))
                    }
                    status if status.is_server_error() => {
                        debug_error!("Server error after token refresh");
                        Err(anyhow!("Server error — try again later"))
                    }
                    _ => {
                        debug_error!("Failed to fetch tier data after token refresh");
                        Err(anyhow!("Failed to fetch tier data"))
                    }
                }
            }
            status if status.is_success() => {
                debug_claude!("Successfully fetched tier data");
                Self::handle_tier_response(response).await
            }
            StatusCode::FORBIDDEN => {
                debug_error!("Access denied — check your permissions");
                Err(anyhow!("Access denied — check your permissions"))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                debug_error!("Rate limited — please wait and try again");
                Err(anyhow!("Rate limited — please wait and try again"))
            }
            status if status.is_server_error() => {
                debug_error!("Server error — try again later");
                Err(anyhow!("Server error — try again later"))
            }
            _ => {
                debug_error!("Failed to fetch tier data");
                Err(anyhow!("Failed to fetch tier data"))
            }
        }
    }

    async fn handle_tier_response(response: reqwest::Response) -> Result<ClaudeTierData> {
        let response_text = response.text().await?;

        let tier_response: TierResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse tier response: {}", e))?;

        let plan_name =
            Self::infer_plan_name(&tier_response.rate_limit_tier, &tier_response.billing_type);
        let raw_tier = tier_response.rate_limit_tier.unwrap_or_default();

        Ok(ClaudeTierData {
            plan_name,
            rate_limit_tier: raw_tier,
        })
    }

    fn infer_plan_name(rate_limit_tier: &Option<String>, billing_type: &Option<String>) -> String {
        let tier = rate_limit_tier.as_deref().unwrap_or("").to_lowercase();
        let billing = billing_type.as_deref().unwrap_or("").to_lowercase();

        if tier.contains("max") {
            "Max".into()
        } else if tier.contains("pro") || billing.contains("stripe") {
            "Pro".into()
        } else if tier.contains("team") {
            "Team".into()
        } else if tier.contains("enterprise") {
            "Enterprise".into()
        } else {
            "Free".into()
        }
    }
}
