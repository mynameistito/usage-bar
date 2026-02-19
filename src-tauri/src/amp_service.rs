use crate::credentials::CredentialManager;
use crate::models::AmpUsageData;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::{Arc, LazyLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{debug_amp, debug_error, debug_net};

const AMP_SETTINGS_URL: &str = "https://ampcode.com/settings";

/// Amp reports monetary values in integer cents; divide by this to get dollars.
/// Verified assumption: the Amp settings page JS object uses cents (integer hundredths).
const CENTS_TO_DOLLARS: f64 = 100.0;

static RE_QUOTA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"quota:\s*([0-9]+(?:\.[0-9]+)?)").unwrap());
static RE_USED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"used:\s*([0-9]+(?:\.[0-9]+)?)").unwrap());
static RE_HOURLY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"hourlyReplenishment:\s*([0-9]+(?:\.[0-9]+)?)").unwrap());
static RE_WINDOW_HOURS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"windowHours:\s*([0-9]+(?:\.[0-9]+)?)").unwrap());

pub struct AmpService;

impl AmpService {
    /// Check redirect and status codes common to Amp HTTP requests.
    /// Returns `Err` for auth failures, unexpected redirects, and non-success status codes.
    /// Differences between callers (cookie source, Referer header, body parsing) remain
    /// in each caller's own function.
    fn check_response_validity(response: &reqwest::Response) -> Result<()> {
        let status = response.status();

        if status.is_redirection() {
            if let Some(location) = response.headers().get("location") {
                let loc = location.to_str().unwrap_or_default().to_lowercase();
                if loc.contains("login") || loc.contains("signin") || loc.contains("auth") {
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

        Ok(())
    }

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

        Self::check_response_validity(&response)?;

        let body = response.text().await?;
        debug_amp!("Response body length: {} bytes", body.len());
        debug_amp!("Response preview: {:?}", &body[..body.len().min(100)]);

        // Check for login page content (more specific markers)
        let body_lower = body.to_lowercase();
        if body_lower.contains("sign in to your account")
            || body_lower.contains("log in to your account")
            || body_lower.contains("please sign in")
            || body_lower.contains("create an account")
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
        // Two search terms: "freeTierUsage" matches property syntax (freeTierUsage: {...}),
        // "getFreeTierUsage" matches getter syntax. Both use ":" or "=" as separators.
        let search_terms = ["freeTierUsage", "getFreeTierUsage"];
        let mut obj_start = None;

        'outer: for term in &search_terms {
            let mut search_from = 0;
            while let Some(pos) = html[search_from..].find(term) {
                let abs_pos = search_from + pos;
                // Skip occurrences that are string values (term is both preceded and followed by a quote)
                let preceded_by_quote = html[..abs_pos]
                    .chars()
                    .next_back()
                    .is_some_and(|c| matches!(c, '"' | '\'' | '`'));
                let end_pos = abs_pos + term.len();
                let followed_by_quote = html[end_pos..]
                    .chars()
                    .next()
                    .is_some_and(|c| matches!(c, '"' | '\'' | '`'));
                // Skip only if it's a string literal (both quotes present)
                if preceded_by_quote && followed_by_quote {
                    search_from = abs_pos + 1;
                    continue;
                }

                // Skip if it's part of a longer quoted string
                if preceded_by_quote && end_pos < html.len() {
                    // Check the character immediately after the term
                    if !html[end_pos..].starts_with(':')
                        && !html[end_pos..].starts_with('=')
                        && !html[end_pos..].starts_with('{')
                    {
                        search_from = abs_pos + 1;
                        continue;
                    }
                }
                let after_term = &html[abs_pos + term.len()..];
                let rest = after_term.trim_start();
                let matched = rest.strip_prefix(':').or_else(|| rest.strip_prefix('='));
                if let Some(after_sep) = matched {
                    let after_sep = after_sep.trim_start();
                    if after_sep.starts_with('{') {
                        let brace_offset = html.len() - after_sep.len();
                        debug_amp!("Found '{}' at position {}", term, abs_pos);
                        obj_start = Some(brace_offset);
                        break 'outer;
                    }
                }
                search_from = abs_pos + 1;
            }
        }

        let start = obj_start.ok_or_else(|| {
            anyhow!(
                "Could not find freeTierUsage in {}-byte response from {}",
                html.len(),
                AMP_SETTINGS_URL
            )
        })?;

        // JavaScript object literal, not valid JSON (unquoted keys, trailing commas possible),
        // so we cannot use serde_json. Instead, use brace-counting to find the object boundaries.
        let mut depth: i32 = 0;
        let mut end = start;
        // Safe to iterate bytes: '{' and '}' are single-byte ASCII characters.
        for (i, b) in html[start..].bytes().enumerate() {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(anyhow!("Mismatched braces in freeTierUsage object"));
                    }
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
        let quota_raw = Self::extract_number(obj_str, &RE_QUOTA, "quota")?;
        let used_raw = Self::extract_number(obj_str, &RE_USED, "used")?;
        let hourly_raw = Self::extract_number(obj_str, &RE_HOURLY, "hourlyReplenishment")?;
        let window_hours = Self::extract_number_optional(obj_str, &RE_WINDOW_HOURS, "windowHours");

        debug_amp!(
            "Parsed raw: quota={}, used={}, hourlyReplenishment={}, windowHours={:?}",
            quota_raw,
            used_raw,
            hourly_raw,
            window_hours
        );

        // Convert cents to dollars
        let quota = quota_raw / CENTS_TO_DOLLARS;
        let used = used_raw / CENTS_TO_DOLLARS;
        let hourly_replenishment = hourly_raw / CENTS_TO_DOLLARS;

        if quota > 10_000.0 {
            debug_amp!(
                "Warning: unusually high quota value {} (raw {}); check cents assumption",
                quota,
                quota_raw
            );
        }

        // Compute used_percent clamped 0-100
        let used_percent = if quota > 0.0 {
            ((used / quota) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // NOTE: Assumes Amp usage windows are aligned to the Unix epoch (1970-01-01 00:00:00 UTC).
        // If Amp uses rolling windows anchored to account creation, this calculation will be wrong.
        let resets_at = window_hours.and_then(|hours| {
            let window_seconds = (hours * 3600.0) as u64;
            if window_seconds == 0 {
                return None;
            }
            let now_secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(d) => d.as_secs(),
                Err(_) => return None,
            };
            // Assumes usage windows align to the Unix epoch.
            let window_start = now_secs - (now_secs % window_seconds);
            let reset_secs = window_start + window_seconds;
            i64::try_from(reset_secs)
                .ok()
                .and_then(|s| s.checked_mul(1000))
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

    fn extract_number(obj: &str, re: &Regex, field_name: &str) -> Result<f64> {
        let caps = re
            .captures(obj)
            .ok_or_else(|| anyhow!("Field '{}' not found in freeTierUsage object", field_name))?;
        caps[1]
            .parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse '{}' value: {}", field_name, e))
    }

    fn extract_number_optional(obj: &str, re: &Regex, field_name: &str) -> Option<f64> {
        match re.captures(obj) {
            None => {
                debug_amp!(
                    "Optional field '{}' not found or malformed in object; defaulting to None",
                    field_name
                );
                None
            }
            Some(caps) => caps[1].parse::<f64>().ok(),
        }
    }

    pub async fn validate_session_cookie(
        client: &Arc<reqwest::Client>,
        cookie: &str,
    ) -> Result<()> {
        let response = client
            .get(AMP_SETTINGS_URL)
            .header("Cookie", format!("session={}", cookie))
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://ampcode.com/settings")
            .send()
            .await?;

        Self::check_response_validity(&response)?;

        Ok(())
    }

    pub fn amp_has_session_cookie() -> bool {
        CredentialManager::amp_has_session_cookie()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_minimal() {
        let html = r#"var data = { freeTierUsage: { quota: 5000, used: 2500, hourlyReplenishment: 100, windowHours: 1.0 } };"#;
        let result = AmpService::parse_free_tier_usage(html).unwrap();
        assert!((result.quota - 50.0).abs() < 0.01); // 5000 cents / 100 = $50
        assert!((result.used - 25.0).abs() < 0.01);
        assert!((result.used_percent - 50.0).abs() < 0.01);
        assert_eq!(result.window_hours, Some(1.0));
        assert!((result.hourly_replenishment - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_missing_required_field_quota() {
        let html = r#"var data = { freeTierUsage: { used: 2500, hourlyReplenishment: 100 } };"#;
        let result = AmpService::parse_free_tier_usage(html);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("quota"), "Expected quota error, got: {}", msg);
    }

    #[test]
    fn test_parse_getter_property_syntax() {
        // "getFreeTierUsage" as a property name assigned with "="
        let html =
            r#"obj.getFreeTierUsage = { quota: 10000, used: 5000, hourlyReplenishment: 200 };"#;
        let result = AmpService::parse_free_tier_usage(html).unwrap();
        assert!((result.quota - 100.0).abs() < 0.01); // 10000 cents = $100
        assert!((result.used - 50.0).abs() < 0.01);
        assert_eq!(result.window_hours, None);
    }

    #[test]
    fn test_parse_unclosed_brace_returns_error() {
        // The freeTierUsage object itself is never closed — no matching '}'
        let html =
            r#"var data = { freeTierUsage: { quota: 5000, used: 1000, hourlyReplenishment: 100"#;
        let result = AmpService::parse_free_tier_usage(html);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Malformed") || msg.contains("unmatched"),
            "Expected brace error, got: {}",
            msg
        );
    }

    #[test]
    fn test_parse_window_hours_absent() {
        let html = r#"var data = { freeTierUsage: { quota: 5000, used: 1000, hourlyReplenishment: 100 } };"#;
        let result = AmpService::parse_free_tier_usage(html).unwrap();
        assert_eq!(result.window_hours, None);
        // resets_at should also be None when window_hours is None
        assert_eq!(result.resets_at, None);
    }

    #[test]
    fn test_parse_skips_string_literal_occurrence() {
        // First occurrence is in a quoted string; real data follows
        let html = r#"var desc = "freeTierUsage is cool"; var obj = { freeTierUsage: { quota: 3000, used: 1500, hourlyReplenishment: 50 } };"#;
        let result = AmpService::parse_free_tier_usage(html).unwrap();
        assert!((result.quota - 30.0).abs() < 0.01); // 3000 cents = $30
        assert!((result.used - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_large_quota_no_panic() {
        // Very large quota should trigger sanity warning but not fail
        let html = r#"var data = { freeTierUsage: { quota: 5000000000, used: 1000, hourlyReplenishment: 100 } };"#;
        let result = AmpService::parse_free_tier_usage(html);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.quota > 10_000.0);
    }
}
