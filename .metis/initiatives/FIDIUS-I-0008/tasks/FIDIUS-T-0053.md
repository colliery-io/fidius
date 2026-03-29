---
id: r-17-consolidate-signing-utility
level: task
title: "R-17: Consolidate signing utility functions"
short_code: "FIDIUS-T-0053"
created_at: 2026-03-29T17:19:50.241445+00:00
updated_at: 2026-03-29T17:33:43.851516+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-17: Consolidate signing utility functions

**Addresses**: LEG-11, EVO-11, API-13, LEG-17 | **Effort**: 2-3 hours

## Objective

Eliminate duplicated signing utility code by extracting shared functions (sig path construction, build invocation) into `fidius-host` and having the CLI delegate to them, reducing maintenance burden and divergence risk.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `pub fn sig_path_for(path: &Path) -> PathBuf` added to `fidius-host/src/signing.rs`
- [ ] CLI sign, verify, package_sign, and package_verify commands use `sig_path_for()` instead of inline `.with_extension("sig")` / format constructions
- [ ] CLI `package_build` delegates to `fidius_host::package::build_package` instead of reimplementing the cargo build invocation
- [ ] All three duplicated sig-path construction blocks in CLI removed
- [ ] Both duplicated build invocation blocks in CLI consolidated
- [ ] All existing signing and packaging tests pass
- [ ] No behavioral changes -- purely a refactor

## Implementation Notes

1. In `fidius-host/src/signing.rs`, add:
   ```rust
   pub fn sig_path_for(path: &Path) -> PathBuf {
       path.with_extension("sig")
   }
   ```
2. In `fidius-cli/src/commands.rs`, replace all inline sig path constructions with calls to `fidius_host::signing::sig_path_for()`.
3. In `fidius-cli/src/commands.rs`, have `package_build` call `fidius_host::package::build_package()` instead of duplicating the cargo invocation logic.

### Dependencies

- None.

### Files

- `fidius-host/src/signing.rs` -- add sig_path_for()
- `fidius-cli/src/commands.rs` -- delegate to shared functions

## Status Updates

*To be added during implementation*