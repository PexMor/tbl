# Project Context

## Purpose

**tbl** (Tiny Bootstrapping Launcher) is a self-service, self-modifying web server that:

- Starts a local web server with cookie-based authentication
- Bootstraps itself by asking for a Git URL (via CLI, env, config file, or web form)
- Shallow-clones a web UI from the provided Git repository into `~/.config/tbl/web/`
- Serves that cloned web UI securely with automatic browser authentication
- Provides an extensible `/api/v1/...` endpoint for future functionality
- Builds as a single static binary

## Tech Stack

- **Language**: Rust (2021 edition)
- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio (multi-threaded)
- **Static File Serving**: tower-http (ServeDir)
- **TLS**: axum-server with rustls
- **CLI Parsing**: clap 4.x (derive)
- **Config Formats**: JSON (serde_json), YAML (serde_yaml), TOML (toml)
- **Build Target**: Single static binary via musl (x86_64-unknown-linux-musl)

## Project Conventions

### Code Style

- Standard Rust formatting via `rustfmt`
- Use `anyhow` for error handling with context
- Prefer `Arc<AppState>` for shared state in Axum handlers
- Embedded HTML/CSS/JS as raw string literals for self-contained binary

### Architecture Patterns

- **Configuration Precedence**: CLI > Environment Variables > Config File > Defaults
- **Config Location**: `~/.config/tbl/` (uses `dirs` crate for cross-platform support)
- **Config Formats Supported**: `config.json`, `config.yaml`, `config.yml`, `config.toml`
- **Runtime State**: `~/.config/tbl/run/pid.yaml` (PID, port, auth token, TLS flag)
- **Web Content**: `~/.config/tbl/web/` (shallow-cloned Git repository)

### Security Model

- Per-run random auth token (32 bytes, hex-encoded)
- Cookie-based authentication (`tbl_token`)
- Bootstrap flow: auto-open browser with `?token=...` → JS sets cookie → redirect
- Optional HTTP Basic auth layer on top of cookie auth
- Optional TLS with PEM certificate/key

### Testing Strategy

- Manual testing via `cargo run`
- Static binary testing via `make static`
- Browser-based integration testing for auth flow

### Git Workflow

- Feature branches for new capabilities
- Single `main` branch for stable releases

## Domain Context

### Startup Flow

1. Check for existing daemon via `~/.config/tbl/run/pid.yaml`
   - If running: open new browser context with stored token, exit
   - If stale: remove pid.yaml, continue startup
2. Daemonize (re-exec with `TBL_DAEMONIZED=1`, parent exits)
3. Load config (CLI > ENV > file > defaults)
4. Auto-detect available port starting from configured base (default: 1234)
5. If Git URL known: ensure `git` is available, clone/update repo
6. Generate auth token, write `pid.yaml`
7. Start HTTP(S) server
8. Auto-open browser to `/bootstrap?token=...`

### Key Routes

| Route | Method | Description |
|-------|--------|-------------|
| `/` | GET | Redirect to `/web/` if cloned, else show setup form |
| `/bootstrap` | GET | Validate token, set cookie via JS, redirect to `/` |
| `/setup` | POST | Clone Git repo, save config, redirect to `/` |
| `/web/*` | GET | Serve static files from cloned repo |
| `/tbl.js` | GET | JS SDK for API calls (auto-includes credentials) |
| `/api/v1/ping` | GET | Health check (requires valid cookie + optional basic auth) |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `TBL_GIT_URL` | Git repository URL for web UI |
| `TBL_ADDR` | Bind address (host:port) |
| `TBL_TLS_CERT` | Path to TLS certificate (PEM) |
| `TBL_TLS_KEY` | Path to TLS private key (PEM) |
| `TBL_BASIC_USER` | HTTP Basic auth username |
| `TBL_BASIC_PASS` | HTTP Basic auth password |

### CLI Flags

- `--git-url` - Git repository URL
- `--addr` - Bind address (default: `127.0.0.1:1234`)
- `--tls-cert` / `--tls-key` - TLS certificate and key paths
- `--basic-user` / `--basic-pass` - HTTP Basic auth credentials
- `--no-browser` - Don't auto-open browser

## Important Constraints

- Must build as a single static binary (no external dependencies at runtime except `git`)
- Must work on macOS, Windows, and Linux
- `git` must be available on PATH for cloning (with OS-specific install hints if missing)
- Localhost-first security model (cookie valid for local session only)

## External Dependencies

- **Git**: Required for cloning web UI repositories (detected at runtime)
- **Web Browser**: For initial authentication bootstrap (opened automatically)
- **User's Git Repository**: Contains the web UI to be served (user-provided)
