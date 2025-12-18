# Architecture

## Overview

tbl is a self-bootstrapping web launcher that:

1. Clones a web UI from a Git repository
2. Serves it locally with authentication
3. Runs as a background daemon
4. Provides an extensible API

## Design Principles

### Single Binary

All assets (HTML, CSS, JS) are embedded as string literals. No external files required except `git` at runtime.

### Localhost-First Security

- Binds to `127.0.0.1` by default
- Per-session random tokens (32 bytes, hex-encoded)
- Tokens invalidate on restart (intentional)

### Self-Modifying

The server can modify its own configuration via the web UI setup form, then clone and serve the configured repository.

## Startup Flow

```
┌─────────────────────────────────────────────────────────────┐
│  tbl                                                        │
├─────────────────────────────────────────────────────────────┤
│  1. Check --stop flag → send shutdown request if set        │
│  2. Print banner (if not daemonized)                        │
│  3. Check for existing daemon (pid.yaml)                    │
│     ├─ Running? → Open browser, exit                        │
│     └─ Stale? → Remove pid.yaml, continue                   │
│  4. Daemonize (re-exec with TBL_DAEMONIZED=1)               │
│  5. Load config (CLI > ENV > file > defaults)               │
│  6. Auto-detect available port                              │
│  7. If git_url set: ensure git available, clone/update      │
│  8. Generate auth token                                     │
│  9. Write pid.yaml                                          │
│ 10. Start HTTP(S) server                                    │
│ 11. Open browser to /bootstrap?token=...                    │
└─────────────────────────────────────────────────────────────┘
```

## Security Model

### Authentication Flow

```
Browser                    tbl Server
   │                           │
   │  GET /bootstrap?token=X   │
   │ ─────────────────────────>│
   │                           │ Validate token
   │  HTML with JS             │
   │ <─────────────────────────│
   │                           │
   │  JS sets cookie           │
   │  tbl_token=X              │
   │                           │
   │  Redirect to /            │
   │ ─────────────────────────>│
   │                           │ Cookie validated
   │  Serve content            │
   │ <─────────────────────────│
```

### Token Generation

- 32 random bytes from OS RNG
- Hex-encoded (64 characters)
- New token each server start

### Optional Layers

1. **HTTP Basic Auth**: Username/password checked before cookie
2. **TLS**: HTTPS via rustls with PEM certificates

## Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                              │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │    CLI      │  │  TblConfig  │  │      AppState       │  │
│  │   (clap)    │  │   (serde)   │  │  (Arc<...>)         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                    Axum Router                          ││
│  │  ┌─────────┐ ┌───────────┐ ┌───────┐ ┌──────────────┐   ││
│  │  │   /     │ │/bootstrap │ │/setup │ │ /api/v1/...  │   ││
│  │  └─────────┘ └───────────┘ └───────┘ └──────────────┘   ││
│  │  ┌─────────┐ ┌───────────┐                              ││
│  │  │ /web/*  │ │  /tbl.js  │   (ServeDir + handlers)      ││
│  │  └─────────┘ └───────────┘                              ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │    Git      │  │    Auth     │  │     Config          │  │
│  │  Helpers    │  │   Helpers   │  │     Helpers         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Key Decisions

### Why Axum?

- Mature async framework with excellent tower-http integration
- Type-safe extractors reduce boilerplate
- Active maintenance and community

### Why Embedded HTML?

- Single binary deployment
- No build step for frontend
- Simple maintenance for 2-3 pages

### Why Re-exec for Daemonization?

- Portable across all platforms (no Unix fork)
- Simple implementation
- Child inherits all CLI args

### Why Port Auto-detection?

- Avoids conflicts with other services
- Better UX than failing on port-in-use
- pid.yaml stores actual port for discovery

## Graceful Shutdown

```
┌─────────────────────────────────────────────────────────────┐
│  tbl --stop                                                 │
├─────────────────────────────────────────────────────────────┤
│  1. Read pid.yaml for port and auth_token                   │
│  2. Check if port is actually open                          │
│  3. Send POST /api/v1/shutdown with cookie                  │
│  4. Wait up to 5 seconds for server to stop                 │
│  5. Server clears pid.yaml on shutdown                      │
└─────────────────────────────────────────────────────────────┘
```

## Specifications

Formal requirements are documented in OpenSpec format:

| Spec                      | File                                       |
| ------------------------- | ------------------------------------------ |
| Authentication & Security | `openspec/specs/auth-security/spec.md`     |
| Configuration Management  | `openspec/specs/config-management/spec.md` |
| Core Server               | `openspec/specs/core-server/spec.md`       |
| Daemon Lifecycle          | `openspec/specs/daemon-lifecycle/spec.md`  |
| Git Integration           | `openspec/specs/git-integration/spec.md`   |
