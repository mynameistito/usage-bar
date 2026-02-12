use crate::credentials::CredentialManager;
use crate::models::{ZaiQuotaResponse, ZaiUsageData, ZaiTierData, McpUsage, TokenUsage};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::sync::Arc;

const ZAI_API_URL: &str = "https://api.z.ai/api/monitor/usage/quota/limit";

pub struct ZaiService;

impl ZaiService {
  pub async fn fetch_quota(client: Arc<reqwest::Client>) -> Result<ZaiUsageData> {
    let api_key = CredentialManager::read_zai_api_key()?;

    let response = client
      .get(ZAI_API_URL)
      .header("Authorization", &api_key)
      .header("Accept-Language", "en-US,en")
      .header("Content-Type", "application/json")
      .send()
      .await?;

    match response.status() {
      StatusCode::UNAUTHORIZED => {
        Err(anyhow!("z.ai: Invalid API key — please reconfigure"))
      }
      StatusCode::FORBIDDEN => {
        Err(anyhow!("z.ai: Access denied"))
      }
      StatusCode::TOO_MANY_REQUESTS => {
        Err(anyhow!("z.ai: Rate limited — please wait"))
      }
      status if status.is_success() => {
        Self::handle_response(response).await
      }
      status if status.is_server_error() => {
        Err(anyhow!("z.ai: Server error — try again later"))
      }
      _ => {
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
            resets_at: limit.next_reset_time.map(|ts| Self::format_timestamp(ts)),
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
      if total >= 1400 {
        Some("Max".to_string())
      } else if total >= 300 {
        Some("Pro".to_string())
      } else if total > 0 {
        Some("Lite".to_string())
      } else {
        None
      }
    });

    Ok(ZaiUsageData {
      token_usage,
      mcp_usage,
      tier_name,
    })
  }

  fn format_timestamp(ts: i64) -> String {
    use std::time::UNIX_EPOCH;
    use std::time::Duration;
    
    // Convert milliseconds to seconds and nanoseconds
    let secs = (ts / 1000) as u64;
    let nanos = ((ts % 1000) * 1_000_000) as u32;
    let duration = Duration::new(secs, nanos);
    
    // Convert to SystemTime
    let system_time = UNIX_EPOCH + duration;
    
    // Format as RFC3339-like string (2024-01-15T10:30:00Z)
    let secs_since_epoch = system_time
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    
    // Simple RFC3339 formatting without chrono
    let days_since_epoch = secs_since_epoch / 86400;
    let secs_of_day = secs_since_epoch % 86400;
    let hours = secs_of_day / 3600;
    let mins = (secs_of_day % 3600) / 60;
    let secs = secs_of_day % 60;
    
    // Convert days since epoch to YYYY-MM-DD
    let (year, month, day) = Self::days_to_ymd(days_since_epoch);
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, mins, secs)
  }

  fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    
    (year as u32, m as u32, d as u32)
  }

  pub fn has_api_key() -> bool {
    CredentialManager::has_zai_api_key()
  }

  pub async fn validate_api_key(client: Arc<reqwest::Client>, api_key: &str) -> Result<()> {
    let api_key = api_key.trim();
    
    if api_key.is_empty() {
      return Err(anyhow!("API key cannot be empty"));
    }
    
    if api_key.len() < 10 {
      return Err(anyhow!("API key is too short"));
    }

    let response = client
      .get(ZAI_API_URL)
      .header("Authorization", api_key)
      .header("Accept-Language", "en-US,en")
      .header("Content-Type", "application/json")
      .send()
      .await
      .map_err(|e| {
        if e.is_timeout() {
          anyhow!("Connection timed out - check your network")
        } else if e.is_connect() {
          anyhow!("Could not connect to Z.AI - check your network")
        } else {
          anyhow!("Network error: {}", e)
        }
      })?;

    match response.status() {
      StatusCode::UNAUTHORIZED => Err(anyhow!("Invalid API key")),
      StatusCode::FORBIDDEN => Err(anyhow!("Access denied - key may lack permissions")),
      StatusCode::TOO_MANY_REQUESTS => Err(anyhow!("Rate limited - try again later")),
      status if status.is_server_error() => Err(anyhow!("Z.AI server error - try again later")),
      status if status.is_success() => {
        let body = response.text().await
          .map_err(|e| anyhow!("Failed to read response: {}", e))?;
        
        if body.contains("\"error\"") {
          return Err(anyhow!("Invalid API key"));
        }
        
        if !body.contains("\"limits\"") && !body.contains("\"data\"") {
          return Err(anyhow!("Unexpected response - key may be invalid"));
        }
        
        Ok(())
      }
      _ => Err(anyhow!("Failed to validate API key (HTTP {})", response.status())),
    }
  }

  pub async fn fetch_tier(client: Arc<reqwest::Client>) -> Result<ZaiTierData> {
    let usage_data = Self::fetch_quota(client).await?;
    
    let plan_name = usage_data.tier_name.unwrap_or_else(|| "Unknown".to_string());
    
    Ok(ZaiTierData {
      plan_name,
    })
  }
}
