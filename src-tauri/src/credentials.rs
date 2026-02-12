use crate::models::ClaudeOAuthCredentials;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::FILETIME;
use windows::Win32::Security::Credentials::*;

pub struct CredentialManager;

impl CredentialManager {
    const ZAI_TARGET: &'static str = "usage-bar-zai-credentials";

    // ── Claude credentials (file-based: ~/.claude/.credentials.json) ──

    fn claude_credentials_path() -> Result<PathBuf> {
        println!("[DEBUG] claude_credentials_path called");
        let home = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("USERPROFILE environment variable not set"))?;
        println!("[DEBUG] USERPROFILE: {:?}", home);

        let claude_dir = home.join(".claude");
        println!("[DEBUG] claude_dir: {:?}", claude_dir);

        // Check both possible filenames — .credentials.json (dot prefix) and credentials.json
        let dot_path = claude_dir.join(".credentials.json");
        let plain_path = claude_dir.join("credentials.json");

        println!(
            "[DEBUG] Checking dot_path: {:?} exists: {}",
            dot_path,
            dot_path.exists()
        );
        println!(
            "[DEBUG] Checking plain_path: {:?} exists: {}",
            plain_path,
            plain_path.exists()
        );

        if dot_path.exists() {
            println!("[DEBUG] Using dot_path");
            Ok(dot_path)
        } else if plain_path.exists() {
            println!("[DEBUG] Using plain_path");
            Ok(plain_path)
        } else {
            println!("[DEBUG] Neither exists, defaulting to dot_path");
            // Default to .credentials.json when neither exists (for error messages)
            Ok(dot_path)
        }
    }

    pub fn read_claude_credentials() -> Result<ClaudeOAuthCredentials> {
        println!("[DEBUG] read_claude_credentials called");
        let path = Self::claude_credentials_path()?;
        println!("[DEBUG] Reading credentials from: {:?}", path);

        let json_str = fs::read_to_string(&path).map_err(|e| {
            println!("[DEBUG] Failed to read file: {}", e);
            anyhow!(
                "Credential not found: failed to read {}. {}. \
                 Make sure you are logged in to Claude Code.",
                path.display(),
                e
            )
        })?;
        println!(
            "[DEBUG] Read {} bytes from credentials file",
            json_str.len()
        );

        let credentials: ClaudeOAuthCredentials = serde_json::from_str(&json_str).map_err(|e| {
            println!("[DEBUG] Failed to parse JSON: {}", e);
            anyhow!("Failed to parse Claude credentials: {}", e)
        })?;
        println!("[DEBUG] Successfully parsed credentials");

        Ok(credentials)
    }

    pub fn write_claude_credentials(credentials: &ClaudeOAuthCredentials) -> Result<()> {
        let path = Self::claude_credentials_path()?;

        // Read existing file to preserve fields we don't model (file belongs to Claude Code)
        let mut root: serde_json::Value = if path.exists() {
            let existing = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str(&existing).unwrap_or_else(|_| serde_json::json!({}))
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
        let credential = Self::read_credential(Self::ZAI_TARGET)?;

        let blob_slice = unsafe {
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
        };
        let key = String::from_utf8(blob_slice.to_vec())
            .map_err(|e| anyhow!("Failed to decode API key: {}", e))?;

        Ok(key)
    }

    pub fn write_zai_api_key(api_key: &str) -> Result<()> {
        Self::write_credential(Self::ZAI_TARGET, api_key)
    }

    pub fn delete_zai_api_key() -> Result<()> {
        Self::delete_credential(Self::ZAI_TARGET)
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

            let credential_data = (*credential_ptr).clone();
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
            let result = CredWriteW(&credential, 0);

            if result.is_err() {
                return Err(anyhow!("Failed to write credential: {}", target_name));
            }

            Ok(())
        }
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
