use anyhow::Result;
use mime_guess;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Serialize)]
pub struct AssetEntry {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub mime_type: String,
    pub download_url: Option<String>,
}

#[derive(Debug)]
pub struct AssetListingConfig {
    pub base_url: Option<String>,
    pub output_path: PathBuf,
}

impl AssetEntry {
    pub async fn from_path(path: &Path, base_path: &Path, config: &AssetListingConfig) -> Result<Self> {
        let metadata = fs::metadata(path).await?;
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let rel_path = path.strip_prefix(base_path)?;
        let path_str = rel_path.to_string_lossy().to_string();

        let download_url = config.base_url.as_ref().map(|base| {
            format!(
                "{}/{}",
                base.trim_end_matches('/'),
                path_str.trim_start_matches('/')
            )
        });

        Ok(Self {
            filename: path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: path_str,
            size: metadata.len(),
            mime_type,
            download_url,
        })
    }
}

pub async fn generate_asset_listing(
    dir_path: &Path,
    config: &AssetListingConfig,
) -> Result<()> {
    let mut entries = Vec::new();

    let mut read_dir = fs::read_dir(dir_path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let asset = AssetEntry::from_path(&path, dir_path, config).await?;
            entries.push(asset);
        }
    }

    let json = serde_json::to_string_pretty(&entries)?;
    let output_file = config.output_path.join("assets.json");
    fs::write(output_file, json).await?;

    Ok(())
}
