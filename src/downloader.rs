use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::sleep;
use reqwest::header::AUTHORIZATION;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as base64_engine;
use chrono;

use crate::schema::DrupalFileAsset;

#[derive(Debug, Serialize, Clone)]
pub struct FailedDownload {
    pub filename: String,
    pub path: String,
    pub error: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_concurrent: usize,
    pub download_delay: u64,      // milliseconds
    pub download_timeout: u64,    // seconds
    pub max_retries: usize,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,      // 4 concurrent downloads
            download_delay: 100,    // 100ms delay between downloads
            download_timeout: 30,    // 30 seconds timeout
            max_retries: 3,         // 3 retries for failed downloads
            base_url: None,
            username: None,
            password: None,
        }
    }
}

pub struct Downloader {
    config: DownloadConfig,
    failed_downloads: Arc<Mutex<Vec<FailedDownload>>>,
}

impl Downloader {
    pub fn new(config: DownloadConfig) -> Self {
        Self {
            config,
            failed_downloads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn download_files(&self, assets: &[DrupalFileAsset], destination: PathBuf) -> Result<()> {
        let client = reqwest::Client::new();
        let config = self.config.clone();
        let max_concurrent = config.max_concurrent;

        let mut handles = Vec::new();

        // Clone all assets first to avoid lifetime issues
        let assets: Vec<DrupalFileAsset> = assets.to_vec();

        for asset in assets {
            let client = client.clone();
            let config = config.clone();
            let destination = destination.clone();
            let failed_downloads = self.failed_downloads.clone();

            let handle = tokio::spawn(async move {
                if let Err(e) = Self::download_single_file(&asset, &client, &config, &destination).await {
                    let failed = FailedDownload {
                        filename: asset.filename.clone(),
                        path: asset.path.clone(),
                        error: e.to_string(),
                        timestamp: chrono::Utc::now(),
                    };
                    failed_downloads.lock().await.push(failed);
                }
                sleep(Duration::from_millis(config.download_delay)).await;
            });

            handles.push(handle);

            if handles.len() >= max_concurrent {
                for handle in handles.drain(..) {
                    handle.await?;
                }
            }
        }

        for handle in handles {
            handle.await?;
        }

        Ok(())
    }

    fn get_download_url(asset: &DrupalFileAsset, config: &DownloadConfig) -> Result<String> {
        let base_url = config.base_url.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Base URL is required for downloading assets")
        })?;

        let base = base_url.trim_end_matches('/');
        let path = asset.path.trim_start_matches('/');
        let url = format!("{}/{}", base, path);
        
        Ok(url)
    }

    async fn download_single_file(
        asset: &DrupalFileAsset,
        client: &reqwest::Client,
        config: &DownloadConfig,
        destination: &PathBuf,
    ) -> Result<()> {
        let url = Self::get_download_url(asset, config)?;
        let dest_path = destination.join(&asset.filename);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut request = client.get(&url);

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            request = request.header(
                AUTHORIZATION,
                format!("Basic {}", base64_engine.encode(format!("{}:{}", username, password)))
            );
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download file: {} (status: {})",
                url,
                response.status()
            ));
        }

        let content = response.bytes().await?;
        fs::write(&dest_path, content).await?;

        // Set file permissions to be readable and writable by the owner
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&dest_path).await?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o644); // rw-r--r--
            fs::set_permissions(&dest_path, perms).await?;
        }

        Ok(())
    }
}
