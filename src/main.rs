use anyhow::{Context, Result};
use axum::{
    extract::{Form, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clap::Parser;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    future::IntoFuture,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};
use tokio::sync::oneshot;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

// =============================================================================
// CLI Arguments
// =============================================================================

#[derive(Parser, Debug)]
#[command(name = "tbl", version, about = "Tiny self-bootstrapping web launcher")]
struct Cli {
    /// Git URL of the web app to serve
    #[arg(long)]
    git_url: Option<String>,

    /// Address to bind to (e.g. 127.0.0.1:1234)
    /// The port is auto-detected starting from the specified value.
    #[arg(long)]
    addr: Option<String>,

    /// TLS certificate file (PEM format)
    #[arg(long)]
    tls_cert: Option<String>,

    /// TLS private key file (PEM format)
    #[arg(long)]
    tls_key: Option<String>,

    /// HTTP Basic auth username
    #[arg(long)]
    basic_user: Option<String>,

    /// HTTP Basic auth password
    #[arg(long)]
    basic_pass: Option<String>,

    /// Do not auto-open the browser
    #[arg(long)]
    no_browser: bool,

    /// Stop a running tbl server
    #[arg(long)]
    stop: bool,
}

// =============================================================================
// Configuration
// =============================================================================

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct TblConfig {
    git_url: Option<String>,
    addr: Option<String>,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    basic_user: Option<String>,
    basic_pass: Option<String>,
}

// =============================================================================
// Application State
// =============================================================================

struct AppState {
    auth_token: String,
    web_root: PathBuf,
    config_dir: PathBuf,
    config: TblConfig,
    shutdown_tx: tokio::sync::Mutex<Option<oneshot::Sender<()>>>,
}

// =============================================================================
// Runtime Info (pid.yaml)
// =============================================================================

#[derive(Serialize, Deserialize, Debug)]
struct RunInfo {
    pid: u32,
    port: u16,
    auth_token: String,
    tls: bool,
}

// =============================================================================
// Request/Response Types
// =============================================================================

#[derive(Deserialize)]
struct BootstrapQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
struct SetupForm {
    git_url: String,
}

#[derive(Serialize)]
struct PingResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ShutdownResponse {
    status: &'static str,
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --stop before daemonization
    if cli.stop {
        return handle_stop_command();
    }

    // Daemonize: re-exec in background if not already daemonized
    if std::env::var("TBL_DAEMONIZED").is_err() {
        // Print banner before daemonizing so user sees it
        print_banner();

        let exe = std::env::current_exe().context("cannot get current exe")?;
        let mut cmd = Command::new(exe);
        cmd.args(std::env::args().skip(1));
        cmd.env("TBL_DAEMONIZED", "1");
        cmd.stdin(Stdio::null())
            .stdout(Stdio::inherit())  // Keep stdout for verbose output
            .stderr(Stdio::inherit()); // Keep stderr for errors
        let _child = cmd.spawn().context("failed to spawn tbl daemon")?;
        // Parent exits; daemon continues
        return Ok(());
    }

    // Determine config directory: ~/.config/tbl
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create config dir {:?}", config_dir))?;

    // Change working directory to config dir
    std::env::set_current_dir(&config_dir)
        .with_context(|| format!("Failed to chdir to {:?}", config_dir))?;

    // Load config file if present (JSON, YAML, or TOML)
    let file_cfg = load_config(&config_dir).unwrap_or_default();

    // Environment variables
    let env_git_url = std::env::var("TBL_GIT_URL").ok();
    let env_addr = std::env::var("TBL_ADDR").ok();
    let env_tls_cert = std::env::var("TBL_TLS_CERT").ok();
    let env_tls_key = std::env::var("TBL_TLS_KEY").ok();
    let env_basic_user = std::env::var("TBL_BASIC_USER").ok();
    let env_basic_pass = std::env::var("TBL_BASIC_PASS").ok();

    // Merge configuration with precedence: CLI > ENV > config file > defaults
    let mut effective_cfg = TblConfig {
        git_url: cli.git_url.clone().or(env_git_url).or(file_cfg.git_url),
        addr: cli
            .addr
            .clone()
            .or(env_addr)
            .or(file_cfg.addr)
            .or(Some("127.0.0.1:1234".to_string())),
        tls_cert: cli.tls_cert.clone().or(env_tls_cert).or(file_cfg.tls_cert),
        tls_key: cli.tls_key.clone().or(env_tls_key).or(file_cfg.tls_key),
        basic_user: cli
            .basic_user
            .clone()
            .or(env_basic_user)
            .or(file_cfg.basic_user),
        basic_pass: cli
            .basic_pass
            .clone()
            .or(env_basic_pass)
            .or(file_cfg.basic_pass),
    };

    let tls_enabled = effective_cfg.tls_cert.is_some() && effective_cfg.tls_key.is_some();

    // Check for already-running daemon via pid.yaml
    let run_dir = config_dir.join("run");
    let maybe_run_info = load_run_info(&run_dir);

    if let Some(info) = maybe_run_info {
        if port_is_open(info.port) {
            // Server already running; open new browser context
            let scheme = if info.tls { "https" } else { "http" };
            let public_url = format!(
                "{}://127.0.0.1:{}/bootstrap?token={}",
                scheme, info.port, info.auth_token
            );

            println!();
            println!("  tbl is already running");
            println!("  ───────────────────────────────────────");
            println!("  PID:    {}", info.pid);
            println!("  Port:   {}", info.port);
            println!("  TLS:    {}", if info.tls { "enabled" } else { "disabled" });
            println!();
            print_url_box(&public_url);

            if !cli.no_browser {
                println!("\n  Opening browser...");
                if let Err(e) = webbrowser::open(&public_url) {
                    eprintln!("  Failed to open browser: {e}");
                    eprintln!("  Open the URL above manually to authenticate.");
                }
            } else {
                println!("\n  Open the URL above to authenticate.");
            }
            println!();

            return Ok(());
        } else {
            // Stale pid.yaml; remove it
            clear_run_info(&run_dir);
        }
    }

    // If git URL is known, ensure git is available and repo is present
    if effective_cfg.git_url.is_some() {
        ensure_git_available()?;
    }

    let web_root = config_dir.join("web");

    if let Some(ref url) = effective_cfg.git_url {
        ensure_repo(&config_dir, url)
            .with_context(|| format!("Failed to ensure repo for URL {url}"))?;
    }

    // Generate a per-run secret token
    let auth_token = generate_token();

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let state = Arc::new(AppState {
        auth_token: auth_token.clone(),
        web_root: web_root.clone(),
        config_dir: config_dir.clone(),
        config: effective_cfg.clone(),
        shutdown_tx: tokio::sync::Mutex::new(Some(shutdown_tx)),
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/bootstrap", get(bootstrap_handler))
        .route("/setup", post(setup_handler))
        .route("/api/v1/ping", get(ping_handler))
        .route("/api/v1/shutdown", post(shutdown_handler))
        .route("/tbl.js", get(tbl_js_handler))
        .nest_service("/web", ServeDir::new(&web_root))
        .with_state(state.clone());

    // Port auto-detection starting at configured base port
    let addr_template = effective_cfg.addr.clone().unwrap();
    let (host, base_port) = split_host_port(&addr_template)?;
    let chosen_port = find_available_port(&host, base_port);
    let addr: SocketAddr = format!("{}:{}", host, chosen_port)
        .parse()
        .with_context(|| format!("Invalid addr: {}:{}", host, chosen_port))?;

    // Update effective config with chosen port
    effective_cfg.addr = Some(format!("{}:{}", host, chosen_port));

    // Save config
    if let Err(e) = save_config(&config_dir, &effective_cfg) {
        eprintln!("Failed to save config: {e}");
    }

    let scheme = if tls_enabled { "https" } else { "http" };

    // Write pid.yaml for future instance detection
    let run_info = RunInfo {
        pid: std::process::id(),
        port: chosen_port,
        auth_token: auth_token.clone(),
        tls: tls_enabled,
    };
    if let Err(e) = save_run_info(&run_dir, &run_info) {
        eprintln!("Failed to write pid.yaml: {e}");
    }

    let public_url = format!(
        "{}://127.0.0.1:{}/bootstrap?token={}",
        scheme, chosen_port, auth_token
    );

    // Verbose startup output
    println!();
    println!("  Starting tbl server...");
    println!("  ───────────────────────────────────────");
    println!("  Address: {}://{}", scheme, addr);
    println!("  TLS:     {}", if tls_enabled { "enabled" } else { "disabled" });
    println!("  PID:     {}", std::process::id());
    println!();
    print_url_box(&public_url);

    if !cli.no_browser {
        println!("\n  Opening browser...");
        if let Err(e) = webbrowser::open(&public_url) {
            eprintln!("  Failed to open browser: {e}");
            eprintln!("  Open the URL above manually to authenticate.");
        }
    } else {
        println!("\n  Open the URL above to authenticate.");
    }
    println!();

    // Store run_dir for cleanup on shutdown
    let run_dir_clone = run_dir.clone();

    if tls_enabled {
        let cert = effective_cfg.tls_cert.clone().unwrap();
        let key = effective_cfg.tls_key.clone().unwrap();
        let tls_config = RustlsConfig::from_pem_file(cert, key)
            .await
            .context("failed to load TLS cert/key")?;

        let server = axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service());

        tokio::select! {
            result = server => {
                if let Err(e) = result {
                    eprintln!("Server error: {e}");
                }
            }
            _ = shutdown_rx => {
                println!("  Shutdown signal received, stopping server...");
            }
        }
    } else {
        let listener = TcpListener::bind(addr).await?;
        let server = axum::serve(listener, app);

        tokio::select! {
            result = server.into_future() => {
                if let Err(e) = result {
                    eprintln!("Server error: {e}");
                }
            }
            _ = shutdown_rx => {
                println!("  Shutdown signal received, stopping server...");
            }
        }
    }

    // Cleanup pid.yaml on shutdown
    clear_run_info(&run_dir_clone);
    println!("  tbl server stopped.");

    Ok(())
}

// =============================================================================
// Configuration Helpers
// =============================================================================

fn get_config_dir() -> Result<PathBuf> {
    if let Some(base) = dirs::config_dir() {
        Ok(base.join("tbl"))
    } else {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(Path::new(&home).join(".config").join("tbl"))
    }
}

fn load_config(config_dir: &Path) -> Option<TblConfig> {
    let candidates = [
        ("config.json", "json"),
        ("config.yaml", "yaml"),
        ("config.yml", "yaml"),
        ("config.toml", "toml"),
    ];

    for (file, kind) in candidates {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let parsed = match kind {
                "json" => serde_json::from_str::<TblConfig>(&content).ok(),
                "yaml" => serde_yaml::from_str::<TblConfig>(&content).ok(),
                "toml" => toml::from_str::<TblConfig>(&content).ok(),
                _ => None,
            };
            if let Some(cfg) = parsed {
                return Some(cfg);
            }
        }
    }
    None
}

fn save_config(config_dir: &Path, cfg: &TblConfig) -> Result<()> {
    let path = config_dir.join("config.json");
    let json = serde_json::to_vec_pretty(cfg)?;
    fs::write(path, json)?;
    Ok(())
}

// =============================================================================
// Runtime Info Helpers
// =============================================================================

fn load_run_info(run_dir: &Path) -> Option<RunInfo> {
    let path = run_dir.join("pid.yaml");
    let content = fs::read_to_string(path).ok()?;
    serde_yaml::from_str::<RunInfo>(&content).ok()
}

fn save_run_info(run_dir: &Path, info: &RunInfo) -> Result<()> {
    fs::create_dir_all(run_dir)?;
    let yaml = serde_yaml::to_string(info)?;
    fs::write(run_dir.join("pid.yaml"), yaml)?;
    Ok(())
}

fn clear_run_info(run_dir: &Path) {
    let _ = fs::remove_file(run_dir.join("pid.yaml"));
}

// =============================================================================
// Port Detection
// =============================================================================

fn port_is_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

fn split_host_port(addr: &str) -> Result<(String, u16)> {
    if let Some(pos) = addr.rfind(':') {
        let (host, port_str) = addr.split_at(pos);
        let host = host.to_string();
        let port_str = &port_str[1..];
        let port: u16 = port_str
            .parse()
            .with_context(|| format!("invalid port in addr: {addr}"))?;
        Ok((host, port))
    } else {
        anyhow::bail!("addr must be in host:port form, got: {addr}");
    }
}

fn find_available_port(host: &str, base_port: u16) -> u16 {
    let mut port = base_port;
    for _ in 0..100 {
        let addr_str = format!("{host}:{port}");
        if let Ok(sock_addr) = addr_str.parse::<SocketAddr>() {
            if TcpStream::connect_timeout(&sock_addr, Duration::from_millis(150)).is_err() {
                return port;
            }
        }
        port = port.saturating_add(1);
    }
    base_port
}

// =============================================================================
// Git Integration
// =============================================================================

fn ensure_git_available() -> Result<()> {
    let status = Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            eprintln!("`git` was not found or is not working.");

            let os = std::env::consts::OS;
            match os {
                "macos" => {
                    eprintln!("Install git on macOS:");
                    eprintln!("  xcode-select --install");
                    eprintln!("or using Homebrew:");
                    eprintln!("  brew install git");
                }
                "windows" => {
                    eprintln!("Install Git for Windows from:");
                    eprintln!("  https://git-scm.com/download/win");
                    eprintln!("or via winget:");
                    eprintln!("  winget install --id Git.Git -e");
                }
                "linux" => {
                    eprintln!("Install git on Linux:");
                    eprintln!("  Debian/Ubuntu: sudo apt-get install git");
                    eprintln!("  Fedora:        sudo dnf install git");
                    eprintln!("  Arch Linux:    sudo pacman -S git");
                }
                _ => {
                    eprintln!("Please install git from https://git-scm.com/downloads");
                }
            }

            anyhow::bail!("git not available on PATH");
        }
    }
}

fn ensure_repo(config_dir: &Path, url: &str) -> Result<()> {
    let web_dir = config_dir.join("web");
    let git_dir = web_dir.join(".git");

    if web_dir.exists() && git_dir.exists() {
        // Update existing repo
        let status_fetch = Command::new("git")
            .arg("-C")
            .arg(&web_dir)
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin")
            .status()
            .with_context(|| "failed to execute git fetch")?;

        if !status_fetch.success() {
            eprintln!("git fetch failed, keeping existing checkout");
            return Ok(());
        }

        let status_reset = Command::new("git")
            .arg("-C")
            .arg(&web_dir)
            .arg("reset")
            .arg("--hard")
            .arg("origin/HEAD")
            .status()
            .with_context(|| "failed to execute git reset")?;

        if !status_reset.success() {
            eprintln!("git reset failed, keeping existing checkout");
        }

        return Ok(());
    }

    // Fresh clone
    if web_dir.exists() {
        fs::remove_dir_all(&web_dir)?;
    }

    let status = Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(url)
        .arg(&web_dir)
        .status()
        .with_context(|| "failed to execute git clone")?;

    if !status.success() {
        anyhow::bail!("git clone failed with status {status}");
    }

    Ok(())
}

// =============================================================================
// Authentication Helpers
// =============================================================================

fn generate_token() -> String {
    let mut buf = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    hex::encode(buf)
}

fn extract_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        let mut kv = trimmed.splitn(2, '=');
        if let (Some(name), Some(value)) = (kv.next(), kv.next()) {
            if name == "tbl_token" {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn check_basic_auth(headers: &HeaderMap, user: &str, pass: &str) -> bool {
    let header_val = match headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        Some(v) => v,
        None => return false,
    };

    if !header_val.starts_with("Basic ") {
        return false;
    }

    let b64 = &header_val[6..];
    let decoded = match BASE64.decode(b64) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let creds = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut parts = creds.splitn(2, ':');
    let u = parts.next().unwrap_or("");
    let p = parts.next().unwrap_or("");

    u == user && p == pass
}

// =============================================================================
// HTTP Handlers
// =============================================================================

/// Root handler: redirect to /web/ if content exists, otherwise show setup page
async fn index_handler(State(state): State<Arc<AppState>>) -> Response {
    let index = state.web_root.join("index.html");
    if index.exists() {
        Redirect::temporary("/web/").into_response()
    } else {
        Html(setup_page_html()).into_response()
    }
}

/// Bootstrap handler: validate token and set authentication cookie
async fn bootstrap_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<BootstrapQuery>,
) -> Response {
    let Some(token) = q.token else {
        return (StatusCode::BAD_REQUEST, "missing token in query").into_response();
    };

    if token != state.auth_token {
        return (StatusCode::FORBIDDEN, "invalid bootstrap token").into_response();
    }

    Html(bootstrap_page_html(&token)).into_response()
}

/// Setup handler: clone git repository and save config
async fn setup_handler(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SetupForm>,
) -> Response {
    let url = form.git_url.trim().to_string();
    if url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>Missing git URL</h1>".to_string()),
        )
            .into_response();
    }

    if let Err(e) = ensure_git_available() {
        let body = format!(
            r#"<!doctype html><html><body>
            <h1>Git is required</h1>
            <pre>{}</pre>
            <p>Please install git and try again.</p>
            </body></html>"#,
            e
        );
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(body)).into_response();
    }

    if let Err(e) = ensure_repo(&state.config_dir, &url) {
        let body = format!(
            r#"<!doctype html><html><body>
            <h1>Failed to clone repository</h1>
            <pre>{}</pre>
            <p><a href="/">Back</a></p>
            </body></html>"#,
            e
        );
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(body)).into_response();
    }

    // Persist config with new git_url
    let mut cfg = state.config.clone();
    cfg.git_url = Some(url);

    if let Err(e) = save_config(&state.config_dir, &cfg) {
        eprintln!("Failed to save config: {e}");
    }

    Redirect::to("/").into_response()
}

/// Ping handler: authenticated health check endpoint
async fn ping_handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    // Optional basic auth
    if let (Some(ref user), Some(ref pass)) = (&state.config.basic_user, &state.config.basic_pass) {
        if !check_basic_auth(&headers, user, pass) {
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Basic realm=\"tbl\"")],
                "basic auth required",
            )
                .into_response();
        }
    }

    // Cookie-based auth
    let token = extract_token_from_cookie(&headers);
    if token.as_deref() != Some(&state.auth_token) {
        return (StatusCode::UNAUTHORIZED, "missing or invalid auth cookie").into_response();
    }

    let payload = PingResponse { status: "ok" };
    let json = serde_json::to_string(&payload).unwrap_or_else(|_| r#"{"status":"ok"}"#.to_string());

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    )
        .into_response()
}

/// Shutdown handler: authenticated endpoint to stop the server
async fn shutdown_handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    // Optional basic auth
    if let (Some(ref user), Some(ref pass)) = (&state.config.basic_user, &state.config.basic_pass) {
        if !check_basic_auth(&headers, user, pass) {
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Basic realm=\"tbl\"")],
                "basic auth required",
            )
                .into_response();
        }
    }

    // Cookie-based auth
    let token = extract_token_from_cookie(&headers);
    if token.as_deref() != Some(&state.auth_token) {
        return (StatusCode::UNAUTHORIZED, "missing or invalid auth cookie").into_response();
    }

    // Trigger shutdown
    let mut tx_guard = state.shutdown_tx.lock().await;
    if let Some(tx) = tx_guard.take() {
        let _ = tx.send(());
    }

    let payload = ShutdownResponse { status: "shutting_down" };
    let json = serde_json::to_string(&payload).unwrap_or_else(|_| r#"{"status":"shutting_down"}"#.to_string());

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    )
        .into_response()
}

/// JavaScript SDK handler
async fn tbl_js_handler() -> Response {
    let js = r#"// tbl.js – tiny helper for tbl's local API
(function () {
  const apiBase = '/api/v1';

  async function request(path, opts) {
    const url = apiBase + path;
    const init = Object.assign(
      {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      },
      opts || {}
    );

    const res = await fetch(url, init);
    if (!res.ok) {
      const text = await res.text().catch(() => '');
      throw new Error('API ' + res.status + ' ' + res.statusText + ': ' + text);
    }

    const ct = res.headers.get('content-type') || '';
    if (ct.includes('application/json')) {
      return res.json();
    }
    return res.text();
  }

  async function ping() {
    return request('/ping');
  }

  window.tblApi = {
    request,
    ping,
  };
})();"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript")],
        js,
    )
        .into_response()
}

// =============================================================================
// Embedded HTML Pages
// =============================================================================

fn bootstrap_page_html(token: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>tbl – bootstrapping…</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    :root {{
      color-scheme: light dark;
      --bg: #0b1020;
      --fg: #f5f5f7;
      --accent: #4f46e5;
      --accent-soft: rgba(79,70,229,0.16);
      --border-subtle: rgba(148,163,184,0.35);
    }}
    * {{
      box-sizing: border-box;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "SF Pro Text",
                   "Segoe UI", sans-serif;
    }}
    body {{
      margin: 0;
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background: radial-gradient(circle at top, #1e293b, #020617 55%);
      color: var(--fg);
    }}
    .card {{
      background: rgba(15,23,42,0.95);
      border-radius: 18px;
      padding: 24px 28px;
      box-shadow: 0 18px 40px rgba(15,23,42,0.85);
      max-width: 420px;
      width: 100%;
      border: 1px solid var(--border-subtle);
      backdrop-filter: blur(18px);
    }}
    .badge {{
      display: inline-flex;
      align-items: center;
      gap: 8px;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      padding: 4px 10px;
      border-radius: 999px;
      background: var(--accent-soft);
      color: #c7d2fe;
    }}
    .pill-dot {{
      width: 6px;
      height: 6px;
      border-radius: 999px;
      background: #22c55e;
      box-shadow: 0 0 0 4px rgba(34,197,94,0.35);
    }}
    h1 {{
      margin: 14px 0 6px;
      font-size: 22px;
      font-weight: 600;
    }}
    p {{
      margin: 6px 0 0;
      font-size: 13px;
      opacity: 0.8;
    }}
    .spinner {{
      margin-top: 18px;
      width: 26px;
      height: 26px;
      border-radius: 999px;
      border: 3px solid rgba(148,163,184,0.4);
      border-top-color: var(--accent);
      animation: spin 0.8s linear infinite;
    }}
    @keyframes spin {{
      to {{ transform: rotate(360deg); }}
    }}
  </style>
</head>
<body>
  <div class="card">
    <div class="badge">
      <span class="pill-dot"></span>
      <span>LOCAL SESSION</span>
    </div>
    <h1>Bootstrapping tbl</h1>
    <p>We're securing your local API and loading your workspace.</p>
    <div class="spinner"></div>
  </div>
  <script>
    (function() {{
      const token = "{token}";
      document.cookie = "tbl_token=" + token + "; SameSite=Lax; Path=/";
      setTimeout(function() {{
        window.location.replace("/");
      }}, 400);
    }})();
  </script>
</body>
</html>"#
    )
}

fn setup_page_html() -> String {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>tbl – first-time setup</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    :root {
      color-scheme: light dark;
      --bg: #020617;
      --card: rgba(15,23,42,0.96);
      --fg: #f9fafb;
      --muted: #9ca3af;
      --accent: #6366f1;
      --accent-soft: rgba(99,102,241,0.12);
      --border-subtle: rgba(148,163,184,0.45);
      --input-bg: rgba(15,23,42,0.9);
    }
    * {
      box-sizing: border-box;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "SF Pro Text",
                   "Segoe UI", sans-serif;
    }
    body {
      margin: 0;
      min-height: 100vh;
      background:
        radial-gradient(circle at top, #1e293b, transparent 60%),
        radial-gradient(circle at bottom, #020617, #000);
      color: var(--fg);
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 24px;
    }
    .shell {
      max-width: 460px;
      width: 100%;
    }
    .logo {
      display: flex;
      align-items: center;
      gap: 10px;
      margin-bottom: 12px;
    }
    .logo-mark {
      width: 26px;
      height: 26px;
      border-radius: 9px;
      background: radial-gradient(circle at 20% 0%, #a5b4fc, #4f46e5);
      box-shadow: 0 8px 22px rgba(79,70,229,0.7);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 14px;
      font-weight: 700;
      color: #e5e7eb;
    }
    .logo-text {
      font-weight: 600;
      letter-spacing: 0.06em;
      font-size: 12px;
      text-transform: uppercase;
      color: var(--muted);
    }
    .card {
      background: var(--card);
      border-radius: 20px;
      padding: 22px 22px 20px;
      border: 1px solid var(--border-subtle);
      box-shadow:
        0 22px 50px rgba(15,23,42,0.95),
        0 0 0 1px rgba(15,23,42,0.8);
      backdrop-filter: blur(20px);
    }
    h1 {
      margin: 0 0 6px;
      font-size: 22px;
      font-weight: 600;
    }
    p {
      margin: 0 0 14px;
      font-size: 13px;
      color: var(--muted);
    }
    .field-label {
      font-size: 12px;
      margin-bottom: 6px;
      color: #e5e7eb;
    }
    input[type="text"] {
      width: 100%;
      padding: 10px 11px;
      border-radius: 11px;
      border: 1px solid rgba(148,163,184,0.7);
      background: var(--input-bg);
      color: var(--fg);
      font-size: 13px;
      outline: none;
      transition: border-color 0.15s ease, box-shadow 0.15s ease,
                  background 0.15s ease;
    }
    input[type="text"]::placeholder {
      color: rgba(148,163,184,0.9);
    }
    input[type="text"]:focus {
      border-color: var(--accent);
      box-shadow: 0 0 0 1px rgba(99,102,241,0.7);
      background: rgba(15,23,42,1);
    }
    .hint {
      font-size: 11px;
      margin-top: 6px;
      color: rgba(148,163,184,0.95);
    }
    button {
      margin-top: 16px;
      width: 100%;
      border-radius: 999px;
      border: none;
      padding: 9px 14px;
      font-size: 13px;
      font-weight: 500;
      background: linear-gradient(135deg, #4f46e5, #6366f1);
      color: white;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      box-shadow: 0 14px 32px rgba(79,70,229,0.6);
      transition: transform 0.07s ease, box-shadow 0.07s ease,
                  filter 0.07s ease;
    }
    button:hover {
      transform: translateY(-1px);
      box-shadow: 0 18px 40px rgba(79,70,229,0.7);
      filter: brightness(1.03);
    }
    button:active {
      transform: translateY(0);
      box-shadow: 0 10px 22px rgba(79,70,229,0.65);
    }
    .btn-icon {
      font-size: 15px;
    }
    .meta {
      margin-top: 10px;
      font-size: 11px;
      color: var(--muted);
      display: flex;
      justify-content: space-between;
      gap: 12px;
      flex-wrap: wrap;
    }
    .pill {
      padding: 3px 8px;
      border-radius: 999px;
      font-size: 10px;
      border: 1px dashed rgba(148,163,184,0.5);
      background: rgba(15,23,42,0.8);
    }
  </style>
</head>
<body>
  <div class="shell">
    <div class="logo">
      <div class="logo-mark">t</div>
      <div class="logo-text">tbl bootstrap</div>
    </div>
    <div class="card">
      <h1>Connect your workspace</h1>
      <p>Point <strong>tbl</strong> at a Git repo that contains your web UI. We'll shallow-clone it into your local config and serve it securely.</p>
      <form method="post" action="/setup">
        <label class="field-label" for="git_url">Git repository URL</label>
        <input
          id="git_url"
          type="text"
          name="git_url"
          placeholder="https://github.com/you/your-tbl-web.git"
          required
        />
        <div class="hint">
          We clone with <code>--depth 1</code> into <code>~/.config/tbl/web/</code>.
        </div>
        <button type="submit">
          <span class="btn-icon">⏎</span>
          <span>Clone &amp; launch</span>
        </button>
      </form>
      <div class="meta">
        <div>CLI &amp; ENV override: <code>--git-url</code>, <code>TBL_GIT_URL</code></div>
        <div class="pill">Single static binary • local only</div>
      </div>
    </div>
  </div>
</body>
</html>"#
        .to_string()
}

// =============================================================================
// Verbose Output Helpers
// =============================================================================

fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  ╭─────────────────────────────────────────╮");
    println!("  │                                         │");
    println!("  │   tbl v{:<32} │", version);
    println!("  │   Tiny self-bootstrapping web launcher  │");
    println!("  │                                         │");
    println!("  ╰─────────────────────────────────────────╯");
}

fn print_url_box(url: &str) {
    let padding = 4;
    let url_len = url.len();
    let box_width = url_len + padding * 2;

    let top_bottom = "─".repeat(box_width);
    let spaces = " ".repeat(padding);

    println!("  ╭{}╮", top_bottom);
    println!("  │{}{}{}│", spaces, url, spaces);
    println!("  ╰{}╯", top_bottom);
}

// =============================================================================
// Stop Command
// =============================================================================

fn handle_stop_command() -> Result<()> {
    let config_dir = get_config_dir()?;
    let run_dir = config_dir.join("run");

    let Some(info) = load_run_info(&run_dir) else {
        println!();
        println!("  No tbl server is currently running.");
        println!();
        return Ok(());
    };

    if !port_is_open(info.port) {
        // Stale pid.yaml
        clear_run_info(&run_dir);
        println!();
        println!("  No tbl server is currently running (stale pid file cleaned up).");
        println!();
        return Ok(());
    }

    println!();
    println!("  Stopping tbl server...");
    println!("  ───────────────────────────────────────");
    println!("  PID:  {}", info.pid);
    println!("  Port: {}", info.port);
    println!();

    // Send authenticated shutdown request
    match send_shutdown_request(info.port, &info.auth_token, info.tls) {
        Ok(_) => {
            // Wait for server to stop (up to 5 seconds)
            for _ in 0..50 {
                std::thread::sleep(Duration::from_millis(100));
                if !port_is_open(info.port) {
                    println!("  Server stopped successfully.");
                    println!();
                    return Ok(());
                }
            }
            println!("  Server may still be shutting down.");
            println!();
        }
        Err(e) => {
            eprintln!("  Failed to send shutdown request: {e}");
            eprintln!("  You may need to kill the process manually (PID: {}).", info.pid);
            println!();
        }
    }

    Ok(())
}

fn send_shutdown_request(port: u16, token: &str, _tls: bool) -> Result<()> {
    // For simplicity, we use plain HTTP even for TLS servers on localhost
    // The auth token provides security
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse::<SocketAddr>()?,
        Duration::from_secs(5),
    )?;

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let request = format!(
        "POST /api/v1/shutdown HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Cookie: tbl_token={}\r\n\
         Content-Length: 0\r\n\
         Connection: close\r\n\
         \r\n",
        port, token
    );

    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    if response.contains("200") || response.contains("shutting_down") {
        Ok(())
    } else {
        anyhow::bail!("Unexpected response: {}", response.lines().next().unwrap_or(""))
    }
}
