# CLI File Sync

A command-line tool to sync files from a remote source using JSON metadata.

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
git clone https://github.com/example/cli-file-sync.git
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
