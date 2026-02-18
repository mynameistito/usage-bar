use crate::credentials::CredentialManager;
use crate::models::{
    ClaudeOAuthCredentials, ClaudeTierData, TokenRefreshResponse, UsageData, UsageResponse,
};
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

    async fn handle_combined_response(
        response: reqwest::Response,
        credentials: ClaudeOAuthCredentials,
    ) -> Result<(UsageData, ClaudeTierData)> {
        let response_text = response.text().await?;

        let usage_response: UsageResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse usage response: {}", e))?;

        let extra_usage = usage_response.extra_usage.as_ref();

        let usage_data = UsageData {
            five_hour_utilization: usage_response
                .five_hour
                .as_ref()
                .map(|p| p.utilization)
                .unwrap_or(0.0),
            five_hour_resets_at: usage_response
                .five_hour
                .as_ref()
                .and_then(|p| p.resets_at.clone()),
            seven_day_utilization: usage_response
                .seven_day
                .as_ref()
                .map(|p| p.utilization)
                .unwrap_or(0.0),
            seven_day_resets_at: usage_response
                .seven_day
                .as_ref()
                .and_then(|p| p.resets_at.clone()),
            extra_usage_enabled: extra_usage.map(|e| e.is_enabled).unwrap_or(false),
            extra_usage_monthly_limit: extra_usage.and_then(|e| e.monthly_limit),
            extra_usage_used_credits: extra_usage.and_then(|e| e.used_credits),
            extra_usage_utilization: extra_usage.and_then(|e| e.utilization),
        };

        // Extract tier info from credentials, falling back to API response for older credential files
        let subscription_type = credentials
            .claude_ai_oauth
            .subscription_type
            .clone()
            .unwrap_or_default();
        let plan_name = if subscription_type.is_empty() {
            // For legacy credential files, infer plan from rate_limit_tier patterns
            // NOTE: These tier mappings are speculative based on observed patterns:
            // - "tier_2"/"tier_3" → "Pro" (assumed)
            // - "tier_4"/"tier_5" → "Team" (assumed)
            // - "tier_1_5x"/"tier_free_5x" etc. (actual API values) not yet mapped
            // Fallback to billing type detection for reliability
            Self::infer_plan_name_from_usage_response(&usage_response)
        } else {
            Self::infer_plan_name_from_subscription(&subscription_type)
        };
        let raw_tier = credentials
            .claude_ai_oauth
            .rate_limit_tier
            .clone()
            .unwrap_or_else(|| usage_response.rate_limit_tier.clone().unwrap_or_default());

        let tier_data = ClaudeTierData {
            plan_name,
            rate_limit_tier: raw_tier,
        };

        Ok((usage_data, tier_data))
    }

    /// Fetches both usage and tier data from a single API call.
    /// This is more efficient than calling fetch_usage and fetch_tier separately
    /// since they both hit the same endpoint.
    pub async fn claude_fetch_usage_and_tier(
        client: Arc<reqwest::Client>,
    ) -> Result<(UsageData, ClaudeTierData)> {
        debug_claude!("claude_fetch_usage_and_tier: Starting request");
        debug_net!("GET {}", USAGE_API_URL);

        let credentials = CredentialManager::claude_read_credentials()?;
        let token = credentials.claude_ai_oauth.access_token.clone();
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
                let refreshed_creds = CredentialManager::claude_read_credentials()?;
                let token = refreshed_creds.claude_ai_oauth.access_token.clone();
                let retry_response = client
                    .get(USAGE_API_URL)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("anthropic-beta", "oauth-2025-04-20")
                    .send()
                    .await?;

                debug_net!("Retry response status: {}", retry_response.status());

                match retry_response.status() {
                    status if status.is_success() => {
                        debug_claude!("Successfully fetched usage+tier data after retry");
                        Self::handle_combined_response(retry_response, refreshed_creds).await
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
                        debug_error!("Failed to fetch usage+tier data after token refresh");
                        Err(anyhow!("Failed to fetch usage data"))
                    }
                }
            }
            status if status.is_success() => {
                debug_claude!("Successfully fetched usage+tier data");
                Self::handle_combined_response(response, credentials).await
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
                debug_error!("Failed to fetch usage+tier data");
                Err(anyhow!("Failed to fetch usage data"))
            }
        }
    }

    pub async fn refresh_token(client: Arc<reqwest::Client>) -> Result<()> {
        debug_claude!("refresh_token: Starting token refresh");
        debug_net!("POST {}", TOKEN_REFRESH_URL);

        let credentials = CredentialManager::claude_read_credentials()?;

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

        CredentialManager::claude_update_token(
            &refresh_response.access_token,
            &refresh_response.refresh_token,
            expires_at,
        )?;

        Ok(())
    }

    pub fn is_token_expired() -> bool {
        match CredentialManager::claude_read_credentials() {
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

    fn infer_plan_name_from_usage_response(response: &UsageResponse) -> String {
        let tier = response
            .rate_limit_tier
            .as_ref()
            .map(|t| t.to_lowercase())
            .unwrap_or_default();
        let billing = response
            .billing_type
            .as_ref()
            .map(|b| b.to_lowercase())
            .unwrap_or_default();

        if tier.contains("max") || tier.contains("tier_2_5x") || tier.contains("tier_3_5x") {
            "Max".into()
        } else if tier.contains("team") || tier.contains("tier_4") || tier.contains("tier_5") {
            // tier_4/tier_5 assumed to map to Team; revisit if Anthropic introduces new tier names
            "Team".into()
        } else if (tier.contains("tier_2") && !tier.contains("_1") && !tier.contains("_3"))
            || tier.contains("tier_3")
        {
            "Pro".into()
        } else if billing.contains("stripe") {
            // Stripe-billed user with unrecognized tier: assume at least Pro
            "Pro".into()
        } else {
            "Free".into()
        }
    }

    fn infer_plan_name_from_subscription(subscription_type: &str) -> String {
        let subtype_lower = subscription_type.to_lowercase();

        if subtype_lower.contains("max") {
            "Max".into()
        } else if subtype_lower.contains("pro") {
            "Pro".into()
        } else if subtype_lower.contains("team") {
            "Team".into()
        } else if subtype_lower.contains("enterprise") {
            "Enterprise".into()
        } else {
            "Free".into()
        }
    }
}
