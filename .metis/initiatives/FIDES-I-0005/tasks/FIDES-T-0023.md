---
id: inspect-command-and-cli
level: task
title: "Inspect command and CLI integration tests"
short_code: "FIDES-T-0023"
created_at: 2026-03-29T11:35:19.588700+00:00
updated_at: 2026-03-29T11:52:45.479989+00:00
parent: FIDES-I-0005
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0005
---

# Inspect command and CLI integration tests

## Parent Initiative

[[FIDES-I-0005]]

## Objective

Implement `fides inspect <dylib>` which loads a plugin dylib and dumps its registry metadata. Also write CLI integration tests using `assert_cmd` to verify all commands work end-to-end via the binary.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fides inspect <dylib>` prints: plugin count, each plugin's name, interface name, hash, version, capabilities, wire format, buffer strategy
- [ ] Output is human-readable and structured
- [ ] Non-fides dylibs produce a clear error (invalid magic)
- [ ] CLI integration tests via `assert_cmd`: `--help` works, `init-interface` creates expected files, `keygen` → `sign` → `verify` round-trip, `inspect` on test-plugin-smoke produces expected output
- [ ] All tests pass

## Implementation Notes

### Technical Approach

Inspect uses `fides_host::loader::load_library()` to get the registry, then prints each plugin's info.

For `assert_cmd` tests, add it as a dev-dep of fides-cli and use `Command::cargo_bin("fides")` to invoke the built binary.

### Dependencies
- FIDES-T-0021, FIDES-T-0022 (all other commands must exist)

## Status Updates

- **2026-03-29**: 6 CLI integration tests via assert_cmd: help, init-interface (creates files + errors on duplicate), init-plugin (creates cdylib crate), keygen→sign→verify round-trip, inspect (shows BasicCalculator/Calculator/PluginAllocated). All passing.