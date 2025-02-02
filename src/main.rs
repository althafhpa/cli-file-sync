use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::fs;
use url::Url;
use chrono::Utc;

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
    
    /// Sync files based on configuration
    Sync { 
        /// JSON file containing assets metadata
        #[arg(long = "assets-metadata")]
        assets_source: String,

        /// Base URL for downloading assets (e.g., https://example.com)
        #[arg(long = "assets-base-url")]
        base_url: String,

        /// Optional HTTP Basic Auth username
        #[arg(long = "auth-username")]
        username: Option<String>,

        /// Optional HTTP Basic Auth password
        #[arg(long = "auth-password")]
        password: Option<String>,

        /// Maximum concurrent downloads
        #[arg(long)]
        max_concurrent: Option<usize>,

        /// Delay between downloads in milliseconds
        #[arg(long)]
        download_delay: Option<u64>,

        /// Download timeout in seconds
        #[arg(long)]
        download_timeout: Option<u64>,

        /// Maximum retry attempts per file
        #[arg(long)]
        max_retries: Option<usize>,

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

async fn handle_sync_command(
    assets_source: String,
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    max_concurrent: Option<usize>,
    download_delay: Option<u64>,
    download_timeout: Option<u64>,
    max_retries: Option<usize>,
    max_file_size: Option<u64>,
) -> Result<()> {
    // Load assets JSON from file or URL
    let assets_json = if assets_source.starts_with("http://") || assets_source.starts_with("https://") {
        // Create HTTP client with auth for assets JSON download
        let mut request = reqwest::Client::new().get(&assets_source);
        if let (Some(username), Some(password)) = (&username, &password) {
            request = request.header(
                reqwest::header::AUTHORIZATION,
                format!("Basic {}", base64::encode(format!("{}:{}", username, password)))
            );
        }
        request.send().await?.text().await?
    } else {
        tokio::fs::read_to_string(&assets_source).await?
    };

    let assets: DrupalFileAssets = serde_json::from_str(&assets_json)?;
    
    // Validate the assets
    if let Err(e) = assets.validate() {
        return Err(anyhow::anyhow!("Invalid assets JSON: {}", e));
    }

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
        max_concurrent: max_concurrent.unwrap_or(5),
        download_delay: download_delay.unwrap_or(0),
        download_timeout: download_timeout.unwrap_or(30),
        max_retries: max_retries.unwrap_or(3),
        max_file_size,
        base_url: Some(base_url),
        username,
        password,
    };

    let mut downloader = Downloader::new(download_config);
    downloader.download_files(&assets.files).await?;

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
            assets_source.clone(),
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
            username,
            password,
            max_concurrent,
            download_delay,
            download_timeout,
            max_retries,
            max_file_size,
        } => {
            handle_sync_command(
                assets_source,
                base_url,
                username,
                password,
                max_concurrent,
                download_delay,
                download_timeout,
                max_retries,
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
