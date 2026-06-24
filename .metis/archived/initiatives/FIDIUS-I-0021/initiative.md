---
id: pluggable-execution-backends
level: initiative
title: "Pluggable execution backends + sandboxed WASM executor (Component Model)"
short_code: "FIDIUS-I-0021"
created_at: 2026-06-17T03:13:15.791614+00:00
updated_at: 2026-06-17T12:33:50.979616+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: pluggable-execution-backends
---

# Pluggable execution backends + sandboxed WASM executor (Component Model) Initiative

## Context **[REQUIRED]**

Fidius today has two plugin execution backends that grew up as **parallel, duplicated implementations**:

- **cdylib** — `fidius-host::PluginHandle` dispatches through a `#[repr(C)]` vtable via FFI, bincode wire for typed args, `#[wire(raw)]` for bulk bytes.
- **Python** — `fidius-python::PluginHandle` (shipped in FIDIUS-I-0020) dispatches through an embedded PyO3 interpreter.

These are two distinct `PluginHandle` types with overlapping responsibilities. Adding a third backend by copy-paste is not viable — hence a **`PluginExecutor` trait** that unifies dispatch, with each backend an implementation behind one host-facing `PluginHandle`.

The third backend is a **sandboxed WASM executor on wasmtime**, whose distinctive value over cdylib (shared address space) and PyO3 (full ambient authority) is **isolation** — and, decisively for this initiative, **polyglot plugin authoring**. The wire boundary was decided in **[[FIDIUS-A-0003]]: Path B — Component Model + WIT**, chosen over the cheaper bincode-over-linear-memory Path A specifically so that plugins can be authored in *any* component-targeting language (Go, JS, C#, C/C++, componentize-py, …), not just Rust. bincode is Rust-specific; a WIT-typed component contract is language-neutral by construction.

A feasibility spike ([[FIDIUS-T-0093]], prototype in `wasm-spike/`, findings in `wasm-spike/FINDINGS.md`) de-risked the runtime: wasmtime sandboxes with deny-FS-by-default (empty-`Linker` proof), instantiates in ~14 µs, and loads AOT `.cwasm` in ~83 µs. Those properties carry over to components — only the boundary marshalling changes.

The spike also surfaced the **central design tension** this initiative must resolve: the cdylib and Python backends share a *raw-bytes* dispatch (`serialize → call_raw → deserialize`, bincode living in `PluginHandle`), but a WIT component boundary is *typed*. So the `PluginExecutor` trait must span both a typed world and a bincode world. See Detailed Design.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A `PluginExecutor` trait behind a single `fidius-host::PluginHandle`; the cdylib and Python backends folded under it with no caller-visible API change.
- A `WasmComponentExecutor` backend on wasmtime that loads, instantiates, and dispatches into a signed `.wasm` **component** defined by a fidius **WIT** interface.
- **Polyglot authoring**: a plugin author writes a component against a fidius WIT interface in any component-targeting language and ships one signed `.wasm`. A Rust author path is the reference, but the contract is language-neutral.
- Sandbox by default: deny filesystem; capabilities granted only as declared typed WASI imports.
- The macro emits the WIT + component target; Ed25519 signing extends to `.wasm` components.
- Toolchain (`cargo-component` + `wasm-tools` + `wasm32-wasip2` target) provisioned in the dev toolchain and pack pipeline. (The main repo has **no Flox env** — it uses ambient rustup/cargo; tools install globally via `cargo install` + `rustup target add`.)

**Non-Goals:**
- Removing or changing the cdylib or Python backends' behaviour (the refactor is structural, not semantic).
- Path A (bincode over linear memory) — rejected in [[FIDIUS-A-0003]]; not an interim either, unless a Review Trigger fires.
- Bundling a wasmtime runtime fidius doesn't already pull; wasmtime is a host-side dependency.
- A hosted plugin registry / distribution service. `.fid` archives remain the unit of distribution.
- Per-call timeouts / cancellation and resource quotas (their own future feature; consistent with FIDIUS-I-0020's deferral).
- Non-Rust author *tutorials* for every language in v1 — the contract supports them; docs ship the Rust reference path plus at least one non-Rust proof.

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

### Overview

```
host binary
└── fidius-host::PluginHost
    └── PluginHandle  (one type; wraps Box<dyn PluginExecutor>)
        ├── CdylibExecutor       — current vtable/FFI dispatch (moved from handle.rs)
        ├── Pyo3Executor         — current PyO3 dispatch (moved from fidius-python)
        └── WasmComponentExecutor — wasmtime component instance + WIT bindings
            ├── Engine (shared)         — compiled Component cache / InstancePre
            ├── Store per call/plugin   — sandbox; capabilities via typed WASI imports
            └── Linker                  — grants ONLY declared imports (deny-FS default)
```

WASM plugin on disk (extends the `.fid` package + `runtime` manifest field FIDIUS-I-0020 added):

```
my-connector/
├── package.toml         # runtime = "wasm", interface = "...", interface_version = N
├── connector.wasm       # a component (precompiled to .cwasm at pack time)
├── world.wit            # the fidius interface this component implements (for tooling/inspect)
└── package.sig          # existing fidius Ed25519 signing over the directory digest
```

### Interaction flow (per call)
1. `PluginHost` routes by manifest `runtime` to the right executor (cdylib / python / wasm) — extends FIDIUS-T-0090's routing.
2. `PluginHandle::call_method<I,O>` is unchanged for callers.
3. For wasm: args are lifted/lowered across the WIT boundary by the Canonical ABI; for cdylib/python the existing bincode round trip runs. The trait reconciles these (see Detailed Design).

## Detailed Design **[REQUIRED]**

### RESOLVED: `PluginExecutor` trait shape — option (iii), hybrid (decided 2026-06-16)

The trait must span a raw-bytes world (cdylib/Python) and a typed world (WIT component). Candidates were (i) typed method, (ii) `list<u8>` bincode blob, (iii) hybrid. **Chosen: (iii) hybrid.**

**Why (ii) was not merely undesirable but unworkable:** bincode is **not self-describing** — decoding requires the concrete Rust type. cdylib/Python have it (they hold the generic `I`/`O` at the call site); the `WasmComponentExecutor` sits behind a `dyn` boundary and does **not**, so it cannot transcode a bincode blob into the typed `component::Val`s the Canonical ABI needs. Its only fallback would be to pass the bincode into the guest as `list<u8>` — the polyglot-killing trap. So a self-describing value across the trait is mandatory for typed calls; that rules out a pure raw-bytes trait.

**Decided trait:**

```rust
pub trait PluginExecutor: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn method_count(&self) -> u32;

    /// Typed dispatch. Args/returns cross as a self-describing value tree
    /// (fidius_core::Value); each backend maps it to its NATIVE boundary:
    ///   cdylib : Value -> bincode -> vtable FFI -> bincode -> Value
    ///   python : Value -> PyObject  -> call     -> PyObject -> Value
    ///   wasm   : Value -> component::Val (Canonical ABI) -> call -> Val -> Value
    fn call(&self, method: usize, args: Value) -> Result<Value, CallError>;

    /// Bulk-bytes path for #[wire(raw)]. Opaque bytes ARE language-neutral
    /// (a WIT `list<u8>`), uniform across all three backends.
    fn call_raw(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError>;
}
```

`PluginHandle::call_method<I,O>` is unchanged for callers: `I → Value` (serde) → `executor.call` → `Value → O`.

**Accepted cost:** one extra `I→Value` hop on the cdylib/Python typed path (today it goes straight to bincode). Negligible — typed calls are small control-plane data; bulk payloads use `call_raw` and skip it.

**Deferred to Phase 2 (implementation detail, not an architecture fork):** the concrete shape of `fidius_core::Value` and its `↔ component::Val` mapping. **Layering rule fixed now:** `Value` is a fidius-owned neutral enum so `fidius-core`/cdylib/Python never take a wasmtime dependency; only `WasmComponentExecutor` knows `component::Val`.

**Migration sketch:** `Pyo3Executor` = `Value→PyObject` → current `fidius-python` dispatch → back; `call_raw` = existing raw path verbatim.

### AMENDMENT (2026-06-16) — enum backend, not `Box<dyn>`; `Value` is NOT the cdylib currency

Implementing T-0097 exposed that a `call(method, Value)` trait method **cannot preserve cdylib's bincode ABI**: `Value` is self-describing and lossy w.r.t. bincode (`Vec` vs tuple, struct vs map, enum index vs name all collapse), so a `dyn`-boundary `CdylibExecutor` cannot reproduce the exact bytes existing compiled plugins decode (`fidius_core::wire` = plain `bincode::serialize`). The Python path is unaffected — it already serialised to self-describing JSON. Re-deriving bincode's wire format from `Value` was rejected as fragile; breaking the cdylib ABI is out of scope (Phase 3).

**Decision (human-ratified):** `PluginHandle` holds an **enum backend**, not `Box<dyn PluginExecutor>`:

```rust
enum Backend { Cdylib(CdylibExecutor), /* #[cfg(python)] */ Python(Pyo3Executor) /*, Wasm(..) Phase 2 */ }
```

`call_method<I,O>` branches with the concrete type in scope, so each backend uses its native currency:
- **cdylib:** `bincode(input) → CdylibExecutor::call_raw(method, bytes) → bincode::<O>` — byte-identical to today, **zero cdylib ABI change**.
- **python/wasm:** `to_value(input) → call(method, Value) → from_value::<O>`.

`PluginExecutor` keeps the truly-common surface (`info`, `method_count`, `call_raw`); the self-describing `call(method, Value)` is implemented by the Value backends (Python, WASM). cdylib's typed path is bincode-wrapping around `call_raw`, so cdylib does not implement the `Value` call. This supersedes the `Box<dyn>` / `CdylibExecutor::call(Value)` wording above.

### Version impact (recorded so it isn't a surprise)

Ships as **fidius 0.3.0**. Per ADR-0002, pre-1.0 `ABI_VERSION = MAJOR*10000 + MINOR*100`, so 0.2.x (=200) → 0.3.0 (=300): a 0.3.0 host rejects 0.2.x plugin binaries with `IncompatibleAbiVersion`. This is the **existing** pre-1.0 policy, not a new break — the Phase-1 refactor is itself wire-compatible (cdylib bytes unchanged, Python contract unchanged). Upgrade story: **recompile/re-pack 0.2.x plugins against 0.3.0** (no source changes). The genuinely breaking surface is the host Rust API (new `PluginRuntimeKind` variants, internal `PluginHandle` change).

### Mechanical pieces (mostly settled by the spike / existing code)
- **Error model.** `fidius-python` returns `PythonCallError`; host returns `CallError`. Unify on `CallError` with a backend-error variant. (Main churn of Phase 1.)
- **Runtime routing.** `PluginInfo.runtime` / `PluginRuntimeKind` already exist (`Cdylib`); add `Python` (retroactively, it currently routes another way) and `Wasm`. Host routing extends FIDIUS-T-0090.
- **WIT ↔ fidius interface mapping.** A fidius `#[plugin_interface]` must project to a WIT `world`/`interface`. Decide whether WIT is generated from the Rust interface (macro emits `.wit`) or authored and checked against the interface hash. Interface-hash-equivalent validation must survive (parity with cdylib's `interface_hash` and FIDIUS-I-0020's Python hash constant).
- **Capabilities.** Empty `Linker` by default (deny FS); a manifest-declared allow-list maps to typed WASI preview2 imports the host wires up. Host is the policy point.
- **Performance posture.** Cache the compiled `Component` / `InstancePre` per plugin; precompile to `.cwasm` at pack time. Fresh `Store` per call is affordable (~14 µs) and buys per-call isolation; offer instance reuse for trusted long-lived plugins.
- **Signing.** Ed25519 over the directory digest is artifact-agnostic — `.wasm` falls in with no crypto changes; only `inspect`/validation learns the new `runtime`.

## Testing Strategy

- **Phase 1 regression safety net:** the existing cdylib + Python integration suites must pass unchanged after the trait refactor — that *is* the proof the refactor is behaviour-preserving. No new caller-visible behaviour.
- **WASM backend:** integration tests that load a signed `.wasm` component through `PluginHost` and call it via the standard `Client`; cover typed args, `#[wire(raw)]` bulk payloads, a guest-raised error → `CallError`, interface-hash mismatch rejection, and capability denial (a guest that tries an un-granted import fails to instantiate/link).
- **Polyglot proof:** at least one non-Rust guest (e.g. TinyGo or componentize-py) implementing the same WIT interface, loaded and called identically — the concrete evidence the polyglot goal is met.
- **Perf check:** assert cold-start and AOT-load stay in the spike's ballpark; extend `pluggable-poc/` with a WASM strategy for apples-to-apples numbers vs native/FFI/PyO3.
- Wired into `angreal test`.

## Alternatives Considered **[REQUIRED]**

- **Path A — bincode over linear memory.** The spike's cheaper recommendation; rejected in [[FIDIUS-A-0003]] because bincode is Rust-specific and would leave WASM plugins effectively Rust-only, defeating the polyglot goal. Not adopted even as an interim.
- **Copy-paste a third `PluginHandle`.** Rejected — compounds the existing cdylib/Python duplication; the trait refactor is the whole point of Phase 1.
- **`list<u8>`-only WIT boundary (option ii).** Tempting (uniform raw trait) but unworkable: bincode is not self-describing, so the `dyn`-boundary WASM executor can't transcode it to typed `component::Val` without passing bincode into the guest — forfeiting polyglot typing. Rejected; see Detailed Design → *RESOLVED trait shape*.
- **Skip the trait, special-case WASM in `PluginHost`.** Rejected — pushes backend specifics into the host and blocks any future backend.

## Implementation Plan **[REQUIRED]**

Three phases. **Phase 1 is the gate** and should land independently (it stands on its own as de-duplication and is the foundation for Phase 2). Per-task decomposition happens at the `decompose` transition — this is the skeleton.

### Phase 1 — `PluginExecutor` trait + toolchain foundation *(gating)*
1. **Toolchain provisioning.** Install `cargo-component` + `wasm-tools` (`cargo install`) and add the `wasm32-wasip2` target; confirm a component builds; document the dev setup. No Flox env in the main repo — ambient rustup/cargo. *(Placed here per decision — it unblocks Phase 2 and is low-risk to land early.)*
2. **Define `PluginExecutor`** per the resolved hybrid shape (Detailed Design → *RESOLVED trait shape*): `call(method, Value) -> Value` + `call_raw(method, &[u8]) -> Vec<u8>`; introduce the neutral `fidius_core::Value`.
3. **Unify the error model** on `CallError` (+ backend variant); migrate `fidius-python`'s `PythonCallError`.
4. **`CdylibExecutor`** — move `fidius-host/src/handle.rs` dispatch behind the trait; `PluginHandle` becomes a `Box<dyn PluginExecutor>` wrapper retaining the inherent typed `call_method<I,O>`.
5. **`Pyo3Executor`** — move `fidius-python/src/handle.rs` dispatch behind the trait.
6. **`PluginRuntimeKind`** gains `Python`/`Wasm`; routing unified.
7. Existing cdylib + Python suites pass unchanged (regression gate).

### Phase 2 — `WasmComponentExecutor` on wasmtime
8. **WIT mapping** — project a fidius interface to a WIT world; decide generated-vs-authored; preserve interface-hash-equivalent validation.
9. **`WasmComponentExecutor`** — wasmtime `Engine`/`Component`/`Linker`/`Store`; `wit-bindgen` host bindings; per-call dispatch implementing the trait.
10. **Capability policy** — empty `Linker` default (deny FS); manifest-declared allow-list → typed WASI imports.
11. **Loader + manifest** — `runtime = "wasm"`; `PluginHost` routes wasm packages; `.cwasm` precompile at load or pack.
12. Integration tests (typed, raw, error, hash-mismatch, capability-denied) + a non-Rust polyglot proof.

### Phase 3 — macro emission + packaging + signing
13. **Macro** — `#[plugin_impl]`/`#[plugin_interface]` emit the WIT + component target for Rust authors.
14. **`fidius pack`** — build/validate the component, precompile `.cwasm`, archive into `.fid`.
15. **Signing + inspect** — Ed25519 over the `.wasm` component (artifact-agnostic); `fidius inspect` understands the wasm runtime.
16. **Docs** — "write your first WASM fidius plugin" (Rust reference path) + the non-Rust walkthrough + capability declaration guide.

### Out-of-scope follow-ons
- Per-call timeouts/cancellation and resource quotas.
- A `WasmCoreModuleExecutor` (Path A) if a Rust-only, lowest-overhead tier is ever wanted alongside components.
- Hot-reload; hosted registry.

## Phase 1 — DONE (2026-06-17)

All seven Phase-1 tasks (T-0094…T-0100) are complete. **Phase 1 stands alone and can merge independently of Phase 2.** Summary:

- **`PluginExecutor` seam landed** as an **enum backend** (`PluginHandle` → `Backend { Cdylib | Python }`, Wasm seat reserved), not `Box<dyn>` — see the amendment above for why (`Value` can't reproduce cdylib bincode).
- **cdylib is byte-identical** to pre-refactor (its typed path stays bincode-direct; `Value` is never on the cdylib path — so the feared `I→Value` hop does **not** apply to cdylib at all; for Python it's a negligible extra round-trip on small control-plane data).
- **Python now flows through the unified `PluginHandle`** (`load_python` returns `PluginHandle`).
- **`fidius_core::Value`** + serde bridge in place for the Phase-2 component boundary.

**Verification (gate):** `angreal test` (full workspace, 36 ok), `angreal lint` (fmt + clippy, exit 0, zero warnings), `angreal python-test` (8 passed), host `--features python` suites (8 passed), `fidius-python` tests, `fidius-core` value round-trips (6). No cdylib/Python test needed behavioural changes (Python tests updated only for the unified call surface).

**Public-API delta (for the 0.3.0 bump):** purely additive for cdylib consumers (`PluginHandle::call_method*` etc. unchanged). Breaking surface = new pub enum variants (`PluginRuntimeKind::Wasm`, `PackageRuntime::Wasm`, `CallError::{WireModeMismatch, Backend}`) and `PluginHost::load_python` now returns `PluginHandle` (was `PythonPluginHandle`). Version **not** bumped in-tree yet (would change `ABI_VERSION` and break the dev test plugins mid-stream); bump to 0.3.0 at release.