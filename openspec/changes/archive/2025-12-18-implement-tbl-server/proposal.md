## Why

The `tbl` project currently has no implementation - only design documentation exists from a ChatGPT conversation. This proposal bootstraps the entire codebase to deliver the core self-bootstrapping web launcher that can clone a web UI from Git and serve it with secure cookie-based authentication.

## What Changes

- **ADDED** Rust project structure with Cargo.toml and dependencies
- **ADDED** CLI argument parsing with clap (--git-url, --addr, --tls-*, --basic-*, --no-browser)
- **ADDED** Multi-format config loading (JSON, YAML, TOML) with precedence: CLI > ENV > file > defaults
- **ADDED** Core Axum web server with routes: `/`, `/bootstrap`, `/setup`, `/web/*`, `/tbl.js`, `/api/v1/ping`
- **ADDED** Cookie-based authentication with per-run random token
- **ADDED** Optional HTTP Basic auth layer
- **ADDED** Optional TLS support via rustls
- **ADDED** Git integration: auto-detect `git`, shallow clone, pull on restart
- **ADDED** OS-specific install hints when `git` is missing
- **ADDED** Daemon lifecycle: re-exec for background mode, pid.yaml tracking, port auto-detection
- **ADDED** Embedded HTML/CSS/JS for setup page and bootstrap page
- **ADDED** Makefile for static binary builds

## Impact

- Affected specs: `core-server`, `config-management`, `git-integration`, `auth-security`, `daemon-lifecycle` (all new)
- Affected code: Creates entire `src/main.rs`, `Cargo.toml`, `Makefile`
- No breaking changes (greenfield implementation)

