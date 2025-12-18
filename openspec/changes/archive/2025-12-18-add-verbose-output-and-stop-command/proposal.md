## Why

The current tbl implementation has minimal console output, making it unclear to users:
- What URL to open for authentication
- Whether a daemon is already running
- How to stop a running server

Users need clearer feedback about server state and a way to gracefully stop a running daemon without killing processes manually.

## What Changes

- **MODIFIED** Console output to be more verbose and user-friendly
- **MODIFIED** Daemon detection to always print the URL prominently
- **MODIFIED** Browser opening behavior: always open when daemon found (unless `--no-browser`)
- **ADDED** `--stop` CLI argument to gracefully stop a running daemon
- **ADDED** `/api/v1/shutdown` endpoint (cookie-authenticated) for remote shutdown

## Impact

- Affected specs: `daemon-lifecycle` (modified)
- Affected code: `src/main.rs` - CLI struct, main function, new handler
- Non-breaking: All existing behavior preserved, new features are additive

