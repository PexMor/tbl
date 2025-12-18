## 1. CLI Changes

- [x] 1.1 Add `--stop` flag to Cli struct
- [x] 1.2 Update CLI help text to be descriptive

## 2. Verbose Output

- [x] 2.1 Add clear startup banner with version
- [x] 2.2 Improve "daemon already running" message with prominent URL display
- [x] 2.3 Add clear "starting new server" message
- [x] 2.4 Ensure URL is always printed (not just when --no-browser)

## 3. Stop Functionality

- [x] 3.1 Add `/api/v1/shutdown` POST endpoint (cookie-authenticated)
- [x] 3.2 Implement `--stop` behavior: read pid.yaml, call shutdown endpoint
- [x] 3.3 Handle shutdown gracefully with timeout
- [x] 3.4 Clear pid.yaml after successful shutdown

## 4. Browser Behavior

- [x] 4.1 Always open browser when daemon is found (unless --no-browser)
- [x] 4.2 Print URL prominently even when browser opens

## 5. Validation

- [x] 5.1 Test `cargo build` succeeds
- [x] 5.2 Test verbose output displays correctly
- [x] 5.3 Test `--stop` stops running daemon

## Dependencies

- Section 2 can be done in parallel with section 3
- Section 4 depends on section 2
- Section 5 depends on all previous sections
