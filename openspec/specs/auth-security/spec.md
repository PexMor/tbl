# auth-security Specification

## Purpose
Specifies authentication and security mechanisms including cookie-based token authentication, HTTP Basic auth, and secure bootstrap flow for the tbl server.
## Requirements
### Requirement: Per-Run Token Generation
The system SHALL generate a cryptographically random authentication token on each startup.

#### Scenario: Token generation
- **WHEN** tbl starts
- **THEN** generate a 32-byte random value
- **AND** encode as 64-character hex string
- **AND** store in memory for session validation

### Requirement: Bootstrap Authentication Flow
The system SHALL provide a `/bootstrap` endpoint that validates tokens and sets authentication cookies.

#### Scenario: Valid bootstrap token
- **WHEN** user navigates to `/bootstrap?token=<valid_token>`
- **THEN** return HTML page with embedded JavaScript
- **AND** JavaScript sets `tbl_token` cookie with the token value
- **AND** redirect to `/` after cookie is set

#### Scenario: Missing token
- **WHEN** user navigates to `/bootstrap` without token parameter
- **THEN** return HTTP 400 Bad Request

#### Scenario: Invalid token
- **WHEN** user navigates to `/bootstrap?token=<wrong_token>`
- **THEN** return HTTP 403 Forbidden

### Requirement: Cookie-Based Authentication
The system SHALL validate the `tbl_token` cookie on protected API endpoints.

#### Scenario: Valid cookie
- **WHEN** request includes `Cookie: tbl_token=<valid_token>`
- **AND** endpoint requires authentication
- **THEN** allow request to proceed

#### Scenario: Missing cookie
- **WHEN** request lacks `tbl_token` cookie
- **AND** endpoint requires authentication
- **THEN** return HTTP 401 Unauthorized

#### Scenario: Invalid cookie
- **WHEN** request includes `Cookie: tbl_token=<wrong_value>`
- **AND** endpoint requires authentication
- **THEN** return HTTP 401 Unauthorized

### Requirement: Optional HTTP Basic Authentication
The system SHALL support optional HTTP Basic authentication as an additional layer on API endpoints.

#### Scenario: Basic auth enabled and valid
- **WHEN** basic_user and basic_pass are configured
- **AND** request includes valid `Authorization: Basic <credentials>`
- **AND** cookie is also valid
- **THEN** allow request

#### Scenario: Basic auth enabled but missing
- **WHEN** basic_user and basic_pass are configured
- **AND** request lacks Authorization header
- **THEN** return HTTP 401 with `WWW-Authenticate: Basic realm="tbl"` header

#### Scenario: Basic auth not configured
- **WHEN** basic_user and basic_pass are not set
- **THEN** skip Basic auth check (cookie only)

### Requirement: Optional TLS Support
The system SHALL support HTTPS when TLS certificate and key are provided.

#### Scenario: TLS enabled
- **WHEN** `--tls-cert` and `--tls-key` are provided
- **THEN** start HTTPS server using rustls
- **AND** use `https://` in bootstrap URL

#### Scenario: TLS not configured
- **WHEN** TLS options are not provided
- **THEN** start HTTP server
- **AND** use `http://` in bootstrap URL

#### Scenario: Invalid TLS files
- **WHEN** certificate or key file is invalid or missing
- **THEN** exit with clear error message

### Requirement: Browser Auto-Open
The system SHALL automatically open the user's default browser to the bootstrap URL.

#### Scenario: Auto-open enabled
- **WHEN** tbl starts
- **AND** `--no-browser` is not set
- **THEN** open browser to `http(s)://127.0.0.1:<port>/bootstrap?token=<token>`

#### Scenario: Auto-open disabled
- **WHEN** `--no-browser` is set
- **THEN** print the bootstrap URL to stdout instead

#### Scenario: Browser open fails
- **WHEN** webbrowser::open() fails
- **THEN** print error and URL to stderr
- **AND** continue running

