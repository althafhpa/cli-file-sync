# TODO List for CLI File Sync

## 1. Missing Commands to Implement

### Check Command
```bash
cli-file-sync check \
  --assets-metadata <URL_OR_PATH> \
  [--destination <PATH>]
```
Purpose: Verify file integrity and sync status without downloading

### List Assets Command
```bash
cli-file-sync list \
  --assets-metadata <URL_OR_PATH> \
  --output <PATH> \
  --base-url <URL>
```
Features to implement:
- Generate CSV and JSON asset listings
- Include full download URLs
- Create thumbnails for image files
- Output metadata (name, size, type, paths, URLs)

## 2. Config Command Options to Add

Current implementation only has:
- `--base-url`
- `--destination`

Need to add these options:
```bash
cli-file-sync config \
  [--max-logs <NUMBER>]           # Maximum number of log files to keep (default: 10)
  [--max-concurrent 5]            # Default concurrent downloads
  [--download-delay 100]          # Default delay between downloads (ms)
  [--download-timeout 30]         # Default download timeout (seconds)
  [--max-retries 3]              # Default retry attempts
  [--max-file-size 104857600]    # Default max file size (100 MB)
```

Implementation details:
1. Log Management:
   - Default to keeping last 10 log files
   - When max_logs is reached, delete oldest log before creating new one
   - Allow configuration via config command
   - Store logs in ~/.config/cli-file-sync/logs/
   - Log format: sync_YYYYMMDD_HHMMSS.log

2. File Size Limit:
   - Default max file size: 100 MB (104857600 bytes)
   - Skip files larger than limit with warning
   - Allow override via config
   - Document size in human-readable format (MB)

3. Download Settings:
   - max_concurrent: Number of parallel downloads
   - download_delay: Milliseconds between starting downloads
   - download_timeout: Seconds before download times out
   - max_retries: Number of retry attempts for failed downloads

4. Configuration Storage:
   - Store in ~/.config/cli-file-sync/config.json
   - Load on startup
   - Update via config command
   - Validate all numeric values > 0

## 3. Environment Variables Support

Need to implement support for these environment variables:
```bash
CLI_SYNC_SOURCE_USER="username"    # Default source username for authentication
CLI_SYNC_SOURCE_PASS="password"    # Default source password for authentication
```

Implementation details:
1. Read environment variables in get_default_auth()
2. Use them as fallback when no credentials are provided via CLI
3. Add proper error handling for malformed environment variables
4. Add tests for environment variable functionality

## 4. Implementation Notes
- These features are planned for future releases
- Current focus is on stabilizing the core sync functionality
- Remove these features from README until they are implemented
