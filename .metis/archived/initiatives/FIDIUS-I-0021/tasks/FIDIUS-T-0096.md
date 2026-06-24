---
id: p1-3-define-pluginexecutor-trait
level: task
title: "P1.3 — Define PluginExecutor trait + neutral fidius_core::Value"
short_code: "FIDIUS-T-0096"
created_at: 2026-06-17T03:23:57.393878+00:00
updated_at: 2026-06-17T03:46:24.574122+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0095]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.3 — Define PluginExecutor trait + neutral fidius_core::Value

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Introduce the `PluginExecutor` trait (the unifying dispatch seam) and a neutral self-describing value type `fidius_core::Value`, per the resolved hybrid design in FIDIUS-I-0021. No backend is wired in this task — this delivers the contract + the value model that all three backends (cdylib, Python, WASM component) will map to.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `PluginExecutor: Send + Sync` defined in `crates/fidius-host/src/executor.rs` with exactly `info`, `method_count`, `call(method, Value) -> Result<Value, CallError>`, `call_raw(method, &[u8]) -> Result<Vec<u8>, CallError>`.
- [x] `fidius_core::Value` neutral enum defined in `crates/fidius-core/src/value.rs` — Component-Model-shaped: `Bool`, distinct `S8..S64`/`U8..U64`, `F32`/`F64`, `Char`, `String`, `Bytes`, `Option`, `List`, `Record`, `Map`, `Variant`, `Unit`. No wasmtime dependency.
- [x] serde bridge: `to_value<T: Serialize>` + `from_value<T: DeserializeOwned>` via a hand-rolled serde `Serializer`/`Deserializer` (covers the full serde data model incl. all four enum-variant kinds). Enables `I → Value → executor.call → Value → O`.
- [x] Round-trip unit tests (6) pass: primitives (incl. `u64::MAX`, `i64::MIN`, `char`), collections (vec/option/tuple/nested-option), structs, string + non-string-keyed maps, all four enum variant kinds, nested. `Value` also implements `Serialize`/`Deserialize` itself.
- [x] Layering rule documented in `value.rs` and `executor.rs`: only the Phase-2 WASM executor maps `Value ↔ wasmtime::component::Val`; cdylib/Python never see wasmtime.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Add a `value` module to `fidius-core` with the enum plus serde `Serializer`/`Deserializer` impls (or follow an existing value-model pattern). Define `PluginExecutor` where it can reference `PluginInfo` + `CallError` — likely `fidius-host` (confirm crate boundaries; `Value` stays in `fidius-core`). Keep `Value` minimal but sufficient for the Canonical ABI value space: bool, char, ints, floats, string, bytes, list, record/map, option, result, variant/enum.

### Dependencies
Depends on FIDIUS-T-0095 (unified `CallError`). Blocks FIDIUS-T-0097 (cdylib executor) and FIDIUS-T-0098 (python executor).

### Risk Considerations
The serde↔Value bridge is the subtle part — ensure it covers the types fidius interfaces actually serialize. Under-modeling `Value` now forces churn in Phase 2 when mapping to `component::Val`; over-modeling adds dead shapes. Aim at exactly the Component Model value space.

## Status Updates **[REQUIRED]**

**2026-06-16 — COMPLETE.**
- `crates/fidius-core/src/value.rs`: `Value` enum (Component-Model-shaped) + hand-rolled serde `Serializer` (`to_value`) and `Deserializer` (`from_value`), plus `Serialize`/`Deserialize` for `Value` itself and `ValueError`. Structs → `Record` (field order preserved); string-keyed maps → `Record`, other maps → `Map`; enum variants → `Variant { name, value }` (unit→`Unit`, newtype→inner, tuple→`List`, struct→`Record`). Exported from `fidius_core` (`Value`, `to_value`, `from_value`, `ValueError`).
- `crates/fidius-host/src/executor.rs`: `PluginExecutor` trait (`call` typed + `call_raw` bytes), exported as `fidius_host::PluginExecutor`.

**Design note:** chose a hand-rolled serde bridge over routing through `serde_json::Value` so the neutral type isn't coupled to serde_json and maps 1:1 to `component::Val` in Phase 2. The distinct integer-width / `char` / `bytes` variants exist for that Phase-2 mapping; the Phase-1 serde bridge populates them faithfully from Rust types.

Verified: `cargo test -p fidius-core value::` → 6/6 pass; `cargo check -p fidius-host` default and `--features python` both clean; clippy on fidius-core clean.