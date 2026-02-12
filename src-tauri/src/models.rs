use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub five_hour: UsagePeriod,
    pub seven_day: UsagePeriod,
    pub extra_usage: Option<ExtraUsageResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraUsageResponse {
    pub is_enabled: bool,
    pub monthly_limit: Option<f64>,
    pub used_credits: Option<f64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePeriod {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub five_hour_utilization: f64,
    pub five_hour_resets_at: Option<String>,
    pub seven_day_utilization: f64,
    pub seven_day_resets_at: Option<String>,
    pub extra_usage_enabled: bool,
    pub extra_usage_monthly_limit: Option<f64>,
    pub extra_usage_used_credits: Option<f64>,
    pub extra_usage_utilization: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiQuotaResponse {
    pub data: ZaiQuotaData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiQuotaData {
    pub limits: Vec<ZaiQuotaLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiQuotaLimit {
    #[serde(rename = "type")]
    pub limit_type: String,
    pub percentage: f64,
    #[serde(rename = "nextResetTime")]
    pub next_reset_time: Option<i64>,
    #[serde(rename = "currentValue")]
    pub current_value: Option<i32>,
    pub usage: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiUsageData {
    pub token_usage: Option<TokenUsage>,
    pub mcp_usage: Option<McpUsage>,
    pub tier_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiTierData {
    pub plan_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub percentage: f64,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpUsage {
    pub percentage: f64,
    pub used: i32,
    pub total: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeOAuthCredentials {
    #[serde(rename = "claudeAiOauth")]
    pub claude_ai_oauth: ClaudeOAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeOAuth {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResponse {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierResponse {
    #[serde(default)]
    pub rate_limit_tier: Option<String>,
    #[serde(default)]
    pub billing_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeTierData {
    pub plan_name: String,
    pub rate_limit_tier: String,
}
