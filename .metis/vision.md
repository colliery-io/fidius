---
id: project-vision
level: vision
title: "Project Vision"
short_code: "FIDIUS-V-0001"
created_at: 2026-03-28T22:32:00.544361+00:00
updated_at: 2026-03-29T00:26:11.442161+00:00
archived: false

tags:
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
initiative_id: NULL
---

# Fidius — Generalized Rust Plugin Framework

## Purpose

Fidius extracts the trait → cFFI → dylib → dynamic loading pattern from cloacina into a standalone Rust library. Plugin authors define a trait, annotate it, and get a compiled dylib with a stable C ABI. Host applications load, validate, and call plugins through a type-safe proxy — no handwritten FFI.

## Product/Solution Overview

**Target audience**: Rust developers who need runtime-extensible applications via dynamic libraries.

**What Fidius provides**:
- A proc macro (`#[plugin_interface]`) that turns a Rust trait into a stable C ABI vtable
- A proc macro (`#[plugin_impl]`) that generates FFI shims for trait implementations
- A host-side library for loading, validating, and calling plugins
- A CLI (`fidius-cli`) for scaffolding interface/plugin crates, signing, verification, and inspection
- Interface evolution via optional methods and compile-time hash checking

**Key differentiator**: The plugin author writes normal Rust traits and impls. Fidius generates all the unsafe FFI machinery, serialization, panic catching, and buffer management. The CLI scaffolds the correct crate topology so plugin authors get a single-dependency experience.

## Current State

The pattern exists inside cloacina, tightly coupled to its workflow/task model. There is no standalone Rust crate that provides this "trait to dylib plugin" pipeline in a general-purpose way.

## Future State

A published crate ecosystem (`fidius`, `fidius-core`, `fidius-macro`, `fidius-host`, `fidius-cli`) that any Rust project can depend on to add a plugin system with:
- Compile-time ABI safety (interface hashing, version checking)
- Multiple plugins per dylib
- Configurable buffer management strategy per interface (caller-allocated, plugin-allocated, arena)
- Debug/release wire format switching (JSON/bincode)
- Feature-gated async support
- Cryptographic plugin signing

## Success Criteria

- A non-cloacina Rust project can define, compile, sign, load, and call a plugin using only fidius crates
- Interface evolution (adding optional methods) works without recompiling existing plugins
- ABI mismatches are caught at load time, never at call time
- Buffer strategy is a trait-level decision that the macro handles end-to-end

## Principles

1. **Safety at the boundary** — All unsafe code is generated and audited in one place. Plugin authors write safe Rust.
2. **No code execution until first call** — Metadata is static data. Loading a plugin reads symbols, never runs plugin code.
3. **Explicit contracts** — Interface hash, ABI version, wire format, and buffer strategy are all checked before any call. Mismatches fail loudly.
4. **Plugin side stays light** — `fidius-core` is the only dependency for plugins. No libloading, no signing verification, no host machinery.
5. **Evolution is additive** — Optional methods are free to add. Required method changes are intentionally breaking.

## Constraints

- Rust-only for v1 (no C/C++/Zig plugin authoring)
- No WASM sandboxing
- Stateless plugins only (no shared memory between host and plugin)
- Host and plugin must be compiled with the same debug/release profile (enforced via wire_format field)