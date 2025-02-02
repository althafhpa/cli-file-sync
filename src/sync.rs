use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Represents the result of a sync operation
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    /// Timestamp when the sync operation completed
    pub timestamp: DateTime<Utc>,
    /// List of files that were newly added
    pub added_files: Vec<String>,
    /// List of files that were updated
    pub updated_files: Vec<String>,
    /// List of files that failed to sync
    pub failed_files: Vec<String>,
    /// List of error messages encountered during sync
    pub errors: Vec<String>,
}

/// Represents a file sync failure
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFailure {
    /// Name of the file that failed to sync
    pub file: String,
    /// Error message describing why the sync failed
    pub error: String,
    /// Additional details about the failure
    pub details: String,
}

/// Configuration for the sync operation
#[derive(Debug)]
pub struct SyncConfig {
    /// Base URL for resolving relative paths
    pub base_url: Option<String>,
    /// Source of the assets (URL or file path)
    pub assets_source: String,
    /// Destination path for synced files
    pub desti_path: PathBuf,
    /// Authentication credentials
    pub auth: Option<(String, String)>,
    /// Time-to-live for automatic sync (in seconds)
    pub ttl: Option<u64>,
    /// Path to write the report file
    pub report_file: Option<PathBuf>,
    /// Maximum number of log files to keep
    pub max_logs: u32,
}

impl SyncResult {
    /// Creates a new SyncResult with the current timestamp
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            added_files: Vec::new(),
            updated_files: Vec::new(),
            failed_files: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Saves the sync result to a JSON file
    pub async fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await.context("Failed to write sync result")?;
        Ok(())
    }
}

/// Checks if a file needs to be synced based on metadata
pub fn needs_sync(source_meta: &crate::Asset, dest_path: &PathBuf) -> bool {
    // If file doesn't exist, it needs sync
    if !dest_path.exists() {
        return true;
    }

    // Compare metadata with existing file
    if let Ok(metadata) = std::fs::metadata(dest_path) {
        // Check file size
        if metadata.len() != source_meta.metadata.filesize {
            return true;
        }

        // Check permissions (on Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode() & 0o777;
            if format!("{:o}", mode) != source_meta.metadata.permissions {
                return true;
            }
        }

        // Check modification time
        if let Ok(modified) = metadata.modified() {
            if let Ok(modified_secs) = modified.duration_since(std::time::UNIX_EPOCH) {
                if modified_secs.as_secs() as i64 != source_meta.metadata.changed {
                    return true;
                }
            }
        }
    }

    false
}

/// Syncs a single file from source to destination
pub async fn sync_file(
    source_url: &str,
    dest_path: &PathBuf,
    auth: &Option<(String, String)>,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut req = client.get(source_url);

    if let Some((username, password)) = auth {
        req = req.basic_auth(username, password);
    }

    let response = req.send().await?;
    let content = response.bytes().await?;

    // Ensure parent directories exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(dest_path, content).await?;

    Ok(())
}

/// Manages log rotation based on max_logs configuration
pub async fn rotate_logs(log_dir: &PathBuf, max_logs: u32) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(log_dir)
        .await?
        .filter_map(|e| e.ok())
        .collect();

    // Sort by modified time
    entries.sort_by_key(|e| {
        e.metadata()
            .unwrap()
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    // Remove oldest logs if we exceed max_logs
    let to_remove = entries.len().saturating_sub(max_logs as usize);
    for entry in entries.iter().take(to_remove) {
        fs::remove_file(entry.path()).await?;
    }

    Ok(())
}
