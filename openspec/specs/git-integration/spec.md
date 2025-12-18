# git-integration Specification

## Purpose
Specifies Git integration including availability detection with OS-specific install hints, shallow cloning of web UI repositories, and automatic updates on restart.
## Requirements
### Requirement: Git Availability Detection
The system SHALL detect if `git` is available on the system PATH before attempting repository operations.

#### Scenario: Git available
- **WHEN** `git --version` succeeds
- **THEN** proceed with repository operations

#### Scenario: Git not available on macOS
- **WHEN** `git` is not found
- **AND** OS is macOS
- **THEN** display install instructions: `xcode-select --install` or `brew install git`

#### Scenario: Git not available on Windows
- **WHEN** `git` is not found
- **AND** OS is Windows
- **THEN** display install instructions: download from git-scm.com or `winget install --id Git.Git`

#### Scenario: Git not available on Linux
- **WHEN** `git` is not found
- **AND** OS is Linux
- **THEN** display install instructions for apt/dnf/pacman

### Requirement: Repository Cloning
The system SHALL shallow-clone Git repositories into `~/.config/tbl/web/`.

#### Scenario: Fresh clone
- **WHEN** web directory does not exist
- **AND** Git URL is configured
- **THEN** execute `git clone --depth 1 <url> ~/.config/tbl/web/`

#### Scenario: Clone success
- **WHEN** git clone completes successfully
- **THEN** web content is available at `~/.config/tbl/web/`

#### Scenario: Clone failure
- **WHEN** git clone fails (invalid URL, network error, auth required)
- **THEN** return error with git's output
- **AND** do not leave partial directory

### Requirement: Repository Updates
The system SHALL update existing repositories on restart using fetch and reset.

#### Scenario: Existing repo update
- **WHEN** `~/.config/tbl/web/.git` exists
- **AND** tbl starts
- **THEN** execute `git fetch --depth 1 origin`
- **AND** execute `git reset --hard origin/HEAD`

#### Scenario: Fetch failure
- **WHEN** git fetch fails
- **THEN** log warning
- **AND** continue using existing checkout

#### Scenario: Reset failure
- **WHEN** git reset fails
- **THEN** log warning
- **AND** continue using existing checkout

