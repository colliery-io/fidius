---
id: multi-argument-methods-auto-tuple
level: initiative
title: "Multi-Argument Methods — Auto-Tuple Packing at FFI Boundary"
short_code: "FIDIUS-I-0010"
created_at: 2026-04-01T01:30:44.811063+00:00
updated_at: 2026-04-17T13:17:21.185476+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: multi-argument-methods-auto-tuple
---

# Multi-Argument Methods — Auto-Tuple Packing at FFI Boundary Initiative

## Context

Plugin interface traits currently require every method to have exactly one input parameter. This is because the FFI shim in `fidius-macro/src/impl_macro.rs` deserializes the input buffer into a single value and passes it as a single argument:

```rust
let args = fidius::wire::deserialize(in_slice)?;
let output = instance.method_name(args);
```

This forces plugin authors to either wrap multiple values in a struct or pass serialized compound types manually — an unnecessary ergonomic tax. Natural Rust trait signatures like `fn add(&self, a: f64, b: f64) -> f64` should just work.

The IR layer (`ir.rs`) already parses multiple arguments correctly (`arg_types: Vec<Type>`, `arg_names: Vec<Ident>`), but the codegen layer collapses them into a single deserialization.

## Goals & Non-Goals

**Goals:**
- Plugin interface traits support methods with 0, 1, or N arguments
- The macro auto-packs arguments into a tuple for serialization and unpacks on the other side
- Plugin authors write natural Rust signatures — no manual wrapping
- Host-side `call_method` continues to work (caller packs args as a tuple)
- Breaking change: all methods use tuple encoding (pre-1.0, acceptable)

**Non-Goals:**
- Changing the FFI ABI (still one input buffer, one output buffer per call)
- Supporting `&self` as non-first arg or other exotic patterns
- Named parameters or keyword arguments

## Detailed Design

### Approach: Auto-tuple at the macro level

The FFI boundary stays the same — one `(in_ptr, in_len)` input buffer per method call. The macro generates the packing/unpacking code.

#### Plugin side (`plugin_impl` shim generation)

For a method `fn foo(&self, a: u32, b: String) -> bool`:

**Today:**
```rust
let args = deserialize(in_slice)?;          // expects single value
let output = instance.foo(args);            // passes one arg
```

**After:**
```rust
let (a, b) = deserialize::<(u32, String)>(in_slice)?;  // deserialize as tuple
let output = instance.foo(a, b);                         // pass individually
```

**Zero args** (`fn status(&self) -> String`):
```rust
// skip deserialization entirely (or deserialize `()`)
let output = instance.status();
```

**One arg** (`fn process(&self, input: String) -> String`):
```rust
let (input,) = deserialize::<(String,)>(in_slice)?;   // 1-tuple
let output = instance.process(input);
```

All arg counts use tuple encoding uniformly — no special cases. This is a breaking wire format change but we're pre-1.0 so that's fine.

#### Host side (`plugin_interface` proxy generation)

The `plugin_interface` macro already generates a typed host proxy (R-25). The proxy's generated methods need to serialize args as a tuple:

```rust
// Generated proxy method
fn foo(&self, a: u32, b: String) -> Result<bool, CallError> {
    self.handle.call_method(0, &(a, b))
}
```

All arg counts use tuple encoding uniformly — the proxy always packs as a tuple.

#### Interface hash impact

The signature string format is `"name:arg_type_1,arg_type_2->return_type"`. This already handles multiple args. The hash won't change for existing single-arg methods since the format is the same.

**However**, the wire encoding changes for multi-arg: `serialize(&input)` vs `serialize(&(a, b))`. This means:
All arg counts use uniform tuple encoding:
- Zero args: `()` (unit)
- One arg: `(T,)` (1-tuple)
- N args: `(A, B, C)` (N-tuple)

This is a breaking wire format change. Pre-1.0, so acceptable.

### Files to modify

- `fidius-macro/src/impl_macro.rs` — shim codegen: branch on arg count
- `fidius-macro/src/interface.rs` — proxy codegen: pack args as tuple in generated proxy methods
- `fidius-macro/tests/` — new tests for multi-arg and zero-arg methods
- `fidius-cli/src/commands.rs` — update scaffold templates to show multi-arg example
- `tests/test-plugin-smoke/` — add multi-arg method to smoke test

## Alternatives Considered

- **Require struct wrappers**: The current state. Rejected because it's an unnecessary ergonomic burden — users shouldn't have to create `FooArgs { a: u32, b: String }` for every method.
- **Auto-generate arg structs**: The macro could generate a named struct per method. Rejected because tuples are simpler, don't pollute the namespace, and serde handles them natively.
- **Change the FFI ABI to support multiple buffers**: Rejected — adds complexity to the vtable layout and descriptor for no real benefit. One buffer with a packed tuple is equivalent.

## Implementation Plan

1. Update `impl_macro.rs` shim generation to branch on arg count (0, 1, N)
2. Update `interface.rs` proxy generation to pack args as tuples for multi-arg
3. Add compile tests for multi-arg and zero-arg methods
4. Update smoke test and E2E test with multi-arg examples
5. Update scaffold templates to show multi-arg pattern
6. Update docs (tutorial, ABI spec) to reflect multi-arg support