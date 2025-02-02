use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Represents the CLI configuration for a specific destination
#[derive(Debug, Serialize, Deserialize)]
pub struct CliConfig {
    /// Unique identifier for this configuration
    pub id: String,
    /// Base URL for resolving relative paths
    pub base_url: Option<String>,
    /// Destination path for synced files
    pub desti_path: String,
    /// Maximum number of log files to keep
    pub max_logs: u32,
    /// Time-to-live for automatic sync (in seconds)
    pub ttl: Option<u64>,
    /// Timestamp of the last successful sync
    pub last_sync: Option<DateTime<Utc>>,
}

impl CliConfig {
    /// Creates a new configuration instance
    pub fn new(id: String, desti_path: String) -> Self {
        Self {
            id,
            base_url: None,
            desti_path,
            max_logs: 100,
            ttl: None,
            last_sync: None,
        }
    }

    /// Gets the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "althaf", "cli-file-sync")
            .context("Failed to determine project directories")?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }

    /// Gets the configuration file path for a specific ID
    pub fn config_file(id: &str) -> Result<PathBuf> {
        let mut path = Self::config_dir()?;
        path.push(format!("{}.json", id));
        Ok(path)
    }

    /// Loads configuration from file
    pub async fn load(id: &str) -> Result<Self> {
        let path = Self::config_file(id)?;
        let content = fs::read_to_string(path).await?;
        let config: CliConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Saves configuration to file
    pub async fn save(&self) -> Result<()> {
        let path = Self::config_file(&self.id)?;
        
        // Ensure config directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }

    /// Updates the last sync timestamp
    pub fn update_last_sync(&mut self) {
        self.last_sync = Some(Utc::now());
    }

    /// Checks if sync is needed based on TTL
    pub fn needs_sync(&self) -> bool {
        match (self.ttl, self.last_sync) {
            (Some(ttl), Some(last_sync)) => {
                let duration = Utc::now()
                    .signed_duration_since(last_sync)
                    .num_seconds() as u64;
                duration >= ttl
            }
            _ => true,
        }
    }
}

/// Lists all available configurations
pub async fn list_configs() -> Result<Vec<CliConfig>> {
    let config_dir = CliConfig::config_dir()?;
    let mut configs = Vec::new();

    if !config_dir.exists() {
        return Ok(configs);
    }

    let mut entries = fs::read_dir(&config_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(path).await {
                if let Ok(config) = serde_json::from_str(&content) {
                    configs.push(config);
                }
            }
        }
    }

    Ok(configs)
}

/// Validates a destination path
pub async fn validate_desti_path(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).await?;
    }
    Ok(())
}
