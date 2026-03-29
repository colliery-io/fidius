---
id: r-13-fix-verify-command-process
level: task
title: "R-13: Fix verify command process::exit → error return"
short_code: "FIDIUS-T-0041"
created_at: 2026-03-29T16:29:50.107468+00:00
updated_at: 2026-03-29T16:42:16.396975+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-13: Fix verify command process::exit → error return

**Addresses**: COR-18, API-12, OPS-06

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Replace `process::exit(1)` with `Err(...)` in the `verify` command. The current approach bypasses destructors and prevents composability. The `package verify` command inherits this behavior.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `std::process::exit(1)` replaced with `Err(format!("Signature INVALID: {}", dylib_path.display()).into())` in the verify command
- [ ] `package verify` no longer calls `process::exit` either
- [ ] Error propagation follows the same pattern as other CLI commands

## Implementation Notes

### Technical Approach

1. In `fidius-cli/src/commands.rs`, change the `Err(_)` branch of verify to:
   ```rust
   Err(_) => Err(format!("Signature INVALID: {}", dylib_path.display()).into())
   ```

### Dependencies

None.

## Status Updates

*To be added during implementation*