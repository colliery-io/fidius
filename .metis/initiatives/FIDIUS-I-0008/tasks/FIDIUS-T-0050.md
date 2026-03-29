---
id: r-11-build-package-returns-error
level: task
title: "R-11: build_package returns error when cdylib not found"
short_code: "FIDIUS-T-0050"
created_at: 2026-03-29T17:19:46.683036+00:00
updated_at: 2026-03-29T17:30:52.181561+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-11: build_package returns error when cdylib not found

**Addresses**: API-07, LEG-15 | **Effort**: < 1 hour

## Objective

Fix `build_package` to return an error when the cdylib artifact is not found after building, instead of silently returning the target directory path which is indistinguishable from success.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `build_package` returns `Err(PackageError::CdylibNotFound { dir: target_dir })` when no cdylib is found
- [ ] `CdylibNotFound` variant added to `PackageError` with a `dir` field
- [ ] `CdylibNotFound` has a meaningful `Display` message (e.g., "no cdylib artifact found in {dir}")
- [ ] Callers that previously received `Ok(target_dir)` now receive an error and can handle it appropriately
- [ ] All tests pass

## Implementation Notes

1. In `fidius-host/src/package.rs`, locate the fallback `Ok(target_dir)` return and replace with `Err(PackageError::CdylibNotFound { dir: target_dir })`.
2. Add the `CdylibNotFound { dir: PathBuf }` variant to `PackageError`.
3. This is a small, self-contained fix.

### Dependencies

- None.

### Files

- `fidius-host/src/package.rs` -- error variant and return fix

## Status Updates

*To be added during implementation*