# config-management Specification

## Purpose
Specifies configuration loading and persistence including multi-format support (JSON, YAML, TOML), precedence rules (CLI > ENV > file > defaults), and configuration directory management.
## Requirements
### Requirement: Configuration Precedence
The system SHALL load configuration with precedence: CLI arguments > environment variables > config file > defaults.

#### Scenario: CLI overrides environment
- **WHEN** `TBL_ADDR=127.0.0.1:9999` is set
- **AND** `--addr 127.0.0.1:8080` is provided
- **THEN** use `127.0.0.1:8080`

#### Scenario: Environment overrides file
- **WHEN** config file contains `addr: 127.0.0.1:7777`
- **AND** `TBL_ADDR=127.0.0.1:9999` is set
- **AND** no CLI flag provided
- **THEN** use `127.0.0.1:9999`

#### Scenario: File provides value when others absent
- **WHEN** config file contains `git_url: https://github.com/user/repo.git`
- **AND** no CLI or ENV provides git_url
- **THEN** use the file value

### Requirement: Multi-Format Config File Support
The system SHALL read configuration from JSON, YAML, or TOML files in the config directory.

#### Scenario: JSON config
- **WHEN** `~/.config/tbl/config.json` exists with valid JSON
- **THEN** parse and apply configuration

#### Scenario: YAML config
- **WHEN** `~/.config/tbl/config.yaml` or `config.yml` exists
- **THEN** parse and apply configuration

#### Scenario: TOML config
- **WHEN** `~/.config/tbl/config.toml` exists with valid TOML
- **THEN** parse and apply configuration

#### Scenario: First matching format wins
- **WHEN** both `config.json` and `config.yaml` exist
- **THEN** use `config.json` (checked first)

### Requirement: CLI Arguments
The system SHALL accept the following CLI arguments via clap derive macros.

#### Scenario: Git URL argument
- **WHEN** `--git-url https://github.com/user/repo.git` is provided
- **THEN** use that URL for web content cloning

#### Scenario: Address argument
- **WHEN** `--addr 0.0.0.0:8443` is provided
- **THEN** bind to that address

#### Scenario: TLS arguments
- **WHEN** `--tls-cert /path/cert.pem --tls-key /path/key.pem` are provided
- **THEN** enable HTTPS with those certificates

#### Scenario: Basic auth arguments
- **WHEN** `--basic-user admin --basic-pass secret` are provided
- **THEN** require HTTP Basic authentication for API endpoints

#### Scenario: No browser argument
- **WHEN** `--no-browser` is provided
- **THEN** do not auto-open the browser

### Requirement: Environment Variables
The system SHALL read configuration from environment variables prefixed with `TBL_`.

#### Scenario: Git URL from environment
- **WHEN** `TBL_GIT_URL` is set
- **THEN** use that value for web content URL

#### Scenario: All supported variables
- **WHEN** environment variables are set
- **THEN** support `TBL_GIT_URL`, `TBL_ADDR`, `TBL_TLS_CERT`, `TBL_TLS_KEY`, `TBL_BASIC_USER`, `TBL_BASIC_PASS`

### Requirement: Config Persistence
The system SHALL persist configuration changes to `~/.config/tbl/config.json`.

#### Scenario: Save after setup
- **WHEN** user submits Git URL via `/setup`
- **THEN** save updated configuration to `config.json`
- **AND** preserve existing configuration values

