---
id: r-15-restrict-secret-key-file
level: task
title: "R-15: Restrict secret key file permissions"
short_code: "FIDIUS-T-0042"
created_at: 2026-03-29T16:29:51.001314+00:00
updated_at: 2026-03-29T16:42:48.425054+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-15: Restrict secret key file permissions

**Addresses**: SEC-01

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Restrict secret key file permissions to `0o600` on Unix after writing. The secret key is currently written with default permissions (typically `0o644`), making it readable by any user on the system. This is a direct compromise of the signing model on shared machines.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `std::fs::set_permissions` called with mode `0o600` after writing the secret key file on Unix
- [ ] Permission setting is gated behind `#[cfg(unix)]`
- [ ] Consider emitting a warning on non-Unix platforms about manual permission setting

## Implementation Notes

### Technical Approach

1. In `fidius-cli/src/commands.rs`, after `std::fs::write(&secret_path, ...)`:
   ```rust
   #[cfg(unix)]
   {
       use std::os::unix::fs::PermissionsExt;
       std::fs::set_permissions(&secret_path,
           std::fs::Permissions::from_mode(0o600))?;
   }
   ```
2. Consider emitting a warning on non-Unix platforms about manual permission setting.

### Dependencies

None.

## Status Updates

*To be added during implementation*