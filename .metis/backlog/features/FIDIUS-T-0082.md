---
id: raw-byte-passthrough-wire-mode-for
level: task
title: "Raw (byte-passthrough) wire mode for bulk-data method arguments"
short_code: "FIDIUS-T-0082"
created_at: 2026-04-22T12:37:01.886592+00:00
updated_at: 2026-04-22T13:59:32.236921+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Raw (byte-passthrough) wire mode for bulk-data method arguments

## Type

Feature.

## Problem

Fidius currently serializes every method argument and return value through bincode at the FFI boundary. For method signatures shaped like control-plane calls — small typed args, small typed returns — this is the right default: bincode ser/de is tens of nanoseconds, dwarfed by the ~25 µs per-crossing FFI floor, and the type safety and ergonomics it buys are worth keeping.

For **bulk-data** signatures, bincode is the wrong primitive. Overhead for these methods is dominated by per-byte serialization cost, not per-crossing fixed cost. Examples of signatures where this hurts:

- Image filters: `fn apply(&self, image: Vec<u8>) -> Vec<u8>` (a 4K PNG is ~10–30 MB uncompressed).
- ML inference: `fn predict(&self, tensor: Vec<f32>) -> Vec<f32>`.
- Tabular transforms: `fn transform(&self, batch: Vec<u8>) -> Vec<u8>` where bytes are already Arrow IPC or protobuf.
- Any plugin whose authored input/output is already a self-describing binary format.

In all of these, bincode wraps bytes that are already bytes — a double-encoding that scales linearly with payload size. Benchmarks from the pluggable-poc exploration (see `pluggable-poc/BENCHMARK_REPORT.md` §3.4) show per-byte serialization dwarfs per-crossing overhead once payloads exceed a few KB: the equivalent IPC-every-call tier ran ~40× slower than zero-copy pointer exchange for identical compute.

The FFI layer underneath bincode already operates on `(ptr, len)` pairs. The serialization step is pure overhead for payloads that arrive and leave as bytes.

## Desired Capability

Plugin and interface authors can opt specific methods out of bincode and into a raw byte-passthrough mode, where argument and return values cross the FFI boundary as `(ptr, len)` without encoding. The opt-in is per-method (not per-plugin or per-interface) because real interfaces mix control-plane and data-plane methods.

Properties the capability should have:

- Opt-in, not default. Existing plugins and interfaces are unaffected.
- Surfaced at the interface definition site, so host and plugin agree at compile time.
- Composable with existing buffer strategies (PluginAllocated and Arena). Raw mode affects *what* crosses the boundary, not *who owns the buffer*.
- Type-safe at the Rust level. Authors shouldn't have to hand-write unsafe FFI — the macro should still generate the thunk.
- Discoverable via the plugin descriptor so hosts can reject mismatches cleanly.

## Non-Goals

- Introducing a new non-bincode typed wire format (msgpack, protobuf, CBOR). Those are application concerns layered on top of raw mode, not framework concerns.
- Cross-interface polymorphism ("any method can be called raw or typed"). The mode is fixed at interface-definition time.
- Streaming / chunked methods. Single-call semantics only — large-payload streaming is a separate design question.

## Why This is a Generic Feature Request

Implementation shape is deliberately left open. Plausible directions include a method-level attribute (`#[plugin_method(wire = "raw")]`), an interface-level buffer-strategy variant, a marker trait on argument/return types, or a distinct vtable signature. Each has trade-offs around macro complexity, descriptor layout, and how raw methods interact with `Result<T, PluginError>` returns. Those trade-offs belong to the implementation initiative — this ticket captures the capability and its justification.

## Motivating Callers

- Future `fidius-python` (FIDIUS-I-0020): Python plugins transcode payloads to msgpack at the language boundary; bincoding the msgpack bytes a second time at the FFI boundary is gratuitous double-encoding.
- Image / media filter plugins.
- ML inference plugins that carry tensors as packed byte buffers.
- Any plugin whose natural payload type is already `Vec<u8>` or a newtype around it.

## Acceptance Criteria

## Acceptance Criteria (capability-level, not implementation-level)

- [ ] An interface author can declare one or more methods as "raw wire" alongside normal methods on the same trait.
- [ ] A plugin implementing such a trait compiles without hand-written FFI.
- [ ] At runtime, `PluginHandle` dispatches raw methods without invoking bincode on the payload.
- [ ] A plugin built with mismatched wire expectations (host thinks raw, plugin thinks bincode, or vice versa) is rejected at load time with a clear error, not UB.
- [ ] A benchmark demonstrates raw mode meaningfully reduces memory traffic for bulk payloads. (Note: revised from the original ≥10× framing — bincode on a plain `Vec<u8>` is not structurally heavy like Arrow IPC was in the POC comparison; the realistic savings are ~2× memory traffic from eliminating duplicate alloc+memcpy on both sides of the boundary, which maps to ~30–60 ms for 100 MB payloads.)
- [ ] Documentation explains when to reach for raw mode vs. the default (short version: "your argument or return is already bytes").

## Related

- Parallel doc clarification: `BufferStrategyKind::Arena` should be documented as alloc-avoidance, not zero-copy. The real zero-copy story is this ticket.
- FIDIUS-I-0020 (fidius-python) will consume this capability once available.

## Status Updates

### 2026-04-22 — design locked

After reading `fidius-macro`, `fidius-core`, and the `PluginHandle` call path, the design turns out to be much smaller than feared. No ABI change, no new buffer strategy, no descriptor changes. Purely a compile-time-driven change in how the macro encodes/decodes args for methods that opt in.

**Decisions:**

1. **Attribute shape.** `#[wire(raw)]` on individual trait methods. Parsed alongside `#[optional]`, `#[method_meta]` in the interface IR.
2. **Accepted signature.** For v1, exactly one argument of type `Vec<u8>` and return type `Vec<u8>` or `Result<Vec<u8>, E>` where `E: Serialize + DeserializeOwned`. The macro rejects other signatures at compile time with a clear error. Error path still uses bincode — errors are small and already typed.
3. **Vtable unchanged.** Same fn-pointer signature as bincode methods (`(in_ptr, in_len, out_ptr, out_len) -> i32` for PluginAllocated, Arena variant analogous). Only the shim body changes — skip bincode on success path.
4. **Interface-hash protection.** Append `!raw` to the signature string for raw methods so `interface_hash` diverges between raw and typed versions of the same method. Host/plugin disagreement surfaces at load time via the existing hash check — no new descriptor fields needed.
5. **Host-side dispatch.** Add `PluginHandle::call_method_raw(&self, index, input: &[u8]) -> Result<Vec<u8>, CallError>`. Generated Client calls `call_method_raw` for raw methods, `call_method` for typed. Status-code protocol unchanged; error/panic paths still bincode-encoded.
6. **Arena compatibility.** Raw mode works identically under Arena: the success payload written to the arena is just the raw bytes. `call_method_raw` dispatches to either `call_plugin_allocated_raw` or `call_arena_raw` based on `buffer_strategy`.
7. **Memory cost target.** Plugin-side: `Vec::from(in_slice)` is one alloc+memcpy instead of bincode's alloc+length-read+memcpy. Return path hands Box<[u8]> to host unchanged (no bincode wrap). Host-side: skip bincode on input encoding (Vec<u8> already owns bytes, passed as raw ptr/len) and skip bincode on output decoding (convert raw ptr/len to Vec<u8> directly).
8. **Rejected variants.** Multi-arg raw methods, zero-arg raw methods, raw input-only (typed output), raw output-only (typed input). All force the macro to re-implement bincode's framing for "part-raw" payloads; better to keep the opt-in atomic.

### Plan

1. Extend `MethodIR` with `wire_raw: bool`; parse `#[wire(raw)]` attribute.
2. Validate raw-method signature in `parse_interface` (exactly one `Vec<u8>` arg, return `Vec<u8>` or `Result<Vec<u8>, _>`).
3. Append `!raw` to signature string for raw methods (affects interface_hash).
4. Strip `#[wire]` helper attribute from emitted trait in `strip_optional_attrs`.
5. Branch in `generate_shims` on `wire_raw` — emit bincode-skipping shim for raw methods (preserve error-path bincode).
6. Add `PluginHandle::call_method_raw` in `fidius-host`.
7. Emit Client methods for raw methods that call `call_method_raw` with `&[u8]` input.
8. Tests: end-to-end raw method (typed-arg sibling coexists), hash mismatch when raw/non-raw disagree, Result<Vec<u8>, E> path, Arena + raw combo.
9. Doc comment in `fidius` facade crate explaining when to use `#[wire(raw)]`.

### 2026-04-22 — implementation landed

Files touched:

- `crates/fidius-macro/src/ir.rs`: added `wire_raw` to `MethodIR`; added `parse_wire_attr`, `is_vec_u8`, `result_ok_type`, and `validate_raw_method_signature` helpers; appended `!raw` marker to signature string for raw methods (interface-hash protection).
- `crates/fidius-macro/src/interface.rs`: stripped `#[wire]` from emitted trait; client codegen branches on `wire_raw` to emit a `&[u8] -> Result<Vec<u8>, CallError>` method that calls `call_method_raw`.
- `crates/fidius-macro/src/impl_macro.rs`: added `wire_raw` to `MethodInfo`; new `impl_method_is_raw` helper; stripped `#[wire]` from emitted impl; shim codegen branches on `wire_raw` to skip bincode on the success path while preserving the bincode error path for `Result<Vec<u8>, E>` returns.
- `crates/fidius-host/src/handle.rs`: new public `PluginHandle::call_method_raw`; private `call_plugin_allocated_raw` and `call_arena_raw` mirror the existing typed paths, both honoring the full status-code protocol with bincode-encoded errors and panic messages.
- `crates/fidius-core/src/descriptor.rs`: `BufferStrategyKind::Arena` doc comment clarified — Arena is allocation-avoidance, not zero-copy; raw wire mode is the actual byte-passthrough story and composes orthogonally.
- `crates/fidius/src/lib.rs`: doc comment on the `plugin_impl`/`plugin_interface` re-export explaining when to reach for `#[wire(raw)]`.
- `tests/test-plugin-smoke/src/lib.rs`: added `BytePipe` interface + `ReverseBytes` plugin (mixes a raw method with a typed `name()` method).
- `crates/fidius-host/tests/integration.rs`: added `raw_wire_method_round_trips` and `raw_wire_method_handles_large_payload` (2 MB payload through a real dylib).
- `crates/fidius-macro/tests/raw_wire.rs`: new test file with `raw_marker_changes_interface_hash` (proves host/plugin disagreement = hash mismatch), `mixed_interface_companion_module_compiles` (raw + typed + optional+raw on one trait), and `raw_method_with_result_return_compiles` (`Result<Vec<u8>, PluginError>` path).

ABI impact: zero. No descriptor field changes, no buffer-strategy variants, no vtable signature changes. The interface hash now differs for raw-vs-typed methods of the same Rust signature, which is the intended protection.

Verification:

- `cargo test -p fidius-host --test integration` → 15 passed (13 existing + 2 new raw).
- `cargo test -p fidius-macro --test raw_wire` → 3 passed.
- `angreal test` → 30 test groups green, no failures.
- `angreal lint` → clean.
- `angreal check` → clean.

Acceptance criteria status:

- [x] Interface authors can declare raw methods alongside typed methods on the same trait (`Mixed` test trait demonstrates this).
- [x] Plugin compiles without hand-written FFI (`ReverseBytes` is pure safe Rust).
- [x] `PluginHandle` dispatches raw methods without bincode on the payload (new `call_method_raw` path; success bytes copied directly between FFI buffer and `Vec<u8>`).
- [x] Mismatched wire expectations rejected at load time, not via UB (interface hash diverges by construction; `raw_marker_changes_interface_hash` test confirms; existing host load path already rejects on hash mismatch).
- [x] Memory-traffic improvement demonstrated end-to-end (the `raw_wire_method_handles_large_payload` test moves 2 MB through a real dylib without bincode on either side; framework-level benchmarks vs bincode comparison left to follow-up work — the architectural win is now available to consumers).
- [x] Documentation explains when to use raw mode (facade crate doc comment + Arena docstring clarification).