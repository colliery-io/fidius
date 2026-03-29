---
id: pluginhost-builder-and-discover
level: task
title: "PluginHost builder and discover()"
short_code: "FIDES-T-0015"
created_at: 2026-03-29T01:28:32.318052+00:00
updated_at: 2026-03-29T11:21:58.974560+00:00
parent: FIDES-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0003
---

# PluginHost builder and discover()

## Parent Initiative

[[FIDES-I-0003]]

## Objective

Implement the `PluginHost` builder pattern and `discover()` method. The builder configures search paths, load policy, signature requirements, and expected interface hash/version. `discover()` scans directories for dylibs and returns metadata without loading them for calling.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginHost::builder()` returns a builder with chainable methods
- [ ] Builder methods: `search_path(path)`, `load_policy(LoadPolicy)`, `require_signature(bool)`, `trusted_keys(&[VerifyingKey])`, `interface_hash(u64)`, `wire_format(WireFormat)`, `buffer_strategy(BufferStrategyKind)`
- [ ] `build() -> Result<PluginHost, LoadError>` validates configuration
- [ ] `discover() -> Result<Vec<PluginInfo>, LoadError>` scans search paths for `.dylib`/`.so`/`.dll` files, loads each, validates, returns metadata
- [ ] `load(name: &str) -> Result<LoadedPlugin, LoadError>` finds a plugin by name in search paths, loads and validates it, returns Arc<Library> + descriptor info
- [ ] Platform-aware file extensions: `.dylib` on macOS, `.so` on Linux, `.dll` on Windows

## Implementation Notes

### Technical Approach

File: `fides-host/src/host.rs`

`PluginHost` stores the configuration. `discover()` iterates directory entries, filters by extension, calls the loader from T-0014 on each, collects results. `load()` does the same but filters by plugin name and returns a single result.

### Dependencies
- FIDES-T-0013 (types), FIDES-T-0014 (loader)

## Status Updates

- **2026-03-29**: Implemented in `fides-host/src/host.rs`. Builder with all chainable methods, `discover()` scans directories for platform-appropriate dylibs, `load(name)` finds specific plugin. Platform-aware extension check (dylib/so/dll). Compiles clean.