# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-18

### Added

- Initial release of tbl (Tiny Bootstrapping Launcher)
- Core Axum web server with routes: `/`, `/bootstrap`, `/setup`, `/web/*`, `/tbl.js`, `/api/v1/ping`, `/api/v1/shutdown`
- Cookie-based authentication with per-run random tokens
- Optional HTTP Basic auth layer
- Optional TLS support via rustls
- Multi-format configuration support (JSON, YAML, TOML)
- Configuration precedence: CLI > ENV > config file > defaults
- Git integration with shallow cloning and auto-update on restart
- OS-specific Git installation hints (macOS, Windows, Linux)
- Daemon mode with automatic backgrounding
- Port auto-detection starting from configured base port
- Existing instance detection via `~/.config/tbl/run/pid.yaml`
- Browser auto-open with authentication URL
- `--stop` flag to gracefully stop running daemon
- Verbose startup output with banner and URL display
- Embedded HTML/CSS/JS for setup and bootstrap pages
- JavaScript SDK at `/tbl.js` for API calls
- Makefile for static binary builds (MUSL target)

[Unreleased]: https://github.com/yourusername/tbl/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/tbl/releases/tag/v0.1.0
