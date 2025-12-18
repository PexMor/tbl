# core-server Specification

## Purpose
Specifies the core Axum web server functionality including routing, static file serving, embedded UI pages, and the JavaScript SDK endpoint.
## Requirements
### Requirement: Web Server Initialization
The system SHALL start an HTTP(S) server on a configurable address using Axum with Tokio runtime.

#### Scenario: Server starts on default address
- **WHEN** tbl is started without address configuration
- **THEN** the server binds to `127.0.0.1:1234` or the next available port

#### Scenario: Server starts with custom address
- **WHEN** tbl is started with `--addr 0.0.0.0:8080`
- **THEN** the server binds to `0.0.0.0` on port 8080 or next available

### Requirement: Root Route Handler
The system SHALL serve the root route `/` with conditional behavior based on web content availability.

#### Scenario: Web content exists
- **WHEN** `~/.config/tbl/web/index.html` exists
- **AND** user navigates to `/`
- **THEN** redirect to `/web/`

#### Scenario: No web content
- **WHEN** `~/.config/tbl/web/index.html` does not exist
- **AND** user navigates to `/`
- **THEN** display the embedded setup page with Git URL form

### Requirement: Static File Serving
The system SHALL serve static files from the cloned web repository under the `/web/` path.

#### Scenario: Valid static file request
- **WHEN** user requests `/web/index.html`
- **AND** the file exists in `~/.config/tbl/web/`
- **THEN** return the file with appropriate Content-Type

#### Scenario: Missing static file
- **WHEN** user requests `/web/nonexistent.js`
- **AND** the file does not exist
- **THEN** return HTTP 404

### Requirement: JavaScript SDK Endpoint
The system SHALL serve an embedded JavaScript SDK at `/tbl.js` for API interaction.

#### Scenario: SDK request
- **WHEN** user requests `/tbl.js`
- **THEN** return JavaScript code with Content-Type `application/javascript`
- **AND** the SDK provides `window.tblApi.request()` and `window.tblApi.ping()` functions
- **AND** requests include credentials automatically

### Requirement: Health Check API
The system SHALL provide a health check endpoint at `/api/v1/ping` that requires authentication.

#### Scenario: Authenticated ping
- **WHEN** user calls `/api/v1/ping` with valid `tbl_token` cookie
- **THEN** return `{"status":"ok"}` with HTTP 200

#### Scenario: Unauthenticated ping
- **WHEN** user calls `/api/v1/ping` without valid cookie
- **THEN** return HTTP 401 Unauthorized

### Requirement: Setup Form Handler
The system SHALL accept Git repository URLs via POST to `/setup` and trigger cloning.

#### Scenario: Valid Git URL submission
- **WHEN** user submits a valid Git URL to `/setup`
- **AND** git clone succeeds
- **THEN** save URL to config file
- **AND** redirect to `/`

#### Scenario: Invalid or empty URL
- **WHEN** user submits an empty Git URL
- **THEN** return HTTP 400 with error message

#### Scenario: Clone failure
- **WHEN** git clone fails
- **THEN** return HTTP 500 with error details
- **AND** provide link back to setup page

