use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents the source information in the metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct DrupalSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub version: String,
}

/// Represents a single file asset from Drupal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrupalFileAsset {
    pub id: String,
    pub filename: String,
    pub uri: String,
    #[serde(default)]
    pub path: String,
    pub mime: String,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub changed: i64,
    #[serde(default)]
    pub scheme: String,
}

impl DrupalFileAsset {
    /// Validates that the file asset has all required fields and valid values
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Missing ID".to_string());
        }
        if self.filename.is_empty() {
            return Err("Missing filename".to_string());
        }
        if self.uri.is_empty() {
            return Err("Missing URI".to_string());
        }
        if self.mime.is_empty() {
            return Err("Missing MIME type".to_string());
        }
        Ok(())
    }

    /// Gets the local destination path for this file
    pub fn get_local_path(&self, base_path: &str) -> String {
        if self.path.is_empty() {
            // If path is empty, use the filename
            format!("{}/{}", base_path.trim_end_matches('/'), self.filename)
        } else {
            format!("{}/{}", base_path.trim_end_matches('/'), self.path.trim_start_matches('/'))
        }
    }

    /// Checks if the file is an image
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }

    /// Gets the file extension
    pub fn get_extension(&self) -> Option<String> {
        self.filename
            .rsplit('.')
            .next()
            .map(|s| s.to_lowercase())
    }
}

/// Represents a collection of file assets from Drupal with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct DrupalFileAssetsWrapper {
    pub version: String,
    pub generated: i64,
    pub source: DrupalSource,
    pub files: Vec<DrupalFileAsset>,
}

impl DrupalFileAssetsWrapper {
    /// Validates all file assets in the collection
    pub fn validate(&self) -> Result<(), String> {
        for asset in &self.files {
            if let Err(e) = asset.validate() {
                return Err(format!("Asset {} validation failed: {}", asset.id, e));
            }
        }
        Ok(())
    }

    /// Gets total size of all files
    pub fn total_size(&self) -> u64 {
        self.files.iter().filter_map(|f| f.size).sum()
    }

    /// Gets count of image files
    pub fn image_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_image()).count()
    }

    /// Gets files grouped by MIME type
    pub fn group_by_mime(&self) -> HashMap<String, Vec<DrupalFileAsset>> {
        let mut groups: HashMap<String, Vec<DrupalFileAsset>> = HashMap::new();
        for asset in &self.files {
            groups
                .entry(asset.mime.clone())
                .or_default()
                .push(asset.clone());
        }
        groups
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DrupalFileAssetsResponse {
    Wrapper(DrupalFileAssetsWrapper),
    Array(Vec<DrupalFileAsset>),
}

impl DrupalFileAssetsResponse {
    pub fn into_vec(self) -> Vec<DrupalFileAsset> {
        match self {
            DrupalFileAssetsResponse::Wrapper(wrapper) => wrapper.files,
            DrupalFileAssetsResponse::Array(assets) => assets,
        }
    }
}
