use crate::models::ClaudeOAuthCredentials;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::FILETIME;
use windows::Win32::Security::Credentials::*;

use crate::debug_cred;

/// Short-lived credential cache to avoid repeated file/Win32 reads within a single operation batch.
/// TTL is intentionally short (5 seconds) since credentials can change externally.
struct CredentialCache {
    claude_credentials: Option<(Instant, ClaudeOAuthCredentials)>,
    zai_api_key: Option<(Instant, String)>,
}

impl CredentialCache {
    const TTL: Duration = Duration::from_secs(5);

    fn new() -> Self {
        Self {
            claude_credentials: None,
            zai_api_key: None,
        }
    }

    fn get_claude(&self) -> Option<ClaudeOAuthCredentials> {
        self.claude_credentials
            .as_ref()
            .and_then(|(instant, creds)| {
                if instant.elapsed() < Self::TTL {
                    Some(creds.clone())
                } else {
                    None
                }
            })
    }

    fn set_claude(&mut self, creds: ClaudeOAuthCredentials) {
        self.claude_credentials = Some((Instant::now(), creds));
    }

    fn invalidate_claude(&mut self) {
        self.claude_credentials = None;
    }

    fn get_zai(&self) -> Option<String> {
        self.zai_api_key.as_ref().and_then(|(instant, key)| {
            if instant.elapsed() < Self::TTL {
                Some(key.clone())
            } else {
                None
            }
        })
    }

    fn set_zai(&mut self, key: String) {
        self.zai_api_key = Some((Instant::now(), key));
    }

    fn invalidate_zai(&mut self) {
        self.zai_api_key = None;
    }
}

static CACHE: Mutex<Option<CredentialCache>> = Mutex::new(None);

fn with_cache<F, R>(f: F) -> R
where
    F: FnOnce(&mut CredentialCache) -> R,
{
    let mut guard = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_none() {
        *guard = Some(CredentialCache::new());
    }
    f(guard.as_mut().unwrap())
}

pub struct CredentialManager;

impl CredentialManager {
    const ZAI_TARGET: &'static str = "usage-bar-zai-credentials";

    // ── Claude credentials (file-based: ~/.claude/.credentials.json) ──

    fn claude_credentials_path() -> Result<PathBuf> {
        debug_cred!("claude_credentials_path called");
        let home = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("USERPROFILE environment variable not set"))?;
        debug_cred!("USERPROFILE: {:?}", home);

        let claude_dir = home.join(".claude");
        debug_cred!("claude_dir: {:?}", claude_dir);

        // Check both possible filenames — .credentials.json (dot prefix) and credentials.json
        let dot_path = claude_dir.join(".credentials.json");
        let plain_path = claude_dir.join("credentials.json");

        debug_cred!(
            "Checking dot_path: {:?} exists: {}",
            dot_path,
            dot_path.exists()
        );
        debug_cred!(
            "Checking plain_path: {:?} exists: {}",
            plain_path,
            plain_path.exists()
        );

        if dot_path.exists() {
            debug_cred!("Using dot_path");
            Ok(dot_path)
        } else if plain_path.exists() {
            debug_cred!("Using plain_path");
            Ok(plain_path)
        } else {
            debug_cred!("Neither exists, defaulting to dot_path");
            // Default to .credentials.json when neither exists (for error messages)
            Ok(dot_path)
        }
    }

    pub fn read_claude_credentials() -> Result<ClaudeOAuthCredentials> {
        debug_cred!("read_claude_credentials called");

        // Check cache first
        if let Some(cached) = with_cache(|c| c.get_claude()) {
            debug_cred!("Returning cached Claude credentials");
            return Ok(cached);
        }

        let path = Self::claude_credentials_path()?;
        debug_cred!("Reading credentials from: {:?}", path);

        let json_str = fs::read_to_string(&path).map_err(|e| {
            debug_cred!("Failed to read file: {}", e);
            anyhow!(
                "Credential not found: failed to read {}. {}. \
                 Make sure you are logged in to Claude Code.",
                path.display(),
                e
            )
        })?;
        debug_cred!("Read {} bytes from credentials file", json_str.len());

        let credentials: ClaudeOAuthCredentials = serde_json::from_str(&json_str).map_err(|e| {
            debug_cred!("Failed to parse JSON: {}", e);
            anyhow!("Failed to parse Claude credentials: {}", e)
        })?;
        debug_cred!("Successfully parsed credentials");

        // Cache the result
        with_cache(|c| c.set_claude(credentials.clone()));

        Ok(credentials)
    }

    pub fn write_claude_credentials(credentials: &ClaudeOAuthCredentials) -> Result<()> {
        let path = Self::claude_credentials_path()?;

        // Read existing file to preserve fields we don't model (file belongs to Claude Code)
        let mut root: serde_json::Value = if path.exists() {
            let existing = fs::read_to_string(&path)
                .map_err(|e| anyhow!("Failed to read credentials file: {}", e))?;
            serde_json::from_str(&existing).map_err(|e| {
                anyhow!("Failed to parse credentials file (may be corrupted): {}", e)
            })?
        } else {
            serde_json::json!({})
        };

        // Update only the claudeAiOauth subtree
        let oauth_value = serde_json::to_value(&credentials.claude_ai_oauth)
            .map_err(|e| anyhow!("Failed to serialize credentials: {}", e))?;
        root["claudeAiOauth"] = oauth_value;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create .claude directory: {}", e))?;
        }

        let json_str = serde_json::to_string_pretty(&root)
            .map_err(|e| anyhow!("Failed to serialize credentials: {}", e))?;

        // Atomic write: temp file + rename
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, &json_str)
            .map_err(|e| anyhow!("Failed to write credentials: {}", e))?;
        fs::rename(&temp_path, &path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            anyhow!("Failed to save credentials: {}", e)
        })?;

        // Invalidate cache after writing new credentials
        with_cache(|c| c.invalidate_claude());

        Ok(())
    }

    pub fn read_claude_access_token() -> Result<String> {
        let credentials = Self::read_claude_credentials()?;
        Ok(credentials.claude_ai_oauth.access_token)
    }

    pub fn update_claude_token(
        access_token: &str,
        refresh_token: &str,
        expires_at: i64,
    ) -> Result<()> {
        let mut credentials = Self::read_claude_credentials()?;
        credentials.claude_ai_oauth.access_token = access_token.to_string();
        credentials.claude_ai_oauth.refresh_token = refresh_token.to_string();
        credentials.claude_ai_oauth.expires_at = Some(expires_at);
        Self::write_claude_credentials(&credentials)
    }

    pub fn read_zai_api_key() -> Result<String> {
        // Check cache first
        if let Some(cached) = with_cache(|c| c.get_zai()) {
            debug_cred!("Returning cached Z.ai API key");
            return Ok(cached);
        }

        let credential = Self::read_credential(Self::ZAI_TARGET)?;

        // Extract blob data BEFORE calling CredFree to avoid use-after-free
        let blob_slice = unsafe {
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
        };

        // Clone the data to owned Vec<u8> while the credential is still valid
        let blob_vec = blob_slice.to_vec();

        // Now CredFree is called inside read_credential, which is safe
        // because we've already cloned the data we need

        let key =
            String::from_utf8(blob_vec).map_err(|e| anyhow!("Failed to decode API key: {}", e))?;

        // Cache the result
        with_cache(|c| c.set_zai(key.clone()));

        Ok(key)
    }

    pub fn write_zai_api_key(api_key: &str) -> Result<()> {
        let result = Self::write_credential(Self::ZAI_TARGET, api_key);
        // Invalidate cache after writing
        with_cache(|c| c.invalidate_zai());
        result
    }

    pub fn zai_delete_api_key() -> Result<()> {
        let result = Self::delete_credential(Self::ZAI_TARGET);
        // Invalidate cache after deleting
        with_cache(|c| c.invalidate_zai());
        result
    }

    pub fn has_zai_api_key() -> bool {
        Self::read_credential(Self::ZAI_TARGET).is_ok()
    }

    fn read_credential(target_name: &str) -> Result<CREDENTIALW> {
        let target_name_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();

        let mut credential_ptr: *mut CREDENTIALW = std::ptr::null_mut();

        unsafe {
            let result = CredReadW(
                PCWSTR(target_name_wide.as_ptr()),
                CRED_TYPE_GENERIC,
                0,
                &mut credential_ptr,
            );

            if result.is_err() {
                return Err(anyhow!("Credential not found: {}", target_name));
            }

            let credential_data = *credential_ptr;
            CredFree(credential_ptr as *const _);

            Ok(credential_data)
        }
    }

    fn write_credential(target_name: &str, data: &str) -> Result<()> {
        let target_name_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();
        let blob: Vec<u8> = data.as_bytes().to_vec();

        let credential = CREDENTIALW {
            Flags: windows::Win32::Security::Credentials::CRED_FLAGS(0),
            Type: CRED_TYPE_GENERIC,
            TargetName: PWSTR(target_name_wide.as_ptr() as *mut u16),
            Comment: PWSTR::null(),
            LastWritten: FILETIME::default(),
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            TargetAlias: PWSTR::null(),
            UserName: PWSTR::null(),
            AttributeCount: 0,
            Attributes: std::ptr::null_mut(),
        };

        unsafe {
            // Vectors are still alive here because credential borrows from them
            let result = CredWriteW(&credential, 0);

            if result.is_err() {
                return Err(anyhow!("Failed to write credential: {}", target_name));
            }

            Ok(())
        } // Vectors dropped here, after CredWriteW completes
    }

    fn delete_credential(target_name: &str) -> Result<()> {
        let target_name_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let result = CredDeleteW(PCWSTR(target_name_wide.as_ptr()), CRED_TYPE_GENERIC, 0);

            if result.is_err() {
                return Err(anyhow!("Failed to delete credential: {}", target_name));
            }

            Ok(())
        }
    }
}
