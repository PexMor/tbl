<!-- OPENSPEC:START -->

# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:

- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:

- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

---

# Architecture & Design Guide

This document provides architectural context for developers and AI assistants working on tbl.

## Project Overview

**tbl** is a self-bootstrapping web launcher written in Rust. It serves web UIs cloned from Git repositories with built-in authentication.

## Tech Stack

| Component     | Technology                   | Purpose                |
| ------------- | ---------------------------- | ---------------------- |
| Language      | Rust 2021                    | Core implementation    |
| Web Framework | Axum 0.7                     | HTTP routing, handlers |
| Async Runtime | Tokio                        | Multi-threaded async   |
| Static Files  | tower-http ServeDir          | Serving cloned web UI  |
| TLS           | axum-server + rustls         | Optional HTTPS         |
| CLI           | clap 4.x (derive)            | Argument parsing       |
| Config        | serde_json, serde_yaml, toml | Multi-format support   |

## Key Architecture Decisions

### Single Binary Design

All HTML/CSS/JS is embedded as string literals. No external assets or build step. This enables:

- Zero runtime dependencies (except `git`)
- Simple deployment
- Cross-platform builds via MUSL

### Configuration Precedence

```
CLI flags > Environment variables > Config file > Defaults
```

Config stored in `~/.config/tbl/`. Supports JSON, YAML, and TOML for reading; writes JSON.

### Security Model

- **Per-session tokens**: Random 32-byte hex token generated on each run
- **Cookie-based auth**: Token set via JavaScript at `/bootstrap`
- **Localhost-first**: Default bind to 127.0.0.1
- **Optional layers**: HTTP Basic auth, TLS

### Daemon Lifecycle

1. Parent re-executes itself with `TBL_DAEMONIZED=1`
2. Parent exits immediately
3. Child writes `pid.yaml` with port, token, TLS flag
4. Subsequent runs detect existing daemon and reuse it

### Port Auto-detection

Starts at configured port (default 1234), increments until finding available one. Stores actual port in `pid.yaml`.

## File Structure

```
~/.config/tbl/
├── config.json          # Persisted configuration
├── run/
│   └── pid.yaml         # Runtime state (pid, port, token)
└── web/                  # Cloned Git repository
```

## Code Organization

```
src/
└── main.rs              # All code in single file
    ├── CLI struct       # clap-derived arguments
    ├── TblConfig        # Configuration model
    ├── AppState         # Shared server state
    ├── RunInfo          # pid.yaml model
    ├── main()           # Startup orchestration
    ├── Handlers         # HTTP route handlers
    └── Helpers          # Git, auth, config utilities
```

## Specifications

Detailed requirements are in `openspec/specs/`:

| Spec                | Coverage                                      |
| ------------------- | --------------------------------------------- |
| `auth-security`     | Token generation, cookie auth, basic auth     |
| `config-management` | Multi-format loading, precedence, persistence |
| `core-server`       | Routes, static files, embedded UI             |
| `daemon-lifecycle`  | Startup, shutdown, port detection             |
| `git-integration`   | Clone, update, availability detection         |

## Development Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run                # Run locally
make static              # Static MUSL binary (Linux)
./target/release/tbl --stop  # Stop running daemon
```

## Related Documents

- [docs/architecture.md](docs/architecture.md) — Detailed design rationale
- [docs/configuration.md](docs/configuration.md) — Full config reference
- [docs/api.md](docs/api.md) — HTTP endpoints and JS SDK
- [openspec/project.md](openspec/project.md) — Domain context
