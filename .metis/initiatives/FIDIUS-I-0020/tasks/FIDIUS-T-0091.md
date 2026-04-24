---
id: fidius-python-stub-cli-subcommand
level: task
title: "fidius python-stub CLI subcommand"
short_code: "FIDIUS-T-0091"
created_at: 2026-04-24T00:10:03.552460+00:00
updated_at: 2026-04-24T17:49:31.391177+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# fidius python-stub CLI subcommand

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

`fidius python-stub --interface=<crate-or-path> --out=<file.py>` reads an interface crate's source and emits a Python stub: type-hinted method signatures + the `__interface_hash__` constant the plugin author needs to declare. Closes the contract loop between Rust-defined interfaces and Python plugin authors.

## Scope

- New `fidius-cli` subcommand `python-stub`.
- Source resolution: parse the interface crate's source via `syn` (path to `lib.rs` or directory). Loading a compiled artifact is harder and unnecessary — interface traits are simple syntactic structures.
- Output: a `.py` file with:
  - `__interface_hash__ = 0x...` constant matching what the Rust macro would compute.
  - One stub function per trait method, with type hints from a Rust → Python primitive mapping table (`i64` → `int`, `String` → `str`, `Vec<u8>` → `bytes`, `bool` → `bool`, `f64` → `float`, etc.). Each stub uses the `@fidius.method` decorator from the SDK.
  - For unsupported types in the mapping, emit `# TODO: unsupported Rust type <X>` and a generic `Any` hint.
  - Module docstring referencing the interface and crate version.
- For methods marked `#[wire(raw)]`, the stub uses `bytes`-typed args and return.
- Test: generate a stub for the `BytePipe` interface (from T-0082) and assert: imports cleanly, has the right hash, has both `reverse(data: bytes) -> bytes` and `name() -> str`.

## Acceptance Criteria

## Acceptance Criteria

- [x] Generated stub's `__interface_hash__` matches the runtime constant — guaranteed by both sides going through `fidius_core::hash::signature_string` + `interface_hash`. Verified by `rendered_stub_hash_matches_macro` test.
- [x] Primitive Rust types (`i*`, `u*`, `f*`, `bool`, `String`, `&str`, `()`) map to Python equivalents (`int`, `float`, `bool`, `str`, `None`).
- [x] `Vec<T>` → `list[T]`, `Vec<u8>` → `bytes`, `Option<T>` → `Optional[T]`, `Result<T, _>` → `T` (errors are raised, not returned).
- [x] `#[wire(raw)]` methods force `bytes` for both args and return regardless of underlying signature.
- [x] Unsupported types fall back to `Any  # TODO: unsupported Rust type \`X\`` and the stub emits a header comment pointing them out.
- [x] Live smoke test: `fidius python-stub --interface tests/test-plugin-smoke/src/lib.rs --out - --trait-name BytePipe` produced the expected stub with hash `0xdf233d1a5936eb5c`.

## Dependencies

- T-0086 (Python SDK — the stub `import fidius` for the decorator).

## Implementation Notes

- Reuse `fidius-macro::ir::parse_interface` for reading the trait IR. Don't reimplement the parser. The CLI may need to expose a thin façade or the IR parser may need to move to a shared crate — pick whichever is less invasive.
- The hash function (`fidius_core::hash::interface_hash`) is already shared. Generated stubs call into the same canonical signature-string builder so the hash is guaranteed equal.
- Python stubs are advisory at the type-hint level; the actual interface-hash check happens at load time. Authors who don't use the stub still get hash mismatches caught — the stub just makes the right answer easy to type.

## Status Updates

### 2026-04-24 — landed

- **Single source of truth for canonical sig strings**: added `fidius_core::hash::signature_string(name, arg_types, ret, wire_raw)`. The proc macro's `build_signature_string` and the new CLI stub generator both call it. Drift between them is no longer possible. Existing macro tests still pass.
- **New CLI subcommand** `fidius python-stub --interface <src.rs> --out <file.py | -> [--trait-name <T>]`. Lives in `crates/fidius-cli/src/python_stub.rs` (~330 LOC including tests).
- Parses the interface source via `syn`, finds traits with `#[plugin_interface]`, picks one (or the named one when multiple are present), extracts methods, maps Rust types to Python type hints, and emits a stub with the matching `__interface_hash__`.
- Type mapping table: ints/floats/bool/String/&str/() → Python primitives; `Vec<u8>` → `bytes` (special-cased); `Vec<T>` → `list[T]`; `Option<T>` → `Optional[T]`; `Result<T, _>` → `T`; unknown → `Any  # TODO`.
- `#[wire(raw)]` methods get bytes-in-bytes-out regardless of underlying Rust signature, plus a comment explaining the wire mode.
- 6 inline unit tests in `python_stub.rs` covering: primitives, `Vec<u8>` mapping, raw-wire behaviour, unknown-type TODO marker, hash equivalence with the macro, multi-trait selection (named, ambiguous error, not-found error).
- Live smoke run against `tests/test-plugin-smoke/src/lib.rs` for the `BytePipe` trait produced a clean stub with `__interface_hash__ = 0xdf233d1a5936eb5c` — matching what the runtime computes.
- `angreal lint`, `angreal check`, `angreal test` all clean (33 test groups green, 6 new python_stub tests included).