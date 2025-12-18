# Configuration

tbl supports multiple configuration sources with the following precedence (highest to lowest):

1. **CLI flags** — Command-line arguments
2. **Environment variables** — `TBL_*` prefixed
3. **Config file** — `~/.config/tbl/config.{json,yaml,toml}`
4. **Defaults** — Built-in values

## CLI Flags

| Flag                  | Description                   | Default          |
| --------------------- | ----------------------------- | ---------------- |
| `--git-url <URL>`     | Git repository URL for web UI | —                |
| `--addr <HOST:PORT>`  | Bind address                  | `127.0.0.1:1234` |
| `--tls-cert <PATH>`   | TLS certificate file (PEM)    | —                |
| `--tls-key <PATH>`    | TLS private key file (PEM)    | —                |
| `--basic-user <USER>` | HTTP Basic auth username      | —                |
| `--basic-pass <PASS>` | HTTP Basic auth password      | —                |
| `--no-browser`        | Don't auto-open browser       | `false`          |
| `--stop`              | Stop running tbl server       | —                |

### Examples

```bash
# Basic usage with Git URL
tbl --git-url https://github.com/you/web-ui.git

# Custom port
tbl --addr 127.0.0.1:8080

# With TLS
tbl --tls-cert cert.pem --tls-key key.pem

# With HTTP Basic auth
tbl --basic-user admin --basic-pass secret

# Stop running server
tbl --stop
```

## Environment Variables

| Variable         | Description              |
| ---------------- | ------------------------ |
| `TBL_GIT_URL`    | Git repository URL       |
| `TBL_ADDR`       | Bind address (host:port) |
| `TBL_TLS_CERT`   | Path to TLS certificate  |
| `TBL_TLS_KEY`    | Path to TLS private key  |
| `TBL_BASIC_USER` | HTTP Basic auth username |
| `TBL_BASIC_PASS` | HTTP Basic auth password |

### Example

```bash
export TBL_GIT_URL=https://github.com/you/web-ui.git
export TBL_ADDR=127.0.0.1:3000
tbl
```

## Config File

tbl reads configuration from `~/.config/tbl/` in these formats (first found wins):

1. `config.json`
2. `config.yaml` / `config.yml`
3. `config.toml`

### JSON Example

```json
{
  "git_url": "https://github.com/you/web-ui.git",
  "addr": "127.0.0.1:1234",
  "tls_cert": null,
  "tls_key": null,
  "basic_user": null,
  "basic_pass": null
}
```

### YAML Example

```yaml
git_url: https://github.com/you/web-ui.git
addr: 127.0.0.1:1234
# tls_cert: /path/to/cert.pem
# tls_key: /path/to/key.pem
```

### TOML Example

```toml
git_url = "https://github.com/you/web-ui.git"
addr = "127.0.0.1:1234"
```

## Directory Structure

```
~/.config/tbl/
├── config.json          # Configuration (auto-created)
├── run/
│   └── pid.yaml         # Runtime state
└── web/                  # Cloned Git repository
```

### Runtime State (`pid.yaml`)

Written on startup, contains:

```yaml
pid: 12345
port: 1234
auth_token: abc123...
tls: false
```

Used for:

- Detecting existing running instances
- Providing auth token for `--stop` command
- Browser redirect to correct port
