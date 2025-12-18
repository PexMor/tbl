## Context

This is the initial implementation of `tbl`, a self-bootstrapping web launcher. The design was developed through a ChatGPT conversation and is documented in `openspec/project.md`. Key stakeholders are developers who want a quick way to spin up a local web UI from a Git repository with minimal configuration.

The implementation must:
- Work across macOS, Windows, and Linux
- Build as a single static binary
- Require no external dependencies except `git` at runtime
- Be secure by default (localhost-only, per-session tokens)

## Goals / Non-Goals

**Goals:**
- Implement a fully functional single-binary web server
- Support configuration via CLI, environment variables, and config files
- Provide secure cookie-based authentication
- Clone and serve web UIs from Git repositories
- Support optional TLS and HTTP Basic auth
- Daemonize and detect already-running instances

**Non-Goals:**
- Plugin system or extensibility framework (future work)
- Database or persistent storage beyond config files
- User management or multi-user support
- Container/orchestration support (keep it simple)

## Decisions

### Decision 1: Axum as Web Framework
**Choice**: Axum 0.7 with Tokio runtime
**Rationale**: 
- Mature, well-documented async web framework
- Excellent integration with tower-http for static file serving
- Type-safe extractors reduce boilerplate
- Active maintenance and community

**Alternatives considered**:
- Actix-web: More features but heavier, less ergonomic for small apps
- Warp: Functional style harder to read for this use case
- Rocket: Requires nightly for some features

### Decision 2: Embedded HTML/CSS/JS
**Choice**: Store all UI as raw string literals in Rust code
**Rationale**:
- Single binary with no external assets
- No build step for frontend
- Simple to maintain for the 2-3 embedded pages needed

**Alternatives considered**:
- include_bytes!: More complex, doesn't help since we need dynamic token injection
- External asset bundling: Defeats single-binary goal

### Decision 3: Daemonization via Re-exec
**Choice**: Parent process spawns itself with `TBL_DAEMONIZED=1` env var, then exits
**Rationale**:
- Works on all platforms (no Unix-specific fork())
- Simple to implement
- Child inherits all CLI args

**Alternatives considered**:
- Unix fork(): Not portable to Windows
- Systemd/launchd integration: Out of scope for MVP
- No daemonization: Poor UX, terminal stays blocked

### Decision 4: Port Auto-detection
**Choice**: Start at configured port (default 1234), increment until finding an available one
**Rationale**:
- Avoids port conflicts with other services
- User doesn't need to hunt for available ports
- Works well with daemon detection (pid.yaml stores actual port)

**Alternatives considered**:
- Fixed port only: Bad UX when port is in use
- Port 0 (OS-assigned): Harder to predict for users

### Decision 5: Config File Format Support
**Choice**: Read JSON, YAML, or TOML; write JSON only
**Rationale**:
- Flexibility for users who prefer different formats
- Writing only JSON simplifies code
- YAML popular in DevOps, TOML popular in Rust ecosystem

### Decision 6: Authentication Model
**Choice**: Per-run random token set via URL parameter, stored in cookie by JS
**Rationale**:
- No persistent secrets to manage
- Browser auto-opens with token, sets cookie, redirects
- Cookie is automatically sent on subsequent requests
- Session is inherently tied to the running process

**Trade-offs**:
- Token changes on restart (intentional - forces re-auth)
- Requires JavaScript for cookie setting (acceptable for modern browsers)

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `git` not installed | Detect at startup, show OS-specific install instructions |
| Port exhaustion | Cap search at 100 ports, show clear error |
| Stale pid.yaml | Verify port is actually reachable before reusing |
| TLS cert issues | Use RustlsConfig with clear error messages |
| Static binary size | Accept larger binary for simplicity; MUSL target compresses well |

## Migration Plan

N/A - This is a greenfield implementation with no existing code to migrate.

## Open Questions

1. **Should `--foreground` flag be added?** - Useful for debugging, but can be deferred to a follow-up change.
2. **Should we support digest auth?** - Mentioned in ChatGPT conversation but not implemented in final code; defer to future change.
3. **Config file format for writing** - Currently JSON only; should users be able to choose?

