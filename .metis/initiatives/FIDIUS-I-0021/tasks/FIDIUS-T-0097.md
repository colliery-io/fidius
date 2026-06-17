---
id: p1-4-cdylibexecutor-migrate-fidius
level: task
title: "P1.4 — CdylibExecutor: migrate fidius-host dispatch behind the trait; PluginHandle wraps Box<dyn PluginExecutor>"
short_code: "FIDIUS-T-0097"
created_at: 2026-06-17T03:23:59.276704+00:00
updated_at: 2026-06-17T04:01:56.288909+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0096]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.4 — CdylibExecutor: migrate fidius-host dispatch behind the trait; PluginHandle wraps Box<dyn PluginExecutor>

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Move the existing cdylib vtable/FFI dispatch out of `PluginHandle` into a `CdylibExecutor` that implements `PluginExecutor`, and make `PluginHandle` a thin wrapper over `Box<dyn PluginExecutor>` that retains its inherent generic `call_method<I,O>` / `call_method_raw`. Behaviour-preserving refactor.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

> Note: AC reworded for the enum-backend correction (see Status Updates / FIDIUS-I-0021 amendment). The `Box<dyn>` / `CdylibExecutor::call(Value)` wording in the original is superseded.

- [x] `CdylibExecutor` (`executor/cdylib.rs`) is the cdylib backend — the old `PluginHandle` FFI/arena/metadata code moved verbatim and renamed. Implements `PluginExecutor` (`info`/`method_count`/`call_raw`). It keeps its own generic `call_method<I,O>` (bincode directly), so cdylib never touches `Value`.
- [x] `PluginHandle` holds an **enum `Backend`** (not `Box<dyn>`); `call_method<I,O>` matches → cdylib runs `e.call_method` (bincode, byte-identical to pre-refactor); `call_method_raw`/`info`/`has_capability`/`method_metadata`/`trait_metadata` delegate.
- [x] `from_loaded`, `from_descriptor`, and `find_in_process_descriptor` construct/route to `CdylibExecutor`.
- [x] No public API change: `Client`, `Client::in_process`, and all `PluginHandle` method signatures unchanged; `PluginHandle` stays `Send + Sync` (auto-inherited from `CdylibExecutor`).
- [x] `cargo clippy -p fidius-host` clean; integration (15, incl. raw-wire/arena/multi-arg/metadata-through-handle/capability) + e2e (6) pass unchanged.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Refactor `crates/fidius-host/src/handle.rs`: extract dispatch into an `executor/cdylib.rs`. Move the arena/buffer-strategy and free_buffer logic with the executor. `PluginHandle` becomes wrapper + inherent typed methods. Preserve the existing `unsafe impl Send/Sync` reasoning and move the `_library: Arc<Library>` lifetime ownership into the executor so the dylib stays loaded for the handle's life.

### Dependencies
Depends on FIDIUS-T-0096 (trait + Value). Paired with FIDIUS-T-0098 (python executor); both feed FIDIUS-T-0099 (routing) and FIDIUS-T-0100 (regression gate).

### Risk Considerations
Highest-churn file in Phase 1. Preserve the unsafe Send/Sync invariants and arena dispatch exactly. The `Value→bincode→Value` typed path must be behaviourally identical to today's direct bincode round-trip. Keep `PluginHandle::new` crate-private semantics.

## Status Updates **[REQUIRED]**

**2026-06-16 — BLOCKED: the settled trait shape can't preserve the cdylib bincode ABI.**

Implementation surfaced a flaw in the trait shape settled in FIDIUS-A-0003 / FIDIUS-I-0021. The trait is `call(method, Value) -> Value`, so `PluginHandle::call_method<I,O>` would `to_value(input) → executor.call(Value)`, and `CdylibExecutor` must re-serialise that `Value` to bincode for the FFI. But a `Value` **cannot reproduce cdylib's exact bincode bytes**, which the existing plugin shim decodes into concrete types (`fidius_core::wire` = plain `bincode::serialize`; fixint/LE/u64-length):
- `Vec<T>` (length-prefixed) and `(T, T)` (no prefix) both become `Value::List` — ambiguous.
- structs (fields, no length) vs maps (length-prefixed) — collapsed by `to_value`.
- enums are bincode-encoded by numeric variant **index**; `Value::Variant` carries the **name**.

So Value→bincode can't be byte-identical without re-implementing bincode's wire format from Value (fragile, version/config-coupled) or changing the cdylib plugin ABI (Phase 3 + breaking). The Python path avoids this only because it already serialised to self-describing JSON.

**Proposed fix (awaiting human decision — it changes a settled decision):** keep cdylib's typed path *type-driven* (serialize concrete `I` with bincode directly) and reserve `Value` for the self-describing backends (Python, WASM). Cleanest is an **enum backend** on `PluginHandle` (`Cdylib(..)|Python(..)|Wasm(..)`) so `call_method<I,O>` branches with the concrete type in scope:
- cdylib: `bincode(input) → FFI → bincode::<O>` (byte-identical to today; zero ABI change)
- python/wasm: `to_value(input) → call(Value) → from_value::<O>`

T-0098/0099/0100 also depend on this. Loop paused for the decision; T-0097 → blocked.

**2026-06-17 — UNBLOCKED (human chose enum backend) and COMPLETE.**
- `executor.rs`: split into `PluginExecutor` (common: `info`/`method_count`/`call_raw`) + `ValueExecutor` (adds `call(method, Value)`, for Python/WASM only). Added `pub mod cdylib`.
- `executor/cdylib.rs`: the old `PluginHandle` body moved + renamed `CdylibExecutor` (FFI/arena/plugin-allocated dispatch, metadata accessors, `from_loaded`/`from_descriptor`/`find_in_process_descriptor` constructors). Implements `PluginExecutor`; keeps its generic `call_method<I,O>` so the cdylib typed path is bincode-direct.
- `handle.rs`: rewritten as the thin `PluginHandle` wrapping `enum Backend { Cdylib(CdylibExecutor) }` (Python variant lands in T-0098). All public methods preserved and delegate.

Result: cdylib bytes unchanged; no caller-visible API change. `cargo check`/`clippy` clean; `cargo test -p fidius-host --test integration --test e2e` → 15 + 6 pass. T-0097 → completed.