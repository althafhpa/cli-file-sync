use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use csv::Writer;

/// Represents a sync operation record for CSV export
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRecord {
    /// Unique identifier for the sync operation
    pub sync_id: String,
    /// Timestamp of the operation
    pub timestamp: DateTime<Utc>,
    /// Type of operation (add/update/delete)
    pub operation: String,
    /// File path
    pub file_path: String,
    /// File size in bytes
    pub file_size: u64,
    /// Status (success/failure)
    pub status: String,
    /// Error message if any
    pub error: Option<String>,
    /// Source URL or path
    pub source: String,
    /// Destination path
    pub destination: String,
    /// MD5 hash of the file
    pub md5: String,
    /// Configuration ID used
    pub config_id: String,
}

/// Represents a failure record for CSV export
#[derive(Debug, Serialize, Deserialize)]
pub struct FailureRecord {
    /// Timestamp of the failure
    pub timestamp: DateTime<Utc>,
    /// File that failed
    pub file: String,
    /// Error type
    pub error_type: String,
    /// Error message
    pub error_message: String,
    /// Additional details
    pub details: String,
    /// Configuration ID
    pub config_id: String,
}

/// Report writer that handles both CSV and JSON formats
pub struct ReportWriter {
    csv_path: PathBuf,
    json_path: PathBuf,
}

impl ReportWriter {
    /// Creates a new report writer
    pub fn new(base_path: PathBuf, report_type: &str) -> Self {
        let csv_path = base_path.with_extension("csv");
        let json_path = base_path.with_extension("json");
        Self { csv_path, json_path }
    }

    /// Writes a sync record to both CSV and JSON
    pub async fn write_sync_record(&self, record: &SyncRecord) -> Result<()> {
        // Write to CSV
        let mut wtr = Writer::from_path(&self.csv_path)?;
        wtr.serialize(record)?;
        wtr.flush()?;

        // Also keep JSON for compatibility
        let json = serde_json::to_string_pretty(record)?;
        fs::write(&self.json_path, json).await?;

        Ok(())
    }

    /// Writes multiple sync records
    pub async fn write_sync_records(&self, records: &[SyncRecord]) -> Result<()> {
        // Write to CSV
        let mut wtr = Writer::from_path(&self.csv_path)?;
        for record in records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;

        // Also keep JSON for compatibility
        let json = serde_json::to_string_pretty(records)?;
        fs::write(&self.json_path, json).await?;

        Ok(())
    }

    /// Writes a failure record
    pub async fn write_failure_record(&self, record: &FailureRecord) -> Result<()> {
        // Write to CSV
        let mut wtr = Writer::from_path(&self.csv_path)?;
        wtr.serialize(record)?;
        wtr.flush()?;

        // Also keep JSON for compatibility
        let json = serde_json::to_string_pretty(record)?;
        fs::write(&self.json_path, json).await?;

        Ok(())
    }
}

/// Manages the log directory and rotation
pub struct LogManager {
    log_dir: PathBuf,
    max_logs: u32,
}

impl LogManager {
    /// Creates a new log manager
    pub fn new(log_dir: PathBuf, max_logs: u32) -> Self {
        Self { log_dir, max_logs }
    }

    /// Creates a new log file with current timestamp
    pub async fn create_log_file(&self) -> Result<ReportWriter> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let base_path = self.log_dir.join(format!("sync_log_{}", timestamp));
        Ok(ReportWriter::new(base_path, "sync_log"))
    }

    /// Rotates logs based on max_logs configuration
    pub async fn rotate_logs(&self) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(&self.log_dir)
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

        // Group CSV and JSON files together
        let mut files_to_remove = Vec::new();
        let num_pairs = entries.len() / 2;
        if num_pairs > self.max_logs as usize {
            let to_remove = num_pairs - self.max_logs as usize;
            files_to_remove.extend(entries.iter().take(to_remove * 2));
        }

        // Remove oldest logs
        for entry in files_to_remove {
            fs::remove_file(entry.path()).await?;
        }

        Ok(())
    }
}
