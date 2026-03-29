---
id: implement-loadpolicy-enforcement
level: task
title: "Implement LoadPolicy enforcement in PluginHost"
short_code: "FIDIUS-T-0029"
created_at: 2026-03-29T13:15:37.980760+00:00
updated_at: 2026-03-29T13:15:37.980760+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Implement LoadPolicy enforcement in PluginHost

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

`LoadPolicy` is defined with `Strict` and `Lenient` variants and stored in `PluginHost`, but never actually checked. The `load_policy` field is `#[allow(dead_code)]`. Wire it into the load path so `Lenient` degrades signature/validation failures to warnings instead of errors.

## Current Behavior

- `LoadPolicy::Strict` (default) — defined but unused; signature enforcement is controlled solely by the `require_signature` bool
- `LoadPolicy::Lenient` — defined but unused; documented as "warn on unsigned plugins but allow loading"
- The `load_policy` field on `PluginHost` is `#[allow(dead_code)]`

## Desired Behavior

- `LoadPolicy::Strict` + `require_signature(true)` — reject unsigned/invalid signatures (current behavior, just needs to reference the policy)
- `LoadPolicy::Lenient` + `require_signature(true)` — attempt verification, log warning on failure, but continue loading
- `LoadPolicy::Lenient` without `require_signature` — skip verification entirely (same as current default)
- Remove `#[allow(dead_code)]` from the field

## Acceptance Criteria

- [ ] `PluginHost::load()` checks `load_policy` when signature verification fails
- [ ] `Lenient` policy logs a warning (via `eprintln!` or `log` crate) and continues on `SignatureInvalid` / `SignatureRequired`
- [ ] `Strict` policy returns the error (current behavior)
- [ ] `#[allow(dead_code)]` removed from `load_policy` field
- [ ] Test: Lenient + unsigned plugin → loads successfully (with warning)
- [ ] Test: Strict + unsigned plugin → returns `SignatureRequired`
- [ ] Docs updated to note that LoadPolicy is now enforced

## Implementation Notes

### Technical Approach

File: `fidius-host/src/host.rs`, `load()` method.

Current code (line ~191):
```rust
if self.require_signature {
    signing::verify_signature(&path, &self.trusted_keys)?;
}
```

Change to:
```rust
if self.require_signature {
    match signing::verify_signature(&path, &self.trusted_keys) {
        Ok(()) => {}
        Err(e) if self.load_policy == LoadPolicy::Lenient => {
            eprintln!("warning: {e}");
        }
        Err(e) => return Err(e),
    }
}
```

### Dependencies
- None

## Status Updates

*To be added during implementation*