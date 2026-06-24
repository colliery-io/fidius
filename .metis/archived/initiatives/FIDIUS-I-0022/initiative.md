---
id: fidius-guest-wasm-buildable-guest
level: initiative
title: "fidius-guest — wasm-buildable guest runtime split out of host-heavy fidius-core"
short_code: "FIDIUS-I-0022"
created_at: 2026-06-17T11:14:30.392963+00:00
updated_at: 2026-06-17T11:55:42.886707+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: fidius-guest-wasm-buildable-guest
---

# fidius-guest — wasm-buildable guest runtime split out of host-heavy fidius-core Initiative

## Context **[REQUIRED]**

Surfaced by **FIDIUS-I-0021 / FIDIUS-T-0106**: the "macro auto-export" Rust author flow needs a `#[plugin_impl]` crate to compile to a **wasm component**, but `fidius`/`fidius-core` **do not build for wasm**. `cargo check -p fidius-core --target wasm32-wasip2` fails on `bzip2-sys` (a C lib), and `fidius-core` also pulls `tar`/`tempfile` — host-only packaging/signing concerns mixed into the same crate that holds guest-relevant types (`descriptor`, `hash`, `value`, `wire`, `python_descriptor`, `wasm_descriptor`).

A plugin guest — in any language, but specifically a Rust component author — needs only a tiny, `no-host-deps`, wasm-buildable subset: the interface-hash function, the descriptor/`wasm_descriptor` types, the neutral `Value`, and the wire helpers. It must not drag in archive/compression/filesystem crates.

This initiative extracts that subset into a wasm-buildable **`fidius-guest`** crate, so the FIDIUS-T-0106 auto-export adapter (and any Rust component author) can depend on it and compile to wasm. It unblocks T-0106.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A `fidius-guest` crate that compiles cleanly to `wasm32-wasip2`/`wasip1` with **no host-only deps** (no bzip2/tar/tempfile), containing the guest-essential types: `hash` (FNV-1a), `descriptor` + `wasm_descriptor` types, `value` (the neutral `Value` + serde bridge), and `wire`.
- `fidius-core` re-exports / depends on `fidius-guest` for those types, so host code and the existing public API are unchanged.
- The macro's wasm-emitted adapter references `fidius-guest` (wasm-OK), not `fidius`.
- `cargo check --target wasm32-wasip2` passes for `fidius-guest` and for a macro-using author crate.

**Non-Goals:**
- Changing the host-facing API or the cdylib/Python/WASM-host behaviour (this is an internal crate split; re-exports preserve paths).
- Moving signing/packaging (`package`, `signing`, archive) — those stay host-side in `fidius-core`/`fidius-host`.
- Implementing the T-0106 auto-export itself (this only removes its blocker).

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

```
fidius-guest (NEW; wasm-buildable, no host deps)
  ├── hash            (FNV-1a interface hashing)
  ├── descriptor      (PluginDescriptor + repr(C) types, ABI_VERSION)
  ├── wasm_descriptor (WasmInterfaceDescriptor / WasmMethodDesc)
  ├── value           (neutral Value + serde bridge)
  ├── wire            (bincode helpers)
  └── error           (PluginError)            deps: serde, bincode (wasm-OK), thiserror

fidius-core (host)  →  depends on fidius-guest; re-exports its modules so existing
                       paths (fidius_core::descriptor, ::hash, ::value, …) are unchanged.
                       Keeps host-only: package, signing, archive (tar/bzip2/tempfile),
                       registry/inventory.
fidius (facade)     →  unchanged public paths (re-exports flow through).
fidius-macro        →  wasm adapter references fidius-guest (wasm-OK).
```

## Detailed Design **[REQUIRED]**

1. Create `crates/fidius-guest` with **only** wasm-safe deps (`serde`, `bincode`, `thiserror`; no `tar`/`bzip2`/`tempfile`/`tokio`/`inventory`).
2. **Move** the guest-essential modules from `fidius-core` into `fidius-guest`: `hash`, `descriptor`, `wasm_descriptor`, `value`, `wire`, `error` (and `python_descriptor` if it's dep-clean).
3. `fidius-core` adds `fidius-guest` as a dep and `pub use fidius_guest::{hash, descriptor, wasm_descriptor, value, wire, error, …}` so every existing `fidius_core::*` path and the facade re-exports keep resolving — **zero public-API churn**.
4. `inventory`/`registry` stays in `fidius-core` (host collection); the guest doesn't need it.
5. Verify `cargo check -p fidius-guest --target wasm32-wasip2` passes; then a macro-using author crate depending on `fidius-guest` (not `fidius`) builds to a component.

Risk: a moved module may transitively reference a host-only type — resolve by leaving genuinely host-coupled bits in core. `inventory` registration emitted by `#[plugin_impl]` is cdylib-only and must be `#[cfg(not(target_family="wasm"))]`-gated (paired change in T-0106).

## Testing Strategy

- `cargo check -p fidius-guest --target wasm32-wasip2` (the gate that proves the split worked).
- Existing `fidius-core`/host/`fidius-macro` test suites pass unchanged (re-exports preserve behaviour) — the regression proof.
- Wire into `angreal test` + a CI step checking the wasm target builds.

## Alternatives Considered **[REQUIRED]**

- **Make `fidius-core` itself wasm-buildable** (feature-gate the host deps off). Rejected: bzip2/tar are deep transitive/C deps; cfg-gating them cleanly across the crate is more fragile than a clean module split, and still ships a confusingly dual-purpose crate.
- **Guests don't depend on fidius at all** (pure WIT components — what Phase 2 already does). This *is* the lean flow and remains valid; but the chosen FIDIUS-T-0106 "macro auto-export" specifically wants a fidius-macro-using Rust crate to *become* a component, which requires a wasm-buildable fidius dependency — hence this split.

## Implementation Plan **[REQUIRED]**

Single phase (decompose into tasks at the `decompose` transition):
1. Scaffold `fidius-guest` (wasm-safe deps only).
2. Move `hash`/`descriptor`/`wasm_descriptor`/`value`/`wire`/`error` into it.
3. `fidius-core` depends on + re-exports `fidius-guest`; fix internal references; keep public paths.
4. Gate `#[plugin_impl]` cdylib machinery (shims/vtable/descriptor/inventory) to `#[cfg(not(target_family="wasm"))]`; point the wasm adapter at `fidius-guest`.
5. Verify: `wasm32-wasip2` build of `fidius-guest` + a macro author crate; full existing suites green.
6. **Unblocks FIDIUS-T-0106** (resume the auto-export adapter E2E).