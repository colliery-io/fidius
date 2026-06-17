---
id: wasm-user-defined-types-struct
level: initiative
title: "WASM user-defined types — struct/enum support in plugin interfaces via #[derive(WitType)]"
short_code: "FIDIUS-I-0023"
created_at: 2026-06-17T12:52:11.568766+00:00
updated_at: 2026-06-17T13:46:18.114471+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: wasm-user-defined-types-struct
---

# WASM user-defined types — struct/enum support in plugin interfaces via #[derive(WitType)] Initiative

## Context **[REQUIRED]**

Follows FIDIUS-I-0021 (WASM backend + macro auto-export), which supports only WIT-expressible *primitive* types in a wasm plugin's method signatures (bool, ints, f32/f64, char, String, `Vec<T>`, `Option<T>`, `Result<T, PluginError>`). A method using a user `struct`/`enum` fails with a clear compile error (the guard in commit d214ca5).

That limitation is the **sharp edge that blocks adoption**: real interfaces pass domain types (a `Request` struct, a `Mode` enum), so a developer hits it on their first non-trivial wasm plugin. This initiative removes it: user structs → WIT `record`s, user enums → WIT `variant`s, so the same Rust interface that works for cdylib/Python also auto-exports to a component.

**Hard constraint that shapes the design:** a proc-macro on the trait/impl sees only method *signatures* (type names), never field/variant definitions (which live at the `#[derive]` sites), and `wit_bindgen::generate!{inline}` needs the complete WIT as a literal at expansion. So records/variants **cannot** be assembled inside the macro — the WIT must be produced by a step that parses the source. **Decision (human, 2026-06-17):** a `build.rs` auto-generates `wit/` from the crate source each build; the adapter consumes it via `generate!{ path: "wit" }`; plain `cargo build --target wasm32-wasip2` still works.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- `#[derive(WitType)]` on an author's `struct` → WIT `record`, `enum` → WIT `variant`, usable in `#[plugin_interface]` method signatures.
- A source-parsing WIT generator (trait + all `WitType` types → a complete `wit/`), runnable from a `build.rs` helper (auto) and a `fidius wit` CLI.
- The adapter consumes the generated `wit/` (`generate!{ path }`) and maps the WIT types onto the author's Rust types (wit-bindgen `with`-mapping) so there's no parallel mirror to hand-convert.
- E2E: a wasm plugin whose interface uses a record + a variant builds, loads via `load_wasm`, round-trips. cdylib/Python plugins with the same types keep working (bincode already handles them).

**Non-Goals:**
- Resources/handles, generics, borrowed fields (owned records/variants only in v1).
- Changing the cdylib/Python type story (they already handle arbitrary serde types).
- WIT-first authoring (defining types in `.wit`, generating Rust) — this is the Rust-first direction.

## Detailed Design **[REQUIRED]**

```
author crate (wasm plugin)
  src/lib.rs:  #[derive(WitType)] struct Request { ... } / enum Mode { ... }
               #[plugin_interface] trait Svc { fn run(&self, r: Request) -> Mode; }
               #[plugin_impl] impl Svc for MyPlugin { ... }
  build.rs:    fidius_build::emit_wit();   // parses src → writes wit/<iface>.wit
  wit/:        (generated) package + record/variant + funcs + fidius-interface-hash
        │
        ▼  cargo build --target wasm32-wasip2
  #[plugin_impl] adapter:  wit_bindgen::generate!({ path: "wit", world, with: {
        "fidius:svc/svc/request": crate::Request,   // use the author's type, no mirror
        "fidius:svc/svc/mode":    crate::Mode,
  }}) + Guest impl forwarding to the instance.
```

1. **WIT generator (`fidius-wit` lib)** — parse Rust source with `syn`: find the `#[plugin_interface]` trait + every `#[derive(WitType)]` item. Map `struct {f: T,…}` → `record name { f: wit(T), … }`, fieldless/data enums → `variant name { case(payload?), … }`, reusing/extending the `Type→WIT` mapper from `fidius-macro::wit`. Emit a complete `wit/<iface>.wit` (package + records + variants + funcs + `fidius-interface-hash`). Topo-order/inline nested types. Shared by the CLI + build helper.
2. **`#[derive(WitType)]`** (in `fidius-macro`) — marks the type for the generator and emits whatever wit-bindgen's `with`-mapping requires so the author's own type is used directly (no separate generated mirror to convert). **De-risk first:** spike the exact wit-bindgen 0.44 `with` mechanics (does it need a derive? matching field idents? `#[component]`-style attrs?) against a hand-written `wit/` before building the generator.
3. **`fidius-build` helper + `fidius wit` CLI** — `fidius_build::emit_wit()` for `build.rs` (auto, per the decision) and a `fidius wit [dir]` CLI subcommand (manual/CI). Both call the generator lib.
4. **Adapter rework** (`fidius-macro` impl side) — switch the wasm adapter from `generate!{inline}` to `generate!{ path: "wit", with: {…} }`, deriving the `with` map entries from the interface's user-typed parameters. Primitives-only interfaces keep working (empty `with`).
5. **Fixtures + E2E** — a `records-greeter` wasm fixture using a `record` arg + a `variant` return; load via `load_wasm`, round-trip. Regression: macro-greeter (primitives) still builds; cdylib/Python with the same types still pass.
6. **Docs** — extend the WASM ABI explanation (record/variant mapping) + the Rust tutorial (`#[derive(WitType)]` + `build.rs`).

## Spike Result (2026-06-17) — mechanism PROVEN

De-risked in a throwaway crate (`record point` + `variant shape { circle(u32), rect(point), dot }`), built to `wasm32-wasip2`:
- **`with`-remapping does NOT work for *exported* interface types** (`error: unused remappings`). `with` is for *imported* types. So the guest cannot reuse the author's structs directly — it must use wit-bindgen's **generated** types for the exported interface.
- wit-bindgen generates `record`→struct, `variant`→enum for the export; implementing `Guest` against them compiles and the component exports the real WIT record/variant (confirmed via `wasm-tools component wit`).
- **Generated↔author `From` conversions** compile, including nested user types via `.into()` recursion:
  ```rust
  impl From<wit::Point> for MyPoint { fn from(w) -> Self { Self { x: w.x, y: w.y } } }
  impl From<wit::Shape> for MyShape { fn from(w) -> Self { match w {
      wit::Shape::Rect(p) => Self::Rect(p.into()), ... } } }  // nested recurses
  ```
  The Guest converts args (`generated.into()` → author) before calling the trait, and the return (`author.into()` → generated).

**Adjusted mechanism (vs the initiative's original `with`-mapping note):** the generator emits the `wit/` **and** a conversions module (`From` both directions for every `WitType`, recursing via `.into()`). The path to the generated type is deterministic (`exports::<ns>::<pkg>::<iface>::<Type>`), so the generator can name it. The adapter does `generate!{ path }` + includes the conversions + `.into()` at the boundary. This keeps polyglot fidelity (real records cross the wire) at the cost of a mechanical conversion layer.

## Testing Strategy

- Unit: `Type→WIT` for record/variant/nested in the generator; golden `.wit` output for a sample interface.
- Integration: build the `records-greeter` fixture to `wasm32-wasip2`, `wasm-tools validate`, dump WIT, assert the record/variant exports.
- E2E: `load_wasm` the fixture, call a record-in/variant-out method, assert the round-trip; full native regression stays green.

## Alternatives Considered **[REQUIRED]**

- **Keep `generate!{inline}`, assemble records in the macro.** Impossible: the impl macro can't see external type definitions, and `inline` needs a literal — the core constraint above.
- **Manual `fidius wit` step (no build.rs).** Viable and simpler, but a forgettable manual step that drifts; rejected as the default in favor of `build.rs` auto-gen (human decision). The CLI is still shipped for CI/manual use.
- **WIT-first (author writes `.wit`, generates Rust).** The idiomatic wit-bindgen direction and zero conversion, but it abandons "write a normal Rust trait" — contradicts fidius's authoring model. Rejected.
- **Generate a mirror type + hand-write conversions.** Doubles every type and is error-prone; the wit-bindgen `with`-mapping avoids it by using the author's type directly.

## Implementation Plan **[REQUIRED]**

Single phase; decompose into tasks at `decompose`:
1. **Spike** the wit-bindgen `with`-mapping mechanics against a hand-written `wit/` (de-risk step 2).
2. WIT generator lib (`syn` source-parse → record/variant/funcs).
3. `#[derive(WitType)]` + adapter rework (`generate!{path, with}`).
4. `fidius_build::emit_wit()` + `fidius wit` CLI.
5. `records-greeter` fixture + E2E + regression.
6. Docs.