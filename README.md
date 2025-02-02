# CLI File Sync

A powerful command-line tool for downloading and syncing files from URLs. Designed for efficiently managing bulk downloads from HTTP/HTTPS sources, with support for concurrent downloads, automatic retries, and smart file synchronization.

## Key Features

- **URL-based File Downloads**: 
  - Download files from any HTTP/HTTPS URL
  - Support for both absolute and relative URLs
  - Smart base URL resolution for managing multiple file sources
  - Handles redirects and URL-encoded paths

- **Efficient Synchronization**:
  - Concurrent downloads with configurable limits
  - Automatic retry mechanism for failed downloads
  - Incremental updates - only download changed files
  - Progress tracking and detailed logging

- **Advanced Features**:
  - HTTP Basic Authentication support
  - File permission management
  - Detailed failure reporting
  - JSON-based configuration
  - Customizable download settings (timeout, delay, retries)

## Installation

1. Clone the repository:
```bash
git clone git@github.com:althafhpa/cli-file-sync.git
cd cli-file-sync
```

2. Build the project:
```bash
cargo build --release
```

The binary will be available at `target/release/cli-file-sync`

## Prerequisites

- Rust 1.70 or higher
- Cargo (Rust's package manager)
- Git

## Quick Start

```bash
# Download files from a JSON configuration
cli-file-sync sync \
  --source https://example.com/files.json \
  --dest /path/to/files \
  --base-url https://cdn.example.com \
  --concurrency 4

# Check what would be downloaded (dry run)
cli-file-sync check \
  --source https://example.com/files.json \
  --dest /path/to/files
```

## Usage

The CLI supports four main commands:

### 1. Sync Command

```bash
cli-file-sync sync \
  --assets-source <URL_OR_PATH> \
  [--base-url <BASE_URL>] \
  [--desti-path <DESTINATION_PATH>] \
  [--auth-username <USERNAME>] \
  [--auth-password <PASSWORD>] \
  [--ttl <SECONDS>] \
  [--report-file <PATH>] \
  [--max-logs <NUMBER>] \
  [--max-concurrent 5] \
  [--download-delay 100] \
  [--download-timeout 30] \
  [--max-retries 3] \
  [--max-file-size 104857600]
```

### 2. Config Command

```bash
cli-file-sync config \
  [--base-url <BASE_URL>] \
  [--desti-path <PATH>] \
  [--max-logs <NUMBER>] \
  [--max-concurrent 5] \
  [--download-delay 100] \
  [--download-timeout 30] \
  [--max-retries 3] \
  [--max-file-size 104857600]
```

### 3. Check Command

```bash
cli-file-sync check \
  --assets-source <URL_OR_PATH> \
  [--desti-path <PATH>]
```

### 4. List Assets Command

The list command supports both remote URLs and local paths as the assets source:

```bash
# For remote assets
cli-file-sync list \
  --assets-source https://xyz.com \
  --output download/files \
  --base-url https://example.com/files

# For local assets
cli-file-sync list \
  --assets-source /path/to/assets \
  --output /path/to/output \
  --base-url https://example.com/files
```

The command will:
1. Create the output directory if it doesn't exist
2. Generate both CSV and JSON asset listings
3. Include full download URLs when --base-url is provided
4. Create thumbnails for image files

Output files:
- `assets_YYYYMMDD_HHMMSS.csv`: CSV format asset listing
- `assets_YYYYMMDD_HHMMSS.json`: JSON format asset listing

Each asset entry includes:
- File metadata (name, size, type)
- Source and destination paths
- Full download URLs (when --base-url is provided)
- Sync status and timestamps
- Thumbnails for supported image formats

This command generates:
- `assets_YYYYMMDD_HHMMSS.csv`: CSV format asset listing
- `assets_YYYYMMDD_HHMMSS.json`: JSON format asset listing

Both formats include:
- File metadata (name, size, type)
- Source and destination paths
- Full download URLs (when --base-url is provided)
- Sync status and timestamps

## File Synchronization

### Basic Sync Command
```bash
cli-file-sync sync --assets-source https://example.com/files.json
```

### Advanced Sync Options
```bash
cli-file-sync sync \
  --assets-source https://example.com/files.json \
  --max-concurrent 10 \
  --download-delay 100 \
  --download-timeout 60 \
  --max-retries 5 \
  --max-file-size 100000000
```

### Command Options
- `--assets-source`: URL or path to the JSON asset definitions (required)
- `--max-concurrent`: Maximum number of concurrent downloads (default: 5)
- `--download-delay`: Delay between downloads in milliseconds (default: 100)
- `--download-timeout`: Download timeout in seconds (default: 30)
- `--max-retries`: Maximum retry attempts per file (default: 3)
- `--max-file-size`: Maximum allowed file size in bytes (optional)

### Example with Local JSON File
```bash
cli-file-sync sync --assets-source ./sample-assets.json
```

### Example with Authentication
```bash
# Set environment variables for authentication
export DRUPAL_USER=username
export DRUPAL_PASS=password

# Run sync command
cli-file-sync sync --assets-source https://example.com/files.json
```

### Authentication

The tool supports separate authentication for the assets source and file downloads:

1. **Assets Source Authentication** (for accessing the JSON metadata):
```bash
cli-file-sync sync \
  --assets-source https://api.example.com/files.json \
  --source-username apiuser \
  --source-password apipass \
  --base-url https://cdn.example.com
```

2. **Download Authentication** (for downloading the actual files):
```bash
cli-file-sync sync \
  --assets-source https://api.example.com/files.json \
  --base-url https://cdn.example.com \
  --download-username cdnuser \
  --download-password cdnpass
```

3. **Using Both** (when source and downloads require different authentication):
```bash
cli-file-sync sync \
  --assets-source https://api.example.com/files.json \
  --base-url https://cdn.example.com \
  --source-username apiuser \
  --source-password apipass \
  --download-username cdnuser \
  --download-password cdnpass
```

### Output Format
The sync command will show:
1. Number of files found
2. Total size of all files
3. Number of image files
4. Files grouped by MIME type
5. Download progress for each file

Example output:
```
Found 25 files to sync
Total size: 1234567 bytes
Images: 15
image/jpeg: 10 files
image/png: 5 files
application/pdf: 8 files
text/plain: 2 files

Successfully downloaded: example1.jpg
Successfully downloaded: example2.png
...
```

## URL-based Downloads

The tool supports downloading files from both relative and absolute URLs:

1. **Direct URL Downloads**:
```bash
cli-file-sync sync \
  --source https://example.com/files.json \
  --dest /path/to/files \
  --base-url https://cdn.example.com
```

2. **URL Configuration in JSON**:
```json
{
  "files": [
    {
      "id": "1",
      "filename": "document.pdf",
      "uri": "https://cdn.example.com/files/document.pdf",
      "path": "path/to/document.pdf",
      "mime": "application/pdf",
      "size": 1024,
      "created": 1612345678,
      "changed": 1612345679,
      "scheme": "public"
    },
    {
      "id": "2",
      "filename": "image.jpg",
      "uri": "/files/image.jpg",  // Relative to base-url
      "path": "path/to/image.jpg",
      "mime": "image/jpeg",
      "size": 2048,
      "created": 1612345680,
      "changed": 1612345681,
      "scheme": "public"
    }
  ]
}
```

3. **Base URL Override**:
- Use `--base-url` to override the default URL for relative paths
- Supports both HTTP and HTTPS protocols
- Can include authentication credentials in the URL (though using `--auth-username` and `--auth-password` is recommended)

4. **URL Resolution Rules**:
- Absolute URLs (`https://...`) are used as-is
- Relative URLs (`/files/...`) are resolved against the base URL
- URLs can include query parameters and fragments
- Supports URL-encoded characters in paths

## Configuration

### JSON Asset Format

The tool expects asset definitions in the following JSON format:

```json
[
  {
    "fid": "48",
    "filename": "example.jpg",
    "uri": "public://example.jpg",
    "real_path": "sites/default/files/example.jpg",
    "source": "file",
    "metadata": {
      "filesize": 178567,
      "filemime": "image/jpeg",
      "changed": 1349155742,
      "created": 1349155742,
      "md5": "5fe94c2b9ecab79578630eb259eaf8a8",
      "permissions": "0664",
      "uid": 24090475,
      "gid": 1438417908,
      "inode": 124726
    },
    "usage": {
      "file": ["media"]
    }
  }
]
```

### Sync Rules

Files are synchronized based on the following rules:

1. Missing files in destination are downloaded
2. Files are updated if any of these metadata changes:
   - File size
   - Modified timestamp
   - Permissions
   - MD5 hash
3. Source path resolution:
   - If `base-url` is provided, relative paths are resolved against it
   - Absolute URLs are used as-is
4. Authentication:
   - No auth by default
   - Basic auth when both username and password are provided

### Log Management

- Default maximum log files: 100
- Configurable via `--max-logs`
- Oldest logs are deleted when limit is reached
- Log format includes:
  - Timestamp
  - Operation type
  - File details
  - Success/failure status
  - Error messages if applicable

### Reports and Logs

The tool generates both CSV and JSON formats for all reports and logs:

1. Sync Operation Logs (`sync_log_YYYYMMDD_HHMMSS.csv`):
   ```csv
   sync_id,timestamp,operation,file_path,file_size,status,error,source,destination,md5,config_id
   550e8400,2025-01-31T03:26:28Z,add,example.jpg,178567,success,,https://example.com,/path/to/dest,5fe94c2b9ecab79578630eb259eaf8a8,config1
   ```

2. Asset Listings (`assets_YYYYMMDD_HHMMSS.csv`):
   ```csv
   id,filename,path,size,type,sync_status,last_sync,error
   48,example.jpg,/images/,178567,image/jpeg,success,2025-01-31T03:26:28Z,
   ```

### Documentation

The CLI provides structured documentation in CSV format. Generate documentation using:

```bash
cli-file-sync docs --output docs
```

This creates:
1. Commands documentation (`commands.csv`)
2. Configuration options (`config.csv`)
3. Error codes and solutions (`errors.csv`)
4. JSON schemas (`schemas.csv`)

Example schema documentation:
```csv
id,name,description,schema_example,category
schema_asset,Asset,File asset definition,{"fid":"48","filename":"example.jpg"},Core
```

### Role-Based Documentation

Documentation is organized by user roles:

1. **Operators**: Day-to-day sync operations
   - Basic commands
   - Common troubleshooting
   - Configuration management

2. **Administrators**: System setup and maintenance
   - Advanced configuration
   - Security settings
   - Performance tuning

3. **Developers**: Integration and customization
   - API documentation
   - JSON schemas
   - Extension points

### Documentation Setup

1. Generate documentation:
   ```bash
   cli-file-sync docs --output docs
   ```

2. Import CSV files into your preferred documentation system

3. Set up access controls based on roles:
   - Operators: Basic commands, troubleshooting
   - Administrators: Full system documentation
   - Developers: Technical specifications

### Best Practices

1. Regular documentation updates:
   ```bash
   cli-file-sync docs --output docs
   ```

2. Version control documentation files

3. Maintain role-based access control

4. Keep documentation in sync with deployed version

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For bug reports and feature requests, please open an issue on GitHub.
