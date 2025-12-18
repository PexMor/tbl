# tbl

**Tiny Bootstrapping Launcher** — A self-service, self-modifying web server that clones and serves web UIs from Git repositories with secure cookie-based authentication.

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/tbl

# Or with a Git URL directly
./target/release/tbl --git-url https://github.com/you/your-web-ui.git
```

The server will:

1. Auto-detect an available port (starting from 1234)
2. Open your browser with authentication
3. Serve your web UI at `http://127.0.0.1:<port>/web/`

## Features

- 🔐 **Secure by default** — Cookie-based auth with per-session tokens
- 📦 **Single binary** — No runtime dependencies (except `git`)
- 🔄 **Self-bootstrapping** — Configure via CLI, env vars, config files, or web form
- 🚀 **Daemon mode** — Runs in background, detects existing instances
- 🔒 **Optional TLS** — HTTPS support via rustls

## Documentation

| Document                               | Description                                  |
| -------------------------------------- | -------------------------------------------- |
| [Installation](docs/installation.md)   | Build instructions, platforms, static binary |
| [Configuration](docs/configuration.md) | CLI flags, env vars, config files            |
| [Architecture](docs/architecture.md)   | Design decisions, security model, routes     |
| [API Reference](docs/api.md)           | HTTP endpoints, JavaScript SDK               |

## CLI Reference

```
tbl [OPTIONS]

Options:
      --git-url <URL>      Git repository URL for web UI
      --addr <ADDR>        Bind address (default: 127.0.0.1:1234)
      --tls-cert <PATH>    TLS certificate file (PEM)
      --tls-key <PATH>     TLS private key file (PEM)
      --basic-user <USER>  HTTP Basic auth username
      --basic-pass <PASS>  HTTP Basic auth password
      --no-browser         Don't auto-open browser
      --stop               Stop a running tbl server
  -h, --help               Print help
  -V, --version            Print version
```

## License

[MIT](LICENSE)
