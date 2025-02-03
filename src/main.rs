#![allow(warnings)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::env;
use tokio::fs;
use std::collections::HashMap;
use serde_json;

use crate::schema::{DrupalFileAsset, DrupalFileAssetsWrapper, DrupalFileAssetsResponse};
use crate::downloader::{Downloader, DownloadConfig};
use crate::config::CliConfig;

mod schema;
mod downloader;
mod config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync files from a remote source
    Sync {
        /// Path to assets metadata file or URL
        #[arg(long)]
        assets_metadata: Option<String>,

        /// Destination directory for downloaded files
        #[arg(long)]
        destination: Option<PathBuf>,

        /// Base URL for file downloads
        #[arg(long)]
        base_url: String,

        /// Maximum number of concurrent downloads
        #[arg(long, default_value_t = 4)]
        max_concurrent: usize,

        /// Username for metadata source
        #[arg(long)]
        source_username: Option<String>,

        /// Password for metadata source
        #[arg(long)]
        source_password: Option<String>,

        /// Username for file downloads
        #[arg(long)]
        download_username: Option<String>,

        /// Password for file downloads
        #[arg(long)]
        download_password: Option<String>,

        /// Delay between downloads in milliseconds
        #[arg(long, default_value_t = 100)]
        download_delay: u64,

        /// Download timeout in seconds
        #[arg(long, default_value_t = 60)]
        download_timeout: u64,

        /// Maximum number of retries for failed downloads
        #[arg(long, default_value_t = 3)]
        max_retries: usize,

        /// Force download even if file exists
        #[arg(long)]
        force: bool,
    },

    /// Configure the CLI
    Config {
        /// Base URL for file downloads
        #[arg(long)]
        base_url: Option<String>,

        /// Default destination path for downloads
        #[arg(long)]
        desti_path: Option<String>,

        /// Username for metadata source
        #[arg(long)]
        source_username: Option<String>,

        /// Password for metadata source
        #[arg(long)]
        source_password: Option<String>,

        /// Username for file downloads
        #[arg(long)]
        download_username: Option<String>,

        /// Password for file downloads
        #[arg(long)]
        download_password: Option<String>,

        /// Delay between downloads in milliseconds
        #[arg(long, default_value_t = 100)]
        download_delay: u64,

        /// Download timeout in seconds
        #[arg(long, default_value_t = 60)]
        download_timeout: u64,

        /// Maximum number of retries for failed downloads
        #[arg(long, default_value_t = 3)]
        max_retries: usize,

        /// Force download even if file exists
        #[arg(long)]
        force: bool,
    },
}

async fn get_config_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Failed to get config directory")?
        .join("cli-file-sync"))
}

/// Compare two asset lists and return only the changed or new assets
fn get_changed_assets(old_assets: &[DrupalFileAsset], new_assets: &[DrupalFileAsset]) -> Vec<DrupalFileAsset> {
    let mut changed = Vec::new();
    let old_map: HashMap<_, _> = old_assets
        .iter()
        .map(|asset| (asset.id.clone(), asset))
        .collect();

    for new_asset in new_assets {
        match old_map.get(&new_asset.id) {
            Some(old_asset) => {
                if old_asset.changed != new_asset.changed {
                    changed.push(new_asset.clone());
                }
            }
            None => {
                changed.push(new_asset.clone());
            }
        }
    }

    changed
}

async fn download_metadata(source: &str, destination: &Path, force: bool, username: Option<String>, password: Option<String>) -> Result<Vec<DrupalFileAsset>> {
    // Create destination directory if it doesn't exist
    println!("Ensuring destination directory exists: {}", destination.display());
    if !destination.exists() {
        tokio::fs::create_dir_all(destination).await.context(format!("Failed to create directory: {}", destination.display()))?;
    }

    let metadata_path = destination.join("assets.json");
    println!("Will save metadata to: {}", metadata_path.display());
    
    // First, always download or read the content
    let content = if source.starts_with("http://") || source.starts_with("https://") {
        println!("Downloading metadata from {}", source);
        println!("This may take a while for large files...");
        
        let mut request = reqwest::Client::new().get(source);
        
        if let (Some(username), Some(password)) = (username, password) {
            request = request.basic_auth(username, Some(password));
        }
        
        let response = request.send().await.context("Failed to send HTTP request")?;
        println!("Response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download metadata: HTTP {} {}",
                response.status().as_u16(),
                response.status().as_str()
            ));
        }
        
        let content = response.text().await.context("Failed to read response body")?;
        println!("Download complete! Content length: {} bytes", content.len());
        if content.len() > 0 {
            println!("Content preview: {}", &content[..std::cmp::min(content.len(), 200)]);
        } else {
            println!("Warning: Downloaded content is empty!");
        }
        
        println!("Saving content to file: {}", metadata_path.display());
        tokio::fs::write(&metadata_path, &content)
            .await
            .context(format!("Failed to write content to {}", metadata_path.display()))?;
        
        // Verify the file was written
        if metadata_path.exists() {
            println!("Successfully wrote metadata file");
            let file_size = tokio::fs::metadata(&metadata_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
            println!("File size: {} bytes", file_size);
        } else {
            println!("Warning: File was not created!");
        }
        
        content
    } else {
        println!("Reading local file {}", source);
        tokio::fs::read_to_string(source).await?
    };

    // Now try parsing the content
    println!("Parsing metadata from {}...", metadata_path.display());
    
    // Try parsing as raw value first to understand the structure
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(value) => {
            println!("Successfully parsed as JSON. Root structure: {}", 
                if value.is_object() { "object" }
                else if value.is_array() { "array" }
                else { "other" }
            );
            
            if let Some(obj) = value.as_object() {
                println!("Available fields at root: {:?}", obj.keys().collect::<Vec<_>>());
                if let Some(files) = obj.get("files") {
                    if let Some(files_arr) = files.as_array() {
                        println!("Found files array with {} items", files_arr.len());
                    } else {
                        println!("'files' field is not an array");
                    }
                }
            }
        }
        Err(e) => println!("Failed to parse as raw JSON: {}", e),
    }
    
    // Try parsing as a wrapper
    match serde_json::from_str::<DrupalFileAssetsWrapper>(&content) {
        Ok(wrapper) => {
            println!("Successfully parsed as wrapper with {} files", wrapper.files.len());
            Ok(wrapper.files)
        }
        Err(wrapper_err) => {
            // If that fails, try parsing as an array
            match serde_json::from_str::<Vec<DrupalFileAsset>>(&content) {
                Ok(assets) => {
                    println!("Successfully parsed as array with {} files", assets.len());
                    Ok(assets)
                }
                Err(array_err) => {
                    println!("Failed to parse as wrapper: {}", wrapper_err);
                    println!("Failed to parse as array: {}", array_err);
                    Err(anyhow::anyhow!("Failed to parse metadata as JSON: {}", wrapper_err))
                }
            }
        }
    }
}

async fn handle_sync_command(
    assets_metadata: &str,
    destination: &Path,
    base_url: &str,
    max_concurrent: usize,
    force: bool,
    username: Option<String>,
    password: Option<String>,
) -> Result<()> {
    // Get the current working directory
    let current_dir = std::env::current_dir()?;
    
    // If destination is just a name (like "downloads"), make it relative to current directory
    let destination = if destination.is_absolute() {
        destination.to_path_buf()
    } else {
        current_dir.join(destination)
    };

    // Download or read metadata file
    let assets = download_metadata(
        assets_metadata,
        &destination,
        force,
        username.clone(),
        password.clone(),
    )
    .await?;

    println!("Found {} assets to process", assets.len());

    // Configure downloader
    let config = DownloadConfig {
        max_concurrent,
        base_url: Some(base_url.to_string()),
        username,
        password,
        ..Default::default()
    };

    let downloader = Downloader::new(config);
    downloader.download_files(&assets, destination).await?;

    Ok(())
}

async fn handle_config_command(
    base_url: Option<String>,
    desti_path: Option<String>,
    source_username: Option<String>,
    source_password: Option<String>,
    download_username: Option<String>,
    download_password: Option<String>,
    download_delay: u64,
    download_timeout: u64,
    max_retries: usize,
    force: bool,
) -> Result<()> {
    let config_id = "default"; // Use a default config ID
    
    // Try to load existing config or create new one
    let mut config = if let Ok(existing) = CliConfig::load(config_id).await {
        existing
    } else {
        CliConfig::new(config_id.to_string(), ".".to_string()) // Default to current directory
    };

    // Update config with new values if provided
    if let Some(url) = base_url {
        config.base_url = Some(url);
    }
    if let Some(path) = desti_path {
        config.desti_path = path;
    }
    
    // Update download settings
    config.source_username = source_username;
    config.source_password = source_password;
    config.download_username = download_username;
    config.download_password = download_password;
    config.download_delay = download_delay;
    config.download_timeout = download_timeout;
    config.max_retries = max_retries;

    // Save the updated config
    config.save().await?;

    println!("Configuration updated successfully:");
    println!("  Base URL: {:?}", config.base_url);
    println!("  Destination Path: {}", config.desti_path);
    println!("  Source Username: {:?}", config.source_username);
    println!("  Source Password: {:?}", config.source_password);
    println!("  Download Username: {:?}", config.download_username);
    println!("  Download Password: {:?}", config.download_password);
    println!("  Download Delay: {}ms", config.download_delay);
    println!("  Download Timeout: {}s", config.download_timeout);
    println!("  Max Retries: {}", config.max_retries);

    Ok(())
}

fn get_default_auth() -> (Option<String>, Option<String>) {
    let source_username = env::var("CLI_SYNC_SOURCE_USER").ok();
    let source_password = env::var("CLI_SYNC_SOURCE_PASS").ok();
    (source_username, source_password)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync {
            assets_metadata,
            destination,
            base_url,
            max_concurrent,
            source_username,
            source_password,
            download_username,
            download_password,
            download_delay,
            download_timeout,
            max_retries,
            force,
        } => {
            let assets_metadata = assets_metadata.ok_or_else(|| anyhow::anyhow!("No assets metadata provided"))?;
            let destination = destination.unwrap_or_else(|| PathBuf::from("data"));

            handle_sync_command(
                &assets_metadata,
                &destination,
                &base_url,
                max_concurrent,
                force,
                download_username,
                download_password,
            )
            .await
        }
        Commands::Config {
            base_url,
            desti_path,
            source_username,
            source_password,
            download_username,
            download_password,
            download_delay,
            download_timeout,
            max_retries,
            force,
        } => {
            handle_config_command(
                base_url,
                desti_path,
                source_username,
                source_password,
                download_username,
                download_password,
                download_delay,
                download_timeout,
                max_retries,
                force,
            )
            .await
        }
    }
}
