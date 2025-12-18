## ADDED Requirements

### Requirement: Server Stop Command
The system SHALL provide a `--stop` CLI flag to gracefully stop a running tbl daemon.

#### Scenario: Stop running server
- **WHEN** user runs `tbl --stop`
- **AND** a tbl daemon is running (pid.yaml exists and port is open)
- **THEN** send authenticated shutdown request to the running server
- **AND** wait for graceful shutdown (up to 5 seconds)
- **AND** print confirmation message
- **AND** exit with code 0

#### Scenario: No server running
- **WHEN** user runs `tbl --stop`
- **AND** no tbl daemon is running
- **THEN** print "No tbl server is currently running"
- **AND** exit with code 0

### Requirement: Shutdown API Endpoint
The system SHALL provide a `/api/v1/shutdown` endpoint that accepts POST requests with valid authentication to trigger graceful shutdown.

#### Scenario: Authenticated shutdown
- **WHEN** POST request to `/api/v1/shutdown` with valid `tbl_token` cookie
- **THEN** return HTTP 200 with `{"status":"shutting_down"}`
- **AND** initiate graceful server shutdown

#### Scenario: Unauthenticated shutdown attempt
- **WHEN** POST request to `/api/v1/shutdown` without valid cookie
- **THEN** return HTTP 401 Unauthorized

### Requirement: Verbose Startup Output
The system SHALL provide verbose, user-friendly console output during startup.

#### Scenario: Fresh server start
- **WHEN** tbl starts a new server instance
- **THEN** print startup banner with version
- **AND** print "Starting tbl server..."
- **AND** print bound address and TLS status
- **AND** print authentication URL prominently in a box format
- **AND** print "Browser opening..." or "Open the URL above to authenticate" based on --no-browser

#### Scenario: Daemon already running
- **WHEN** tbl detects an existing daemon via pid.yaml
- **AND** the port is reachable
- **THEN** print "tbl is already running (PID: <pid>, Port: <port>)"
- **AND** print authentication URL prominently in a box format
- **AND** open browser (unless --no-browser)
- **AND** exit with code 0

### Requirement: Browser Auto-Open on Daemon Detection
The system SHALL open the browser when connecting to an existing daemon, unless suppressed.

#### Scenario: Existing daemon with browser
- **WHEN** existing daemon is detected
- **AND** `--no-browser` is NOT set
- **THEN** print URL prominently
- **AND** open browser to bootstrap URL

#### Scenario: Existing daemon without browser
- **WHEN** existing daemon is detected
- **AND** `--no-browser` is set
- **THEN** print URL prominently
- **AND** do NOT open browser

