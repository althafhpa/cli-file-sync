use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::fs;
use url::Url;
use chrono::Utc;
use dirs;
use base64;
use std::collections::HashMap;
use std::env;

mod schema;
mod downloader;
mod assets;
mod config;

use schema::DrupalFileAssets;
use downloader::{DownloadConfig, Downloader};
use assets::{generate_asset_listing, AssetListingConfig};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure CLI settings
    Config { 
        /// Base URL for file downloads
        #[arg(long)]
        base_url: Option<String>,
        
        /// Destination path for downloaded files
        #[arg(long)]
        desti_path: Option<String>,
    },
    
    /// Sync files from source to destination
    Sync {
        /// Source URL or path for assets JSON. If not provided, uses cached assets.json
        #[arg(short, long)]
        assets_source: Option<String>,
        
        /// Base URL for resolving relative paths
        #[arg(short, long)]
        base_url: String,

        /// Username for assets source authentication
        #[arg(long)]
        source_username: Option<String>,

        /// Password for assets source authentication
        #[arg(long)]
        source_password: Option<String>,

        /// Username for file downloads authentication
        #[arg(long)]
        download_username: Option<String>,

        /// Password for file downloads authentication
        #[arg(long)]
        download_password: Option<String>,

        /// Maximum concurrent downloads
        #[arg(long, default_value_t = 5)]
        max_concurrent: usize,

        /// Delay between downloads in milliseconds
        #[arg(long, default_value_t = 0)]
        download_delay: u64,

        /// Download timeout in seconds
        #[arg(long, default_value_t = 30)]
        download_timeout: u64,

        /// Maximum retry attempts for failed downloads
        #[arg(long, default_value_t = 3)]
        max_retries: usize,

        /// Maximum file size in bytes
        #[arg(long)]
        max_file_size: Option<u64>,
    },

    /// Watch and automatically sync files periodically
    Watch {
        /// JSON file containing assets metadata
        #[arg(long = "assets-metadata")]
        assets_source: String,

        /// Base URL for downloading assets
        #[arg(long = "assets-base-url")]
        base_url: String,

        /// Sync interval in seconds (default: 24 hours)
        #[arg(long, default_value = "86400")]  // 24 hours = 86400 seconds
        interval: u64,

        /// Optional HTTP Basic Auth username
        #[arg(long = "auth-username")]
        username: Option<String>,

        /// Optional HTTP Basic Auth password
        #[arg(long = "auth-password")]
        password: Option<String>,
    },

    /// List assets and generate report
    List {
        /// Source directory or URL containing assets
        #[arg(long)]
        assets_source: String,

        /// Output path for the asset listing
        #[arg(long)]
        output: String,

        /// Base URL for generating download URLs
        #[arg(long)]
        base_url: Option<String>,
    },
}

/// Gets the default config directory for cli-file-sync
fn get_config_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Failed to get config directory")?
        .join("cli-file-sync"))
}

/// Gets the default assets file path
fn get_default_assets_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("assets.json"))
}

/// Downloads assets source file if it's a URL and returns the local path
async fn download_assets_source(
    source: &str,
    username: &Option<String>,
    password: &Option<String>,
) -> Result<PathBuf> {
    let config_dir = get_config_dir()?;
    
    // Create config directory if it doesn't exist
    tokio::fs::create_dir_all(&config_dir).await?;
    
    let assets_path = if source.starts_with("http://") || source.starts_with("https://") {
        let assets_file = get_default_assets_path()?;
        
        // Download the assets file
        let mut request = reqwest::Client::new().get(source);
        if let (Some(username), Some(password)) = (username, password) {
            request = request.header(
                reqwest::header::AUTHORIZATION,
                format!("Basic {}", base64::encode(format!("{}:{}", username, password)))
            );
        }
        
        let response = request.send().await?;
        let content = response.text().await?;
        
        // Write to local file
        tokio::fs::write(&assets_file, content).await?;
        assets_file
    } else {
        PathBuf::from(source)
    };
    
    Ok(assets_path)
}

fn get_default_auth() -> ((Option<String>, Option<String>), (Option<String>, Option<String>)) {
    (
        // Source auth
        (
            env::var("CLI_SYNC_SOURCE_USER").ok(),
            env::var("CLI_SYNC_SOURCE_PASS").ok(),
        ),
        // Download auth
        (
            env::var("CLI_SYNC_DOWNLOAD_USER").ok(),
            env::var("CLI_SYNC_DOWNLOAD_PASS").ok(),
        )
    )
}

async fn handle_sync_command(
    assets_source: Option<String>,
    base_url: String,
    source_username: Option<String>,
    source_password: Option<String>,
    download_username: Option<String>,
    download_password: Option<String>,
    max_concurrent: Option<usize>,
    download_delay: Option<u64>,
    download_timeout: Option<u64>,
    max_retries: Option<usize>,
    max_file_size: Option<u64>,
) -> Result<()> {
    // Get default auth from environment if not provided
    let ((default_source_user, default_source_pass), (default_download_user, default_download_pass)) = get_default_auth();
    let source_username = source_username.or(default_source_user);
    let source_password = source_password.or(default_source_pass);
    let download_username = download_username.or(default_download_user);
    let download_password = download_password.or(default_download_pass);

    let default_path = get_default_assets_path()?;
    
    // Get the assets file path and content
    let (local_assets_path, new_assets_json) = match assets_source {
        Some(source) => {
            // If source is provided, download it
            println!("Downloading assets source...");
            let path = download_assets_source(
                &source,
                &source_username,
                &source_password,
            ).await?;
            let content = tokio::fs::read_to_string(&path).await
                .with_context(|| format!("Failed to read assets file: {}", path.display()))?;
            println!("Assets source downloaded to: {}", path.display());
            (path, content)
        }
        None => {
            // If no source provided, try to use cached file
            if !default_path.exists() {
                anyhow::bail!(
                    "No assets source provided and no cached assets found at {}. \
                    Please provide an assets source using --assets-source", 
                    default_path.display()
                );
            }
            println!("Using cached assets from: {}", default_path.display());
            let content = tokio::fs::read_to_string(&default_path).await
                .with_context(|| format!("Failed to read assets file: {}", default_path.display()))?;
            (default_path.clone(), content)
        }
    };

    // Check if we need to sync
    if assets_source.is_some() && default_path.exists() && local_assets_path != default_path {
        // Compare new content with cached content
        let cached_json = tokio::fs::read_to_string(&default_path).await
            .with_context(|| format!("Failed to read cached assets: {}", default_path.display()))?;
            
        if new_assets_json == cached_json {
            println!("No changes detected in assets file");
            return Ok(());
        }
        println!("Changes detected in assets file, proceeding with sync");
    }

    // Parse the assets JSON
    let assets: DrupalFileAssets = serde_json::from_str(&new_assets_json)?;
    
    // Print summary
    println!("Found {} files to sync", assets.files.len());
    println!("Total size: {} bytes", assets.total_size());
    println!("Images: {}", assets.image_count());

    // Group by MIME type
    for (mime, files) in assets.group_by_mime() {
        println!("{}: {} files", mime, files.len());
    }

    // Configure downloader
    let download_config = DownloadConfig {
        username: download_username,
        password: download_password,
        max_concurrent: max_concurrent.unwrap_or(5),
        download_delay: download_delay.unwrap_or(0),
        download_timeout: download_timeout.unwrap_or(30),
        max_retries: max_retries.unwrap_or(3),
        max_file_size,
        base_url: Some(base_url),
    };

    let mut downloader = Downloader::new(download_config);
    downloader.download_files(&assets.files).await?;

    // If we downloaded from a new source, update the cache
    if assets_source.is_some() && local_assets_path != default_path {
        tokio::fs::copy(local_assets_path, default_path).await?;
        println!("Updated assets cache");
    }

    Ok(())
}

async fn handle_list_command(
    assets_source: String,
    output: String,
    base_url: Option<String>,
) -> Result<()> {
    // Create output directory if it doesn't exist
    let output_path = PathBuf::from(output);
    fs::create_dir_all(&output_path).await?;

    // Check if assets_source is a URL or local path
    let is_url = Url::parse(&assets_source).is_ok();

    let config = AssetListingConfig {
        base_url,
        output_path,
    };

    if is_url {
        // TODO: Implement URL handling
        return Err(anyhow::anyhow!("URL source not yet implemented"));
    } else {
        let source_path = PathBuf::from(assets_source);
        generate_asset_listing(&source_path, &config).await?;
    }

    Ok(())
}

async fn handle_watch_command(
    assets_source: String,
    base_url: String,
    interval: u64,
    username: Option<String>,
    password: Option<String>,
) -> Result<()> {
    println!("Starting watch mode with {} second interval", interval);
    
    loop {
        println!("Running sync at {}", Utc::now());
        
        if let Err(e) = handle_sync_command(
            Some(assets_source.clone()),
            base_url.clone(),
            username.clone(),
            password.clone(),
            None,  // Use default max_concurrent
            None,  // Use default download_delay
            None,  // Use default download_timeout
            None,  // Use default max_retries
            None,  // Use default max_file_size
        ).await {
            eprintln!("Sync error: {}", e);
        }

        println!("Next sync scheduled at {}", Utc::now() + chrono::Duration::seconds(interval as i64));
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { base_url: _, desti_path: _ } => {
            // TODO: Implement config handling
            Ok(())
        }
        Commands::Sync { 
            assets_source,
            base_url,
            source_username,
            source_password,
            download_username,
            download_password,
            max_concurrent,
            download_delay,
            download_timeout,
            max_retries,
            max_file_size,
        } => {
            handle_sync_command(
                assets_source,
                base_url,
                source_username,
                source_password,
                download_username,
                download_password,
                Some(max_concurrent),
                Some(download_delay),
                Some(download_timeout),
                Some(max_retries),
                max_file_size,
            ).await
        }
        Commands::List { assets_source, output, base_url } => {
            handle_list_command(assets_source, output, base_url).await
        }
        Commands::Watch { 
            assets_source,
            base_url,
            interval,
            username,
            password,
        } => {
            handle_watch_command(
                assets_source,
                base_url,
                interval,
                username,
                password,
            ).await
        }
    }
}
