---
id: multi-plugin-registry-assembly
level: task
title: "Multi-plugin registry assembly"
short_code: "FIDES-T-0009"
created_at: 2026-03-29T00:53:35.325306+00:00
updated_at: 2026-03-29T00:53:35.325306+00:00
parent: FIDES-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDES-I-0002
---

# Multi-plugin registry assembly

## Parent Initiative

[[FIDES-I-0002]]

## Objective

Enable multiple `#[plugin_impl]` invocations in a single cdylib to produce a single `FIDES_PLUGIN_REGISTRY` that points to all descriptors. T-0008 handles the single-plugin case; this task adds the multi-plugin mechanism.

## Acceptance Criteria

- [ ] Two `#[plugin_impl]` in one cdylib compiles and produces a single `FIDES_PLUGIN_REGISTRY` with `plugin_count = 2`
- [ ] Registry descriptors array points to both descriptors correctly
- [ ] Works on macOS (Mach-O) and Linux (ELF) — the two primary targets
- [ ] Single-plugin case still works (backwards compatible with T-0008)
- [ ] `dlsym("FIDES_PLUGIN_REGISTRY")` returns a valid registry with correct plugin_count

## Implementation Notes

### Technical Approach

Use the `ctor` crate or `#[link_section]`-based approach.

**Option A — `ctor` registration (preferred):**

Add a `fides_core::registry` module with:
```rust
static DESCRIPTORS: Mutex<Vec<*const PluginDescriptor>> = ...;
pub fn register_descriptor(desc: *const PluginDescriptor) { ... }
pub fn build_registry() -> PluginRegistry { ... }
```

Each `#[plugin_impl]` emits a `#[ctor]` function that calls `register_descriptor`. A final `#[ctor]` (or `#[no_mangle] static` with lazy init) assembles the `FIDES_PLUGIN_REGISTRY`.

**Option B — link sections:**

Each `#[plugin_impl]` places its descriptor pointer in a platform-specific link section. A build script or linker script collects them. More complex, less portable.

Going with Option A. Trade-off: runs code at dlopen time for registration, but this is framework bookkeeping, not plugin business logic.

The `fides-core` crate needs a `registry` module that provides the global descriptor collection + the `FIDES_PLUGIN_REGISTRY` export. `#[plugin_impl]` generates the ctor call.

### Dependencies
- FIDES-T-0008 (single-plugin descriptor emission must work first)

### Risk Considerations
- `ctor` ordering is not guaranteed — but we only need all registrations to complete before first use, not a specific order
- Thread safety: `Mutex` around the descriptor vec, or use `std::sync::OnceLock`

## Status Updates

*To be added during implementation*