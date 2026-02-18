use crate::credentials::CredentialManager;
use crate::models::AmpUsageData;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{debug_amp, debug_error, debug_net};

const AMP_SETTINGS_URL: &str = "https://ampcode.com/settings";

pub struct AmpService;

impl AmpService {
    pub async fn amp_fetch_usage(client: &Arc<reqwest::Client>) -> Result<AmpUsageData> {
        debug_amp!("amp_fetch_usage: Starting request");
        debug_net!("GET {}", AMP_SETTINGS_URL);

        let session_cookie = CredentialManager::amp_read_session_cookie()?;
        debug_amp!("Using session cookie: ***REDACTED***");

        let response = client
            .get(AMP_SETTINGS_URL)
            .header("Cookie", format!("session={}", session_cookie))
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://ampcode.com")
            .send()
            .await?;

        debug_net!("Response status: {}", response.status());

        let status = response.status();

        // Check for auth redirects
        if status.is_redirection() {
            if let Some(location) = response.headers().get("location") {
                let loc = location.to_str().unwrap_or_default();
                debug_amp!("Redirect to: {}", loc);
                let loc_lower = loc.to_lowercase();
                if loc_lower.contains("login")
                    || loc_lower.contains("signin")
                    || loc_lower.contains("auth")
                {
                    debug_error!("Amp session expired (redirect to login)");
                    return Err(anyhow!(
                        "Amp session expired — please update your session cookie"
                    ));
                }
            }
            return Err(anyhow!(
                "Amp: Unexpected redirect (HTTP {})",
                status.as_u16()
            ));
        }

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            debug_error!("Amp auth error (HTTP {})", status.as_u16());
            return Err(anyhow!(
                "Amp session invalid — please update your session cookie"
            ));
        }

        if !status.is_success() {
            debug_error!("Amp request failed (HTTP {})", status.as_u16());
            return Err(anyhow!("Amp: Failed to fetch settings (HTTP {})", status));
        }

        let body = response.text().await?;
        debug_amp!("Response body length: {} bytes", body.len());

        // Check for login page content (more specific markers)
        let body_lower = body.to_lowercase();
        if body_lower.contains("sign in to your account")
            || body_lower.contains("log in to your account")
        {
            debug_error!("Amp session expired (login page detected)");
            return Err(anyhow!(
                "Amp session expired — please update your session cookie"
            ));
        }

        // Parse freeTierUsage data from embedded JavaScript
        Self::parse_free_tier_usage(&body)
    }

    fn parse_free_tier_usage(html: &str) -> Result<AmpUsageData> {
        // Find freeTierUsage or getFreeTierUsage in the HTML
        let search_terms = ["freeTierUsage", "getFreeTierUsage"];
        let mut obj_start = None;

        'outer: for term in &search_terms {
            let patterns = [
                format!("{}: {{", term),
                format!("{}:{{", term),
                format!("{} = {{", term),
                format!("{}={{", term),
            ];
            for pattern in &patterns {
                if let Some(pos) = html.find(pattern.as_str()) {
                    debug_amp!("Found '{}' at position {}", pattern, pos);
                    let actual_brace = pos + html[pos..].find('{').unwrap_or(0);
                    obj_start = Some(actual_brace);
                    break 'outer;
                }
            }
        }

        let start = obj_start
            .ok_or_else(|| anyhow!("Could not find freeTierUsage data in Amp settings page"))?;

        // Brace-counting to find matching closing brace
        let mut depth = 0;
        let mut end = start;
        for (i, ch) in html[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            return Err(anyhow!("Malformed freeTierUsage object (unmatched braces)"));
        }

        let obj_str = &html[start..end];
        debug_amp!("Extracted object: {}", obj_str);

        // Extract numeric values using regex
        let quota = Self::extract_number(obj_str, "quota")?;
        let used = Self::extract_number(obj_str, "used")?;
        let hourly_replenishment = Self::extract_number(obj_str, "hourlyReplenishment")?;
        let window_hours = Self::extract_number_optional(obj_str, "windowHours");

        debug_amp!(
            "Parsed: quota={}, used={}, hourlyReplenishment={}, windowHours={:?}",
            quota,
            used,
            hourly_replenishment,
            window_hours
        );

        // Compute used_percent clamped 0-100
        let used_percent = if quota > 0.0 {
            ((used / quota) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // resets_at calculates when the next window reset occurs using window_hours
        let resets_at = window_hours.map(|hours| {
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let window_seconds = (hours * 3600.0) as u64;
            let window_start = now_secs - (now_secs % window_seconds);
            let reset_secs = window_start + window_seconds;
            (reset_secs * 1000) as i64
        });

        Ok(AmpUsageData {
            quota,
            used,
            used_percent,
            hourly_replenishment,
            window_hours,
            resets_at,
        })
    }

    fn get_cached_regex(field: &str) -> Regex {
        static RE_CACHE: std::sync::LazyLock<
            std::sync::Mutex<std::collections::HashMap<String, Regex>>,
        > = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

        let pattern = format!(r"{}:\s*([0-9]+(?:\.[0-9]+)?)", regex::escape(field));
        let mut cache = RE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .entry(pattern.clone())
            .or_insert_with(|| Regex::new(&pattern).expect("Failed to compile regex"))
            .clone()
    }

    fn extract_number(obj: &str, field: &str) -> Result<f64> {
        let re = Self::get_cached_regex(field);
        let caps = re
            .captures(obj)
            .ok_or_else(|| anyhow!("Field '{}' not found in freeTierUsage object", field))?;
        caps[1]
            .parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse '{}' value: {}", field, e))
    }

    fn extract_number_optional(obj: &str, field: &str) -> Option<f64> {
        let re = Self::get_cached_regex(field);
        let caps = re.captures(obj)?;
        caps[1].parse::<f64>().ok()
    }

    pub async fn validate_session_cookie(
        client: &Arc<reqwest::Client>,
        cookie: &str,
    ) -> Result<()> {
        let response = client
            .get(AMP_SETTINGS_URL)
            .header("Cookie", format!("session={}", cookie))
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://ampcode.com/settings")
            .send()
            .await?;

        let status = response.status();

        if status.is_redirection() {
            if let Some(location) = response.headers().get("location") {
                let loc = location.to_str().unwrap_or_default().to_lowercase();
                if loc.contains("login") || loc.contains("signin") || loc.contains("auth") {
                    return Err(anyhow!("Invalid session cookie — redirected to login"));
                }
            }
            return Err(anyhow!(
                "Amp: Unexpected redirect (HTTP {})",
                status.as_u16()
            ));
        }

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(anyhow!("Invalid session cookie (HTTP {})", status.as_u16()));
        }

        if !status.is_success() {
            return Err(anyhow!("Amp: Validation request failed (HTTP {})", status));
        }

        let body = response.text().await?;
        let body_lower = body.to_lowercase();
        if body_lower.contains("sign in to your account")
            || body_lower.contains("log in to your account")
        {
            return Err(anyhow!("Invalid session cookie — login page returned"));
        }

        Ok(())
    }

    pub fn amp_has_session_cookie() -> bool {
        CredentialManager::amp_has_session_cookie()
    }
}
