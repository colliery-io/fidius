---
id: plugin-loading-and-descriptor
level: task
title: "Plugin loading and descriptor validation"
short_code: "FIDIUS-T-0014"
created_at: 2026-03-29T01:28:31.030839+00:00
updated_at: 2026-03-29T11:21:04.052580+00:00
parent: FIDIUS-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0003
---

# Plugin loading and descriptor validation

## Parent Initiative

[[FIDIUS-I-0003]]

## Objective

Implement the core loading function: `dlopen` a dylib, `dlsym("fidius_get_registry")`, call it, validate the registry and each descriptor, copy all FFI data to owned types. This is the heart of fidius-host — the full validation sequence from the spec.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `load_library(path) -> Result<(Arc<Library>, PluginRegistry), LoadError>` — dlopen + dlsym + call fidius_get_registry
- [ ] Validate magic bytes → `LoadError::InvalidMagic`
- [ ] Validate registry_version → `LoadError::IncompatibleRegistryVersion`
- [ ] `validate_descriptor(desc, expected_hash, expected_wire, expected_strategy) -> Result<PluginInfo, LoadError>` — checks abi_version, interface_hash, wire_format, buffer_strategy
- [ ] Copy `*const c_char` fields to owned `String` (interface_name, plugin_name) immediately after validation — no raw pointers escape
- [ ] Returns `Vec<PluginInfo>` for all valid descriptors in the registry
- [ ] Clear error messages: each LoadError variant includes context (e.g., expected vs actual hash)

## Implementation Notes

### Technical Approach

File: `fidius-host/src/loader.rs`

Key change from spec: the host calls `dlsym("fidius_get_registry")` which returns a `extern "C" fn() -> *const PluginRegistry` (changed in FIDIUS-I-0002). Not a static symbol.

The `Arc<Library>` must stay alive as long as any PluginHandle exists — vtable function pointers point into the loaded library's memory.

### Dependencies
- FIDIUS-T-0013 (error and metadata types)

## Status Updates

- **2026-03-29**: Implemented in `fidius-host/src/loader.rs`. `load_library()` does dlopen → dlsym("fidius_get_registry") → call → validate magic/version → iterate descriptors → copy to owned PluginInfo. `validate_against_interface()` checks hash/wire/strategy. `LoadedLibrary` and `LoadedPlugin` structs hold Arc<Library> + vtable pointer. Compiles clean.