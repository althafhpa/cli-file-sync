use anyhow::Result;
use futures::stream::{self, StreamExt};
use serde::Serialize;
use std::path::Path;
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
    pub download_delay: u64,
    pub download_timeout: u64,
    pub max_retries: usize,
    pub max_file_size: Option<u64>,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,      // 3 concurrent downloads
            download_delay: 100,    // 100ms delay between downloads
            download_timeout: 30,    // 30 seconds timeout
            max_retries: 3,         // 3 retries for failed downloads
            max_file_size: None,    // No file size limit by default
            base_url: None,
            username: None,
            password: None,
        }
    }
}

pub struct Downloader {
    config: Arc<DownloadConfig>,
    client: reqwest::Client,
    auth_header: Option<String>,
    failed_downloads: Arc<Mutex<Vec<FailedDownload>>>,
}

impl Downloader {
    pub fn new(config: DownloadConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.download_timeout))
            .build()
            .expect("Failed to create HTTP client");

        let auth_header = match (&config.username, &config.password) {
            (Some(username), Some(password)) => {
                Some(format!(
                    "Basic {}",
                    base64_engine.encode(format!("{}:{}", username, password))
                ))
            }
            _ => None,
        };

        Self { 
            config: Arc::new(config),
            client,
            auth_header,
            failed_downloads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn download_files(&self, assets: &[DrupalFileAsset]) -> Result<()> {
        let config = Arc::clone(&self.config);
        let failed_downloads = Arc::clone(&self.failed_downloads);
        let client = self.client.clone();
        let auth_header = self.auth_header.clone();

        let mut stream = stream::iter(assets)
            .map(move |asset| {
                let config = Arc::clone(&config);
                let failed_downloads = Arc::clone(&failed_downloads);
                let client = client.clone();
                let auth_header = auth_header.clone();
                async move {
                    Self::download_file(
                        asset,
                        &config,
                        &client,
                        auth_header.as_ref(),
                        &failed_downloads,
                    ).await
                }
            })
            .buffered(self.config.max_concurrent);

        while let Some(result) = stream.next().await {
            if let Err(e) = result {
                eprintln!("Error downloading file: {}", e);
            }
            if self.config.download_delay > 0 {
                sleep(Duration::from_millis(self.config.download_delay)).await;
            }
        }

        // Save failed downloads report if there are any failures
        let failed_downloads = self.failed_downloads.lock().await;
        if !failed_downloads.is_empty() {
            let report = serde_json::to_string_pretty(&*failed_downloads)?;
            fs::write("failed_downloads.json", report).await?;
            println!("\nFailed downloads report saved to failed_downloads.json");
            println!("Number of failed downloads: {}", failed_downloads.len());
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

    async fn download_file(
        asset: &DrupalFileAsset,
        config: &DownloadConfig,
        client: &reqwest::Client,
        auth_header: Option<&String>,
        failed_downloads: &Arc<Mutex<Vec<FailedDownload>>>,
    ) -> Result<()> {
        if let Some(max_size) = config.max_file_size {
            if asset.size > max_size {
                let error = format!(
                    "File exceeds maximum size limit ({} > {})",
                    asset.size,
                    max_size
                );
                failed_downloads.lock().await.push(FailedDownload {
                    filename: asset.filename.clone(),
                    path: asset.path.clone(),
                    error,
                    timestamp: chrono::Utc::now(),
                });
                return Err(anyhow::anyhow!(
                    "File {} exceeds maximum size limit ({} > {})",
                    asset.filename,
                    asset.size,
                    max_size
                ));
            }
        }

        let dest_path = asset.get_local_path("downloads");
        let dest_dir = Path::new(&dest_path).parent().unwrap();
        fs::create_dir_all(dest_dir).await?;

        let mut retries = 0;
        let mut last_error = None;

        while retries < config.max_retries {
            match Self::attempt_download(asset, config, client, auth_header, &dest_path).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    retries += 1;
                    if retries < config.max_retries {
                        sleep(Duration::from_secs(1 << retries)).await;
                    }
                }
            }
        }

        // Record the failed download
        if let Some(error) = &last_error {
            failed_downloads.lock().await.push(FailedDownload {
                filename: asset.filename.clone(),
                path: asset.path.clone(),
                error: error.to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error during download")))
    }

    async fn attempt_download(
        asset: &DrupalFileAsset,
        config: &DownloadConfig,
        client: &reqwest::Client,
        auth_header: Option<&String>,
        dest_path: &str,
    ) -> Result<()> {
        let url = Self::get_download_url(asset, config)?;
        
        let mut request = client.get(&url);
        if let Some(auth) = auth_header {
            request = request.header(AUTHORIZATION, auth);
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download {}: HTTP {}",
                asset.filename,
                response.status()
            ));
        }

        let bytes = response.bytes().await?;

        if bytes.len() as u64 != asset.size {
            return Err(anyhow::anyhow!(
                "Downloaded file size mismatch for {}: expected {}, got {}",
                asset.filename,
                asset.size,
                bytes.len()
            ));
        }

        fs::write(dest_path, bytes).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(dest_path).await?.permissions();
            fs::set_permissions(dest_path, perms).await?;
        }

        Ok(())
    }
}
