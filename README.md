# CLI File Sync

A command-line tool to sync files from a remote source using JSON metadata.

## How It Works

This tool implements a lightweight, HTTP-based file synchronization mechanism that doesn't require any server-side daemon or special sync software. Here's how it works:

1. **Metadata Source**: 
   - The tool reads a JSON metadata file from a specified HTTP endpoint
   - This metadata contains information about files: paths, sizes, timestamps, etc.
   - The metadata can be hosted anywhere accessible via HTTP/HTTPS

2. **Delta Detection**:
   - Compares the remote metadata with local state
   - Identifies new, modified, or deleted files
   - Only downloads files that have changed

3. **HTTP Download**:
   - Files are downloaded directly via HTTP/HTTPS
   - Supports concurrent downloads for better performance
   - Handles authentication (Basic Auth, Token)
   - Implements retry logic and timeout handling

4. **State Management**:
   - Maintains local state to track synced files
   - Records failed downloads for retry
   - Preserves file metadata for future comparisons

This approach makes the tool ideal for:
- Syncing with static file servers or CDNs
- Downloading assets from content management systems
- Maintaining local copies of web-hosted resources
- Automated asset synchronization in CI/CD pipelines

## Requirements

- Rust 1.70 or higher
- Cargo (Rust's package manager)

### Installing Rust and Cargo

The easiest way to install Rust and Cargo is using rustup:

#### On macOS or Linux:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### On Windows:
1. Download rustup-init.exe from https://rustup.rs
2. Run the installer

After installation:
1. Restart your terminal
2. Verify installation:
```bash
rustc --version  # Should show 1.70 or higher
cargo --version
```

## Installation

1. Clone the repository:
```bash
git clone https://github.com/althafhpa/cli-file-sync.git
cd cli-file-sync
```

2. Build the project:
```bash
cargo build --release
```

## Quick Start

```bash
cargo run -- sync --assets-metadata https://example.com/assets-metadata.json --destination downloads --base-url https://example.com/ --max-concurrent 4
```

## Metadata Format

The tool expects a JSON metadata file that describes the files to be synced. The metadata should follow this structure:

```json
[
  {
    "id": "1",
    "filename": "example.jpg",
    "uri": "public://example.jpg",
    "path": "path/to/file/example.jpg",
    "mime": "image/jpeg",
    "size": 12345,
    "created": 1612345678,
    "changed": 1612345678,
    "scheme": "public"
  }
]
```

Each file entry contains:
- `filename`: Original filename
- `uri`: URI scheme and path
- `path`: Relative path where the file should be stored
- `mime`: MIME type of the file
- `size`: File size in bytes
- `created`/`changed`: Timestamps for file creation and modification
- `scheme`: URI scheme (e.g., "public", "private")

## Configuration

The tool can be configured via command-line arguments or a configuration file. Key configuration options:

```bash
# Base configuration
cli-file-sync config --base-url "https://example.com/files" --dest-path "./downloads"

# Download settings
cli-file-sync config --download-delay 100 --download-timeout 30 --max-retries 3

# Authentication
cli-file-sync config --source-username "user" --source-password "pass"
```

Key configuration options:
- `base-url`: Base URL for file downloads
- `dest-path`: Local destination for downloaded files
- `download-delay`: Delay between downloads (ms)
- `download-timeout`: Download timeout (seconds)
- `max-retries`: Maximum retry attempts for failed downloads
- Authentication credentials for both metadata and file downloads

## Options

| Option | Description | Example |
|--------|-------------|---------|
| `--assets-metadata` | Path to JSON metadata file or URL | `https://example.com/assets-metadata.json` or `local/path/assets.json` |
| `--destination` | Directory where files will be downloaded | `downloads` |
| `--base-url` | Base URL for resolving relative file paths | `https://example.com/` |
| `--max-concurrent` | Maximum number of concurrent downloads | `4` |
| `--source-username` | Username for metadata source (optional) | `admin` |
| `--source-password` | Password for metadata source (optional) | `password123` |

## Example Files

### Sample assets-metadata.json

```json
{
    "version": "1.0",
    "generated": 1706919581,
    "source": {
        "type": "drupal",
        "version": "10.3.10"
    },
    "files": [
        {
            "id": "1",
            "filename": "sample.pdf",
            "uri": "/files/sample.pdf",
            "path": "documents/sample.pdf",
            "mime": "application/pdf",
            "size": 1024,
            "created": 1706919581,
            "changed": 1706919581,
            "scheme": "public"
        },
        {
            "id": "2",
            "filename": "image.jpg",
            "uri": "/files/image.jpg",
            "path": "images/image.jpg",
            "mime": "image/jpeg",
            "size": 2048,
            "created": 1706919581,
            "changed": 1706919581,
            "scheme": "public"
        }
    ]
}
```

### Base URL and File Resolution

The tool combines the `base-url` with each file's `uri` to create the download URL. For example:

- Base URL: `https://example.com`
- File URI: `/files/sample.pdf`
- Final Download URL: `https://example.com/files/sample.pdf`

Files will be saved to the destination directory preserving their paths:

- Destination: `downloads`
- File path: `documents/sample.pdf`
- Final local path: `downloads/documents/sample.pdf`
