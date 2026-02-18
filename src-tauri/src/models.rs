use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub five_hour: Option<UsagePeriod>,
    pub seven_day: Option<UsagePeriod>,
    pub extra_usage: Option<ExtraUsageResponse>,
    // Tier info also comes from the same /usage endpoint
    #[serde(default)]
    pub rate_limit_tier: Option<String>,
    #[serde(default)]
    pub billing_type: Option<String>,
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
    #[serde(default)]
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
    #[serde(deserialize_with = "deserialize_expires_at")]
    pub expires_at: Option<i64>,
    #[serde(rename = "subscriptionType", default)]
    pub subscription_type: Option<String>,
    #[serde(rename = "rateLimitTier", default)]
    pub rate_limit_tier: Option<String>,
}

fn deserialize_expires_at<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Some(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Some(f as i64))
            } else {
                Err(de::Error::custom("invalid number for expires_at"))
            }
        }
        _ => Err(de::Error::custom("expected number or null for expires_at")),
    }
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
pub struct ClaudeTierData {
    pub plan_name: String,
    pub rate_limit_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmpUsageData {
    pub quota: f64,
    pub used: f64,
    pub used_percent: f64,
    pub hourly_replenishment: f64,
    pub window_hours: Option<f64>,
    pub resets_at: Option<i64>,
}
