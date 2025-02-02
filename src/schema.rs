use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents a file's metadata from Drupal
#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filesize: u64,
    pub filemime: String,
    pub changed: i64,
    pub created: i64,
    pub md5: String,
    pub permissions: String,
    pub uid: u64,
    pub gid: u64,
    pub inode: u64,
}

/// Represents where the file is used in Drupal
#[derive(Debug, Serialize, Deserialize)]
pub struct FileUsage {
    pub file: Vec<String>,
}

/// Represents a single file asset from Drupal
#[derive(Debug, Serialize, Deserialize)]
pub struct DrupalFileAsset {
    pub id: String,
    pub filename: String,
    pub uri: String,
    pub path: String,
    pub mime: String,
    pub size: u64,
    pub created: i64,
    pub changed: i64,
    pub scheme: String,
}

impl DrupalFileAsset {
    /// Validates that the file asset has all required fields and valid values
    pub fn validate(&self) -> Result<(), String> {
        // Check required fields are not empty
        if self.id.is_empty() {
            return Err("File ID is required".to_string());
        }
        if self.filename.is_empty() {
            return Err("Filename is required".to_string());
        }
        if self.uri.is_empty() {
            return Err("URI is required".to_string());
        }
        if self.path.is_empty() {
            return Err("Path is required".to_string());
        }
        if self.mime.is_empty() {
            return Err("MIME type is required".to_string());
        }

        Ok(())
    }

    /// Gets the local destination path for this file
    pub fn get_local_path(&self, base_path: &str) -> String {
        format!("{}/{}", base_path, self.filename)
    }

    /// Checks if the file is an image
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }

    /// Gets the file extension
    pub fn get_extension(&self) -> Option<String> {
        Path::new(&self.filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
    }
}

/// Represents a collection of file assets from Drupal
#[derive(Debug, Serialize, Deserialize)]
pub struct DrupalFileAssets {
    pub files: Vec<DrupalFileAsset>,
}

impl DrupalFileAssets {
    /// Validates all file assets in the collection
    pub fn validate(&self) -> Result<(), String> {
        for asset in &self.files {
            if let Err(e) = asset.validate() {
                return Err(format!("Invalid asset {}: {}", asset.id, e));
            }
        }
        Ok(())
    }

    /// Gets total size of all files
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|asset| asset.size).sum()
    }

    /// Gets count of image files
    pub fn image_count(&self) -> usize {
        self.files.iter().filter(|asset| asset.is_image()).count()
    }

    /// Gets files grouped by MIME type
    pub fn group_by_mime(&self) -> HashMap<String, Vec<&DrupalFileAsset>> {
        let mut groups = HashMap::new();
        for asset in &self.files {
            groups
                .entry(asset.mime.clone())
                .or_insert_with(Vec::new)
                .push(asset);
        }
        groups
    }
}
