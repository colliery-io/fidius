---
id: fides-facade-end-to-end-validation
level: initiative
title: "fides Facade + End-to-End Validation"
short_code: "FIDES-I-0004"
created_at: 2026-03-29T00:26:20.084945+00:00
updated_at: 2026-03-29T00:26:20.084945+00:00
parent: FIDES-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
initiative_id: fides-facade-end-to-end-validation
---

# fides Facade + End-to-End Validation

## Context

The facade crate (`fides`) is a thin re-export layer so interface crates have a single dependency. The real work in this initiative is the end-to-end integration test: build an interface crate, implement a plugin as a cdylib, load it from a host binary, call methods, and verify everything works — including optional methods, async, signing, and multi-plugin-per-dylib.

This initiative is the proof that initiatives 1–3 actually work together. It's small in code but high in signal.

Depends on FIDES-I-0001, FIDES-I-0002, FIDES-I-0003.

## Goals & Non-Goals

**Goals:**
- `fides` facade crate: re-export `fides_core::*` types and `fides_macro::*` macros
- Integration test: full pipeline — define trait → implement plugin → compile cdylib → load → validate → call
- Test scenarios:
  - Basic sync method call (required methods)
  - Optional method: present → call succeeds; absent → `supports()` returns false
  - Async method call (feature-gated)
  - Multi-plugin dylib: two impls in one cdylib, load both
  - Signed plugin: sign → load with trusted key → succeeds; load with wrong key → rejected
  - ABI mismatch: load plugin compiled against different interface version → rejected
  - Wire format mismatch: debug plugin in release host → rejected

**Non-Goals:**
- Performance benchmarking (future work)
- CLI integration tests (FIDES-I-0005)

## Detailed Design

### Facade Crate

```rust
// fides/src/lib.rs
pub use fides_core::*;
pub use fides_macro::{plugin_interface, plugin_impl};
```

Cargo.toml re-exports both as public dependencies.

### Integration Test Structure

```
tests/
├── test-interface/        # Interface crate defining TestPlugin trait
│   ├── Cargo.toml
│   └── src/lib.rs
├── test-plugin-basic/     # Single plugin impl (cdylib)
│   ├── Cargo.toml
│   └── src/lib.rs
├── test-plugin-multi/     # Two plugin impls in one cdylib
│   ├── Cargo.toml
│   └── src/lib.rs
├── test-plugin-async/     # Async plugin impl (cdylib)
│   ├── Cargo.toml
│   └── src/lib.rs
└── integration.rs         # Host-side test binary that loads and exercises all plugins
```

The test uses a Cargo workspace or build script to compile the test plugins as cdylibs before running.

## Implementation Plan

1. Create facade crate with re-exports
2. Create test interface crate with a trait covering: required methods, optional methods, async methods
3. Create test plugin crates (basic, multi, async)
4. Write integration test host that exercises all scenarios
5. Add signing test: generate keypair, sign test plugin, verify load/reject behavior
6. Add negative tests: ABI mismatch, wire format mismatch, bad signature