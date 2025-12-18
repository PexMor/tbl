## ADDED Requirements

### Requirement: Daemonization
The system SHALL run as a background daemon by re-executing itself with the `TBL_DAEMONIZED` environment variable.

#### Scenario: Initial startup
- **WHEN** tbl starts without `TBL_DAEMONIZED` set
- **THEN** spawn new process with `TBL_DAEMONIZED=1`
- **AND** pass all CLI arguments to child
- **AND** detach stdin/stdout/stderr
- **AND** parent process exits immediately

#### Scenario: Daemonized startup
- **WHEN** tbl starts with `TBL_DAEMONIZED=1`
- **THEN** continue as the server process
- **AND** change working directory to `~/.config/tbl/`

### Requirement: Runtime State File
The system SHALL maintain a `~/.config/tbl/run/pid.yaml` file with current process information.

#### Scenario: Write pid.yaml on startup
- **WHEN** server starts successfully
- **THEN** write YAML file containing: pid, port, auth_token, tls (boolean)

#### Scenario: pid.yaml content
- **WHEN** pid.yaml is read
- **THEN** it contains valid YAML with numeric pid, numeric port, string auth_token, boolean tls

### Requirement: Existing Instance Detection
The system SHALL detect if another tbl instance is already running and reuse it.

#### Scenario: Running instance found
- **WHEN** tbl starts
- **AND** `~/.config/tbl/run/pid.yaml` exists
- **AND** the recorded port accepts connections
- **THEN** open browser to existing instance's bootstrap URL
- **AND** exit without starting new server

#### Scenario: Stale pid.yaml
- **WHEN** tbl starts
- **AND** `~/.config/tbl/run/pid.yaml` exists
- **AND** the recorded port does not accept connections
- **THEN** delete stale pid.yaml
- **AND** continue with new server startup

### Requirement: Port Auto-Detection
The system SHALL find an available port starting from the configured base port.

#### Scenario: Base port available
- **WHEN** configured port (default 1234) is not in use
- **THEN** bind to that port

#### Scenario: Base port in use by other program
- **WHEN** configured port is in use
- **AND** it's not a tbl instance (no valid pid.yaml)
- **THEN** try next port (1235, 1236, etc.)
- **AND** continue until finding available port

#### Scenario: Port discovery success
- **WHEN** available port is found
- **THEN** update config with actual bound address
- **AND** record port in pid.yaml

### Requirement: Config Directory Management
The system SHALL use `~/.config/tbl/` as the base directory for all persistent data.

#### Scenario: Directory creation
- **WHEN** tbl starts
- **AND** `~/.config/tbl/` does not exist
- **THEN** create the directory structure

#### Scenario: Cross-platform config location
- **WHEN** running on Windows
- **THEN** use appropriate config directory via `dirs` crate (e.g., `%APPDATA%\tbl`)

#### Scenario: Working directory change
- **WHEN** daemon process starts
- **THEN** change current working directory to config directory

