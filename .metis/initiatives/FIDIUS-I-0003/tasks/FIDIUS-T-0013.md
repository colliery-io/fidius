---
id: loaderror-types-and-plugininfo
level: task
title: "LoadError types and PluginInfo owned metadata"
short_code: "FIDIUS-T-0013"
created_at: 2026-03-29T01:28:30.111978+00:00
updated_at: 2026-03-29T11:19:39.600963+00:00
parent: FIDIUS-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0003
---

# LoadError types and PluginInfo owned metadata

## Parent Initiative

[[FIDIUS-I-0003]]

## Objective

Define the error and metadata types that fidius-host uses. `LoadError` is the enum returned by all loading/validation operations — one variant per failure mode. `PluginInfo` is the owned metadata struct copied from FFI descriptor data (no raw pointers). `LoadPolicy` controls strictness.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `LoadError` enum with variants: `LibraryNotFound`, `SymbolNotFound`, `InvalidMagic`, `IncompatibleRegistryVersion`, `IncompatibleAbiVersion`, `InterfaceHashMismatch`, `WireFormatMismatch`, `BufferStrategyMismatch`, `SignatureInvalid`, `SignatureRequired`, `PluginNotFound`, `LibLoading(libloading::Error)`
- [ ] `LoadError` implements `std::error::Error` + `Display` via thiserror
- [ ] `PluginInfo` struct with owned fields: `name: String`, `interface_name: String`, `interface_hash: u64`, `interface_version: u32`, `capabilities: u64`, `wire_format: WireFormat`, `buffer_strategy: BufferStrategyKind`
- [ ] `LoadPolicy` enum: `Strict` (reject any mismatch/missing sig), `Lenient` (warn but allow unsigned)
- [ ] All types derive Debug, Clone where appropriate
- [ ] Compiles and unit tests pass

## Implementation Notes

### Technical Approach

File: `fidius-host/src/error.rs` for LoadError, `fidius-host/src/types.rs` for PluginInfo and LoadPolicy.

### Dependencies
- None — pure type definitions

## Status Updates

- **2026-03-29**: Implemented. `LoadError` (12 variants + Io), `CallError` (6 variants), `PluginInfo`, `LoadPolicy` all in fidius-host. Added thiserror/serde/serde_json deps. Compiles clean.