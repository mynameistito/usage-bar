use crate::models::{
    CodexAuthFile, CodexCredits, CodexRefreshResponse, CodexTierData, CodexUsageData,
    CodexUsageResponse, CodexUsageWindow, CodexWindowUsage,
};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{debug_error, debug_net};

const CODEX_DEFAULT_BASE_URL: &str = "https://chatgpt.com/backend-api";
const CODEX_USAGE_PATH: &str = "/wham/usage";
const CODEX_ALT_USAGE_PATH: &str = "/api/codex/usage";
const CODEX_REFRESH_URL: &str = "https://auth.openai.com/oauth/token";
const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

pub struct CodexService;

impl CodexService {
    pub fn codex_has_auth() -> bool {
        Self::auth_path().is_ok_and(|path| path.exists())
            && Self::read_auth().is_ok_and(|auth| {
                auth.openai_api_key
                    .as_ref()
                    .is_some_and(|key| !key.is_empty())
                    || auth
                        .tokens
                        .as_ref()
                        .is_some_and(|tokens| !tokens.access_token.is_empty())
            })
    }

    pub async fn codex_fetch_usage_and_tier(
        client: Arc<reqwest::Client>,
    ) -> Result<(CodexUsageData, CodexTierData)> {
        let mut auth = Self::read_auth()?;
        let mut response = Self::fetch_usage(client.clone(), &auth).await;

        if matches!(response, Err(CodexFetchError::Unauthorized)) {
            debug_error!("Codex usage token unauthorized; attempting refresh");
            auth = Self::refresh_auth(client.clone(), auth).await?;
            response = Self::fetch_usage(client, &auth).await;
        }

        let response = response.map_err(|e| e.into_anyhow())?;
        let usage = Self::map_usage_response(response);
        let tier = CodexTierData {
            plan_name: usage
                .tier_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
        };

        Ok((usage, tier))
    }

    fn auth_path() -> Result<PathBuf> {
        if let Ok(codex_home) = std::env::var("CODEX_HOME") {
            let trimmed = codex_home.trim();
            if !trimmed.is_empty() {
                return Ok(PathBuf::from(trimmed).join("auth.json"));
            }
        }

        let home = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("USERPROFILE environment variable not set"))?;
        Ok(home.join(".codex").join("auth.json"))
    }

    fn read_auth() -> Result<CodexAuthFile> {
        let path = Self::auth_path()?;
        let path_display = path.display();
        let json = fs::read_to_string(&path).map_err(|e| {
            anyhow!(
                "Codex auth not found: failed to read {path_display}. {e}. Run `codex` to sign in."
            )
        })?;
        serde_json::from_str(&json).map_err(|e| anyhow!("Failed to parse Codex auth.json: {e}"))
    }

    fn write_auth(auth: &CodexAuthFile) -> Result<()> {
        let path = Self::auth_path()?;
        let existing = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
        let mut root: serde_json::Value =
            serde_json::from_str(&existing).unwrap_or_else(|_| serde_json::json!({}));

        if let Some(tokens) = &auth.tokens {
            root["tokens"] = serde_json::to_value(tokens)
                .map_err(|e| anyhow!("Failed to serialize Codex tokens: {e}"))?;
        }

        let json = serde_json::to_string_pretty(&root)
            .map_err(|e| anyhow!("Failed to serialize Codex auth.json: {e}"))?;
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, json).map_err(|e| anyhow!("Failed to write Codex auth.json: {e}"))?;
        fs::rename(&temp_path, &path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            anyhow!("Failed to save Codex auth.json: {e}")
        })
    }

    async fn fetch_usage(
        client: Arc<reqwest::Client>,
        auth: &CodexAuthFile,
    ) -> Result<CodexUsageResponse, CodexFetchError> {
        let token = Self::access_token(auth).map_err(CodexFetchError::Other)?;
        let account_id = auth
            .tokens
            .as_ref()
            .and_then(|tokens| tokens.account_id.as_deref());
        let url = Self::usage_url();
        debug_net!("GET {url}");

        let mut request = client
            .get(url)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .header("User-Agent", "codex-cli");

        if let Some(account_id) = account_id {
            request = request.header("ChatGPT-Account-Id", account_id);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CodexFetchError::Other(e.into()))?;
        let status = response.status();
        debug_net!("Codex response status: {status}");

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(CodexFetchError::Unauthorized),
            status if status.is_success() => {
                response.json::<CodexUsageResponse>().await.map_err(|e| {
                    CodexFetchError::Other(anyhow!("Invalid response from Codex usage API: {e}"))
                })
            }
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(CodexFetchError::Other(anyhow!(
                    "Codex API error {status}: {body}"
                )))
            }
        }
    }

    async fn refresh_auth(
        client: Arc<reqwest::Client>,
        mut auth: CodexAuthFile,
    ) -> Result<CodexAuthFile> {
        let tokens = auth.tokens.as_mut().ok_or_else(|| {
            anyhow!("Codex auth.json contains no OAuth tokens. Run `codex` to sign in.")
        })?;
        let refresh_token = tokens
            .refresh_token
            .as_deref()
            .ok_or_else(|| anyhow!("Codex refresh token missing. Run `codex` to sign in again."))?;

        debug_net!("POST {CODEX_REFRESH_URL}");
        let response = client
            .post(CODEX_REFRESH_URL)
            .json(&serde_json::json!({
                "client_id": CODEX_CLIENT_ID,
                "grant_type": "refresh_token",
                "refresh_token": refresh_token,
                "scope": "openid profile email"
            }))
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(anyhow!(
                "Codex refresh token expired or invalid. Run `codex` to sign in again."
            ));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Codex token refresh failed ({status}): {body}"));
        }

        let refreshed = response
            .json::<CodexRefreshResponse>()
            .await
            .map_err(|e| anyhow!("Invalid Codex token refresh response: {e}"))?;

        tokens.access_token = refreshed.access_token;
        if let Some(refresh_token) = refreshed.refresh_token {
            tokens.refresh_token = Some(refresh_token);
        }
        if refreshed.id_token.is_some() {
            tokens.id_token = refreshed.id_token;
        }
        Self::write_auth(&auth)?;

        Ok(auth)
    }

    fn access_token(auth: &CodexAuthFile) -> Result<&str> {
        if let Some(api_key) = auth.openai_api_key.as_deref() {
            if !api_key.is_empty() {
                return Ok(api_key);
            }
        }
        auth.tokens
            .as_ref()
            .map(|tokens| tokens.access_token.as_str())
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                anyhow!("Codex auth.json contains no access token. Run `codex` to sign in.")
            })
    }

    fn usage_url() -> String {
        let base = Self::chatgpt_base_url();
        let normalized = normalize_url(&base);
        let path = if normalized.contains("/backend-api") {
            CODEX_USAGE_PATH
        } else {
            CODEX_ALT_USAGE_PATH
        };
        format!("{normalized}{path}")
    }

    fn chatgpt_base_url() -> String {
        let Ok(config_path) = Self::auth_path().map(|path| path.with_file_name("config.toml"))
        else {
            return CODEX_DEFAULT_BASE_URL.to_string();
        };
        let Ok(config) = fs::read_to_string(config_path) else {
            return CODEX_DEFAULT_BASE_URL.to_string();
        };

        for raw_line in config.lines() {
            let line = raw_line.split('#').next().unwrap_or_default().trim();
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() != "chatgpt_base_url" {
                continue;
            }
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return value.to_string();
            }
        }

        CODEX_DEFAULT_BASE_URL.to_string()
    }

    fn map_usage_response(response: CodexUsageResponse) -> CodexUsageData {
        let rate_limit = response.rate_limit;
        let session_usage = rate_limit
            .as_ref()
            .and_then(|limit| limit.primary_window.as_ref())
            .map(Self::map_window);
        let weekly_usage = rate_limit
            .as_ref()
            .and_then(|limit| limit.secondary_window.as_ref())
            .map(Self::map_window);
        let credits = response.credits.map(|credits| CodexCredits {
            has_credits: credits.has_credits,
            unlimited: credits.unlimited,
            balance: credits.balance,
        });

        CodexUsageData {
            session_usage,
            weekly_usage,
            credits,
            tier_name: response.plan_type.map(format_plan_name),
        }
    }

    fn map_window(window: &CodexUsageWindow) -> CodexWindowUsage {
        CodexWindowUsage {
            percentage: window.used_percent,
            resets_at: window.reset_at.checked_mul(1000),
            window_seconds: Some(window.limit_window_seconds),
        }
    }
}

enum CodexFetchError {
    Unauthorized,
    Other(anyhow::Error),
}

impl CodexFetchError {
    fn into_anyhow(self) -> anyhow::Error {
        match self {
            Self::Unauthorized => {
                anyhow!("Codex OAuth token expired or invalid. Run `codex` to re-authenticate.")
            }
            Self::Other(error) => error,
        }
    }
}

fn normalize_url(value: &str) -> String {
    let mut trimmed = value.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        trimmed = CODEX_DEFAULT_BASE_URL.to_string();
    }
    if (trimmed.starts_with("https://chatgpt.com")
        || trimmed.starts_with("https://chat.openai.com"))
        && !trimmed.contains("/backend-api")
    {
        trimmed.push_str("/backend-api");
    }
    trimmed
}

fn format_plan_name(plan: String) -> String {
    let mut parts = Vec::new();
    for part in plan.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            parts.push(format!(
                "{}{}",
                first.to_uppercase(),
                chars.as_str().to_lowercase()
            ));
        }
    }
    if parts.is_empty() {
        "Unknown".to_string()
    } else {
        parts.join(" ")
    }
}
