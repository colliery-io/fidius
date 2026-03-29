---
id: r-09-add-tracing-observability
level: task
title: "R-09: Add tracing observability infrastructure"
short_code: "FIDIUS-T-0048"
created_at: 2026-03-29T17:19:44.450157+00:00
updated_at: 2026-03-29T17:54:13.051949+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-09: Add tracing observability infrastructure

**Addresses**: OPS-01, OPS-02, OPS-04, OPS-05, OPS-07, OPS-08, OPS-09, OPS-12, OPS-13 | **Effort**: 3-5 days

## Objective

Add structured observability to fidius-host using the `tracing` crate behind an optional feature flag, and expose a `--verbose` flag in the CLI, so that plugin loading failures, discovery rejections, and validation steps produce actionable diagnostic output instead of being silently swallowed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `tracing` added as optional dep in `fidius-host/Cargo.toml` behind `tracing` feature flag
- [ ] All tracing calls gated with `#[cfg(feature = "tracing")]` -- zero cost when disabled
- [ ] `PluginHost::load()` instrumented: info span with plugin name, debug events for candidates tried, warn for candidates that match but fail validation
- [ ] `PluginHost::discover()` instrumented: info span with search paths, debug events per dylib with accept/reject and reason
- [ ] `load_library()` instrumented: debug events for each validation step (arch check, magic, version, hash)
- [ ] `verify_signature()` instrumented: debug event with result
- [ ] File path context added to `InvalidMagic` and other errors missing location info
- [ ] CLI accepts `--verbose` global flag that initializes `tracing_subscriber` with DEBUG filter (default: WARN)
- [ ] Build succeeds with and without the `tracing` feature enabled

## Implementation Notes

1. In `fidius-host/Cargo.toml`, add `tracing = { version = "0.1", optional = true }`.
2. In `fidius-host/src/host.rs`, instrument `load()` and `discover()` with spans and events.
3. In `fidius-host/src/loader.rs`, instrument `load_library()` with debug events per validation step.
4. Consider adding `discover_with_diagnostics()` returning `(Vec<PluginInfo>, Vec<(PathBuf, LoadError)>)` to surface rejection reasons programmatically.
5. In `fidius-cli/src/main.rs`, add `--verbose` flag via clap, initialize `tracing_subscriber` accordingly.

### Dependencies

- R-04 (FIDIUS-T-0042): Signature verification should move before dlopen first, so tracing captures the correct flow.

### Files

- `fidius-host/Cargo.toml` -- optional tracing dep
- `fidius-host/src/host.rs` -- instrument load(), discover()
- `fidius-host/src/loader.rs` -- instrument load_library(), verify_signature()
- `fidius-cli/src/main.rs` -- --verbose flag and subscriber init

## Status Updates

*To be added during implementation*