<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# WASM Component ABI — mapping a fidius interface to WIT

> Status: **design (FIDIUS-I-0021 Phase 2, T-0101)**. The WASM executor (T-0102)
> and loader (T-0103) implement this. Path B (Component Model + WIT) was chosen
> in ADR FIDIUS-A-0003 for polyglot authoring.

This explains how a fidius plugin interface projects onto a WebAssembly
**component** described in **WIT**, how the host dispatches calls into it, and
how it stays consistent with the cdylib and Python backends.

## Bindings strategy (decided)

**For Phase 2: hand-authored WIT + dynamic dispatch. No build-time codegen.**

- The plugin's contract is a `.wit` file. A plugin author (in any language)
  implements that world and ships a `.wasm` component.
- The **host** does *not* generate per-interface Rust bindings with
  `wit-bindgen`. Instead `WasmComponentExecutor` dispatches **dynamically**
  through wasmtime's `component::Func::call(&mut store, &[Val], &mut [Val])`,
  looking the export up by name (declaration order = the same index the cdylib
  vtable and the Python loader use). This matches fidius's existing
  by-index `call_method(index, ..)` model and avoids a build-time codegen step
  in the host.

**Deferred to Phase 3 (T-0013):** the `#[plugin_interface]` macro emitting the
`.wit` from the Rust trait, so Rust authors get WIT for free. The dynamic host
path is unaffected by that — it consumes any conforming component.

Rationale: dynamic `Val` keeps the host generic over interfaces (one code path
for all plugins, dispatched by index), mirrors the cdylib/Python dispatch, and
sidesteps committing the host build to a specific WIT toolchain version. The
cost — runtime `Value ↔ Val` marshalling instead of compile-time-typed
bindings — is the same shape of work the other backends already do.

## Dispatch model

A fidius trait method at vtable index `i` maps to a component export, in
declaration order. The host's generic `call_method<I, O>` already tuple-packs a
method's arguments into one value; through the executor that becomes:

```
call_method::<I,O>(i, &args)
  -> to_value(&args)            // I -> fidius_core::Value  (Value::List of positional args)
  -> WasmComponentExecutor::call(i, value)
       -> map Value::List elements -> &[component::Val]   (positional params)
       -> func_i.call(&mut store, &params, &mut results)
       -> results[0] : component::Val -> fidius_core::Value
  -> from_value::<O>(value)     // Value -> O
```

`#[wire(raw)]` methods bypass the typed path: `call_method_raw(i, &[u8])`
dispatches an export whose signature is `func(list<u8>) -> list<u8>`, so opaque
bulk bytes cross as a WIT `list<u8>` with no per-element marshalling. Opaque
bytes are language-neutral, so this stays uniform with cdylib/Python.

## Type mapping (fidius `Value` ↔ WIT)

The `fidius_core::Value` variant set was deliberately shaped to the Component
Model value space (FIDIUS-T-0096), so the mapping is close to 1:1:

| fidius `Value` | serde / Rust source | WIT type | wasmtime `Val` |
|---|---|---|---|
| `Bool` | `bool` | `bool` | `Bool` |
| `S8/S16/S32/S64` | `i8..i64` | `s8/s16/s32/s64` | `S8..S64` |
| `U8/U16/U32/U64` | `u8..u64` | `u8/u16/u32/u64` | `U8..U64` |
| `F32/F64` | `f32/f64` | `f32/f64` | `Float32/Float64` |
| `Char` | `char` | `char` | `Char` |
| `String` | `String/&str` | `string` | `String` |
| `Bytes` | `&[u8]` (serde_bytes) | `list<u8>` | `List` of `U8` |
| `List` | `Vec<T>`, tuples | `list<T>` / `tuple<..>` | `List` / `Tuple` |
| `Record` | structs, string maps | `record { .. }` | `Record` |
| `Option(_)` | `Option<T>` | `option<T>` | `Option` |
| `Variant{name,..}` | enums | `variant { .. }` / `enum` | `Variant` / `Enum` |
| `Unit` | `()`, unit structs | (empty tuple) | `Tuple([])` |

Notes:
- `Value::Map` (non-string-keyed maps) has no direct WIT type; encode as
  `list<tuple<K, V>>`. Most fidius interfaces use structs (`Record`), so this is
  rare.
- The executor derives each export's expected param/result **types** from the
  component's own type information (wasmtime exposes them), and uses those to
  drive the `Value → Val` lowering (e.g. knowing a `u32` param vs `s64`).

## User-defined types — records & variants (FIDIUS-I-0023)

A plugin author's own `struct`/`enum` types in an interface map to WIT
`record`/`variant` when annotated with `#[derive(WitType)]`:

| Rust | WIT |
|---|---|
| `struct P { x: i32, y: i32 }` | `record p { x: s32, y: s32 }` |
| `enum S { Circle(u32), Rect(P), Dot }` | `variant s { circle(u32), rect(p), dot }` |

Two constraints shape the implementation:

1. **A proc-macro can't see external type definitions.** `#[plugin_impl]` sees
   only the method *signatures* (type names), not the fields of `P`. And
   `wit_bindgen::generate!{ inline }` needs the complete WIT as a literal at
   expansion. So the records/variants can't be assembled inside the macro — they
   are generated from the **source** by a build step (`fidius_build::emit_wit()`
   in `build.rs`, sharing the `fidius-wit` generator with the `fidius wit` CLI).
   It writes `wit/<iface>.wit`; the adapter consumes it via
   `generate!{ path: "wit" }`.

2. **wit-bindgen won't remap an *exported* interface's types onto your structs**
   (its `with` option is for imports). The guest therefore uses wit-bindgen's
   *generated* types, and `fidius-wit` also emits `From` conversions both ways
   (`exports::…::P ↔ crate::P`, recursing through `Vec`/`Option`/nested types).
   The `build.rs` writes them to `$OUT_DIR`; the adapter `include!`s them and
   converts at the `Guest` boundary. `#[derive(WitType)]` itself is just a marker
   the generator reads — it emits no code.

**Name normalization.** WIT uses kebab-case; serde produces snake_case fields and
PascalCase enum variants. The executor normalizes record-field and variant-case
names at the `Value ↔ Val` boundary (`to_kebab` inbound; `kebab → snake` for
fields and `kebab → PascalCase` for variants outbound), so a host `Shape::Circle`
matches the WIT `circle` case and a `y_pos` field matches `y-pos`.

The same `#[derive(WitType)]` type still crosses the **cdylib/Python** boundary
unchanged (via serde/bincode) — the records/variants are the WASM projection
only. v1 limits: named-field records; unit or single-field variant cases; types
in `src/lib.rs`.

## Fallible methods

A fidius method returning `Result<T, PluginError>` maps to a WIT
`func(..) -> result<T, plugin-error>` where:

```wit
record plugin-error {
    code: string,
    message: string,
    details: option<string>,
}
```

The host maps `result::err(plugin-error)` → `CallError::Plugin(PluginError{..})`
— behaviour-identical to the cdylib `STATUS_PLUGIN_ERROR` path and the Python
exception path. A wasmtime **trap** (panic/unreachable/OOB) maps to
`CallError::Backend { runtime: "wasm", message }` (the variant added in
FIDIUS-T-0095).

## Interface-hash validation (integrity, not security)

cdylib bakes an `interface_hash` (FNV-1a over sorted method signatures) into its
descriptor; Python exports `__interface_hash__`. The WASM component does the
equivalent by exporting:

```wit
fidius-interface-hash: func() -> u64;
```

At load the host calls it and rejects a mismatch against the expected hash
(`LoadError`-level rejection, same guarantee as the other two backends). This is
an **integrity** check (catch wrong/incompatible interface), *not* a security
control — Ed25519 signing remains the security boundary and is artifact-agnostic
(`.wasm` is signed exactly like a cdylib/`.fid`).

## Reference WIT

`tests/wasm-fixtures/greeter/wit/world.wit` is the reference contract used by the
Phase-2 test components (T-0102 Rust guest, T-0105 non-Rust guest). It exercises
a typed method, a `#[wire(raw)]` method, a fallible method, and the hash carrier.
