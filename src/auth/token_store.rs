use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct TokenStore {
    file_path: PathBuf,
}

impl TokenStore {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?
            .join("tasks");
        
        let file_path = config_dir.join("ms_todo_tokens.json");

        // Ensure the directory exists
        fs::create_dir_all(&config_dir)?;

        Ok(Self { file_path })
    }

    /// Save tokens to storage
    pub fn save_tokens(&self, tokens: &AuthConfig) -> Result<()> {
        let serialized = serde_json::to_string_pretty(tokens)?;
        fs::write(&self.file_path, serialized)?;
        Ok(())
    }

    /// Load tokens from storage
    pub fn load_tokens(&self) -> Result<AuthConfig> {
        if self.file_path.exists() {
            let data = fs::read_to_string(&self.file_path)?;
            let config: AuthConfig = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            Err(anyhow!(
                "No tokens found in {}",
                self.file_path.display()
            ))
        }
    }

    /// Check if tokens exist
    #[allow(dead_code)]
    pub fn has_tokens(&self) -> bool {
        self.file_path.exists()
    }

    /// Get storage method info for debugging
    pub fn get_storage_info(&self) -> String {
        format!("file: {:?}", self.file_path)
    }
}
