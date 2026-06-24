---
id: p3-1-macro-emits-wit-component
level: task
title: "P3.1 — Macro emits WIT + component target + WasmInterfaceDescriptor from the Rust interface"
short_code: "FIDIUS-T-0106"
created_at: 2026-06-17T09:50:06.607031+00:00
updated_at: 2026-06-17T12:00:45.579598+00:00
parent: FIDIUS-I-0021
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.1 — Macro emits WIT + component target + WasmInterfaceDescriptor from the Rust interface

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Make the fidius macros generate the WASM artifacts a Rust plugin author needs — fulfilling the T-0101 deferral (Phase 2 used a hand-authored WIT; Phase 3 generates it). `#[plugin_interface]` emits a `WasmInterfaceDescriptor` const for the host and the interface's WIT; `#[plugin_impl]` emits the component-side export glue so a Rust author builds a conforming component with the standard toolchain.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `#[plugin_interface]` emits a `<Trait>_WASM_DESCRIPTOR: fidius_core::wasm_descriptor::WasmInterfaceDescriptor` const (interface_export, interface_hash, methods + wire_raw) — parity with the Python descriptor.
- [x] The interface's `.wit` is generated (macro-emitted, inline, via `crates/fidius-macro/src/wit.rs`) per the T-0101 mapping: types, `#[wire(raw)]`→`list<u8>`, fallible→`result<T, plugin-error>`, and the `fidius-interface-hash` export. Validated by `wasm-tools` (the macro-greeter component validates).
- [x] `#[plugin_impl]` emits the component export glue: a `wit_bindgen::generate!{inline}` + `Guest` impl wiring the static instance + `export!`, under `#[cfg(target_family="wasm")]`, exporting `fidius-interface-hash`.
- [x] A Rust author builds a component for a **macro-generated** interface (`tests/wasm-fixtures/macro-greeter`) and it loads + calls through `PluginHost::load_wasm` against the generated `Greeter_WASM_DESCRIPTOR`. E2E test (`tests/macro_wasm.rs`, `--features wasm`).
- [x] The descriptor's `interface_hash` and the component's exported `fidius-interface-hash` derive from the same macro computation and match (verified at load — `load_wasm` validates and the E2E loads successfully).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Extend `crates/fidius-macro` (interface.rs + the impl macro). Reuse the existing IR that already parses the trait and computes the FNV-1a `interface_hash` (`fidius_core::hash`). Emit `WasmInterfaceDescriptor` analogously to the existing `PythonInterfaceDescriptor` codegen (FIDIUS-T-0085/0086 era). For WIT: emit a `.wit` string from the IR per the T-0101 type table — likely a `fidius wit` CLI subcommand (mirrors `fidius python-stub`, FIDIUS-T-0091) rather than inline proc-macro file IO. For the component glue, settle author ergonomics: most likely the author uses `cargo component` with the generated `.wit` + a thin generated Guest adapter, since cargo-component already does WIT→bindings.

### Dependencies
Phase 2 (executor, loader, `wasm_descriptor`) DONE. Blocks [[FIDIUS-T-0107]] (pack needs a real macro-built component) and [[FIDIUS-T-0109]]. **Design sub-decision to settle at start:** how much the macro emits vs the component toolchain provides — cargo-component already turns WIT into bindings, so the macro's job is the `.wit` + the descriptor + a thin Guest adapter, not a full bindings generator.

### Risk Considerations
The generated WIT must exactly match the host's `Value↔component::Val` mapping and the hand-authored reference WIT, or loads fail. The descriptor hash and the component's exported hash must derive from one source. Component-author ergonomics vary by toolchain; keep the flow close to cargo-component norms. Proc-macro file IO is awkward — prefer a CLI subcommand for `.wit` emission.

## Status Updates **[REQUIRED]**

**2026-06-17 — IN PROGRESS. Descriptor done; author-build-flow fork needs a decision.**

Done + verified:
- `#[plugin_interface]` now emits `<Trait>_WASM_DESCRIPTOR: WasmInterfaceDescriptor` (kebab-case method names + `interface_export = "fidius:<iface>/<iface>@<version>.0.0"`), parallel to the Python descriptor. `to_kebab_case` helper added. Facade re-exports `fidius::wasm_descriptor`.
- Verified: workspace + out-of-workspace `test-plugin-smoke` compile with the new descriptors; `fidius-macro` tests all pass (no snapshot breakage).

**Fork to settle (the rest of T-0106 hinges on it):** how a Rust author's interface becomes a *loadable component*. Two hard sub-problems:
1. **WIT from arbitrary Rust types.** The greeter WIT was hand-authored using only primitives/string/`list<u8>`. Auto-generating WIT for user structs/enums (→ WIT `record`/`variant`) is a substantial feature.
2. **Component build glue.** A `#[plugin_impl]` trait must end up exported from a wasm *component*:
   - **(A) Lean / ecosystem-aligned:** `fidius wit` CLI emits the `.wit` (like `fidius python-stub`); author builds with `cargo component` + a thin generated `Guest` adapter.
   - **(B) Heavy:** macro emits a full `wit-bindgen generate!/export!` adapter; more codegen, couples to wit-bindgen, still needs the author's crate set up as a component.

**Phase 2 already proved the host loads/runs any conforming component** (Rust + Python), so T-0106 is author *ergonomics* sugar.

**DECISION (human, 2026-06-17): Option B — full macro auto-export.** `#[plugin_impl]` should emit a `wit-bindgen generate!/export!` adapter so the author's trait impl auto-exports as a component.

**Concrete implementation plan (the remaining build):**
1. **WIT generator** — the IR (`MethodIR`) currently keeps only the `signature_string`, *not* parsed `syn::Type`s. Extend the interface IR to retain typed args/returns (the impl macro's `MethodInfo` already has `arg_types: Vec<&Type>`), and write a `Type → WIT` mapper for the supported set (bool, sized ints, f32/f64, char, String, `Vec<u8>`→`list<u8>`, `Vec<T>`→`list<T>`, `Option<T>`, `Result<T, PluginError>`→`result<T, plugin-error>`). Emit the inline WIT string. **Constraint:** a proc-macro can't introspect external `struct`/`enum` definitions, so user records/variants need a separate `#[derive(WitType)]` — a documented follow-on; v1 supports the primitive/string/bytes/list/option/result set.
2. **Adapter codegen** — `#[plugin_impl]` emits, under `#[cfg(target_family = "wasm")]`: `wit_bindgen::generate!({ inline, world })` + `impl exports::…::Guest for __Component` wiring each method to the existing `static __FIDIUS_INSTANCE_<T>: <T> = <T>;` (plugins are unit structs — confirmed), + `fidius-interface-hash` returning the hash const, + `export!`. Map `Result<_, PluginError>` ↔ the WIT `plugin-error` record.
3. **wit-bindgen dependency** — generated code references `wit_bindgen`; re-export it (`fidius::wit_bindgen`) or require authors to add it.
4. **Author fixture + E2E** — a new macro-using plugin crate built with `cargo component`; load the resulting component via `load_wasm` against the macro-emitted `<Trait>_WASM_DESCRIPTOR`; assert the hash matches.

**Progress (pushing through, per human):**
- ✅ `WasmInterfaceDescriptor` emission + facade re-export + naming convention (export name now fixed `@0.1.0`, decoupled from interface version so the impl macro can match it). Verified.
- ✅ **Step 1 — WIT generator** (`crates/fidius-macro/src/wit.rs`): `rust_type_to_wit` (bool/sized-ints/f32/f64/char/String/&str/`Vec<T>`/`&[T]`/`Option<T>`), `return_to_wit` (`Result<T, PluginError>`→`result<T, plugin-error>`, unit→none), `render_wit` (full package/interface/world + `plugin-error` record + `fidius-interface-hash`). Unsupported types → clear compile error. **5 unit tests pass.** `MethodInfo` now captures the return type.
- ⏳ **Step 2 — adapter codegen** (`#[plugin_impl]` emits `wit_bindgen::generate!{inline}` + `Guest` impl wiring the static unit-struct instance + `export!`, under `#[cfg(target_family="wasm")]`; emitted best-effort — skipped if a type is unsupported so cdylib/python are never broken).
- ⏳ **Step 3/4** — wit-bindgen dep, author fixture crate, component build, E2E load.

Steps 2–4 are the high-risk integration. Continuing surfaced a **blocking architectural finding** (verified):

**`fidius`/`fidius-core` do NOT build for wasm.** `cargo check -p fidius-core --target wasm32-wasip2` fails on `bzip2-sys` (a C lib); `fidius-core` also pulls `tar`/`tempfile` — all host-only packaging deps. So the "heavy auto-export" premise — a `#[plugin_impl]` crate that *also* compiles to a component — is **architecturally blocked**: that crate depends on `fidius`, which can't compile to wasm.

This reframes the chosen path: making a fidius-dependent crate *become* a component requires a **wasm-buildable fidius guest crate** (split the macro-runtime essentials — interface-hash const, descriptor types, wit-bindgen glue — out of the host-heavy `fidius-core`). That split is a substantial architectural change, **larger than and outside T-0106's scope** (arguably its own initiative).

**Key realization:** the WASM guests that already work (Phase-2 Rust greeter via cargo-component; greeter-py via componentize-py) depend on **nothing** from fidius — they just implement the WIT. That *is* the natural architecture (components are independent artifacts implementing a WIT contract). The macro's genuinely useful contribution is **generating the `.wit`** (the WIT generator — done, tested — feeds a `fidius wit` CLI) + the descriptor; not turning the author's crate into a component.

**What's emitted now:** the adapter codegen is written and compiles on non-wasm (it's `#[cfg(target_family="wasm")]`), but it cannot be E2E-verified because no fidius-dependent crate compiles to wasm. Pausing to surface this — the heavy path needs an architectural prerequisite the user should weigh in on (pivot to the lean `fidius wit` + independent-component flow, or commit to the fidius-core split as a separate initiative).

**2026-06-17 — COMPLETE (after FIDIUS-I-0022 unblocked it).** The `fidius-guest` split made a macro-using crate wasm-buildable. With the cdylib codegen gated off wasm (T-0111) and two adapter fixes (`use super::*;`; hash const read from the companion module `__fidius_<Trait>`), the full loop is proven end-to-end:
- `tests/wasm-fixtures/macro-greeter` (`#[plugin_interface]`+`#[plugin_impl]`, `crate = "fidius_guest"`) builds to a valid 44 KB component exporting `fidius:greeter/greeter@0.1.0` (wasm-tools validate ✓).
- `crates/fidius-host/tests/macro_wasm.rs` (`--features wasm`): loads that component via `PluginHost::load_wasm` against the **macro-emitted** `Greeter_WASM_DESCRIPTOR`; `greet("Ada")`→`"Hello, Ada!"`, raw `echo` reverses bytes, and the load's hash-validation passes (descriptor hash == component's `fidius-interface-hash`). 2/2 E2E + 14 other wasm tests pass; native suite green (40 ok).
- CI `wasm` job builds macro-greeter as a regression guard.

**Author flow now:** write a trait + impl with the fidius macros (`crate = "fidius_guest"`) → `cargo build --target wasm32-wasip2` → a sandboxable fidius component. **Scope note (documented follow-on):** WIT auto-generation covers the primitive/string/bytes/list/option/result set; user `struct`/`enum` types need a future `#[derive(WitType)]` (a proc-macro on the trait can't introspect external type defs).