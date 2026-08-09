---
id: 011-host-functions-fn-pointer-table
level: adr
title: "Host functions via an in-process function-pointer table, released as a patch"
number: 11
short_code: "FIDIUS-A-0011"
created_at: 2026-08-09T00:00:00.000000+00:00
updated_at: 2026-08-09T00:00:00.000000+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-11: Host functions via an in-process function-pointer table, released as a patch

## Context **[REQUIRED]**

Every fidius channel so far points one way: the host calls indexed methods on
the plugin. Nothing lets **plugin code call back into the host
mid-execution**. The concrete driver is cloacina's `defer_until`: a task
running inside a plugin must ask the host to release its concurrency slot
(host-owned semaphore + DB state), poll a user condition (plugin code), then
reclaim the slot — host state interleaved with plugin execution. The
inversion workaround (plugin returns "deferred", host re-invokes) was
rejected downstream because re-running the task from the top repeats side
effects undetectably.

Design forces:

- Plugins are `dlopen`'d **in-process** — cross-process transports are a
  non-goal.
- Bincode is positional: any version skew that reaches dispatch corrupts
  arguments silently. Mismatches must die at load.
- Panics must not unwind across FFI in either direction.
- The motivating host operation (`reclaim_slot`) is genuinely async and may
  block; the plugin call site sits inside the plugin's own tokio
  `block_on`.
- Deployed 0.5.x plugins must keep loading — and pre-1.0, fidius's
  `ABI_VERSION` gate (ADR-0002) rejects any cross-**minor** load.

## Decision **[REQUIRED]**

1. **Mechanism: a C-ABI function-pointer table, not RPC.** A new
   `#[fidius::host_interface(version = N)]` macro generates, from one
   trait: a `#[repr(C)] HostFunctionTable` (single indexed `dispatch`
   entry + identity fields), a host-side `<Trait>Binding` that wraps an
   `Arc<dyn Trait>` and installs the table, and a plugin-side
   `<Trait>Client` that bincode-encodes args (same tuple wire as the
   forward direction) and dispatches through the bound table. Output
   buffers are host-allocated/host-freed (allocator symmetry with
   `PluginAllocated`, mirrored).

2. **Discovery + double gate.** Plugins advertise consumable host
   interfaces via an *optional* `fidius_get_host_imports` export
   (inventory-collected, emitted by `fidius_plugin_registry!()`), each
   import carrying (name, declared version, FNV-1a signature hash). The
   host-side bind refuses on version or hash mismatch (typed
   `LoadError`s); the plugin-side bind shim re-validates
   ABI/version/hash/fn-count before storing. An unbound interface
   surfaces as the typed `HostCallError::NotBound`. Binding is once-only
   per library (no table swaps under in-flight calls).

3. **Sync-only host functions; explicit threading contract.** Host
   functions block the calling plugin thread; hosts bridge to async
   internally (`Handle::block_on`). Contract (documented in the macro
   rustdoc + `docs/explanation/host-functions.md`): host impls are
   `Send + Sync`, callable from any plugin thread including the reentrant
   same-stack path (host → plugin → host); the host must not hold locks or
   runtime workers across a plugin call that a host function could need.
   `host_ffi::host_callback_depth()` (per-thread) makes reentrancy
   observable for debug assertions.

4. **Errors: `PluginError` is the currency both ways.** Fallible host
   functions must return `Result<T, PluginError>` (macro-enforced);
   panics are caught at the boundary on both sides
   (`HostCallError::HostPanic` / `CallError::Panic`).

5. **Released as 0.5.7 (patch), not 0.6.0.** The channel is purely
   additive at the ABI level (a new optional export; no existing struct
   changed). A minor bump would flip `ABI_VERSION` 500 → 600 and
   force-reject every deployed 0.5.x plugin — exactly the compatibility
   break the feature is required to avoid.

6. **v1 is dylib-only.** For WASM, host functions are component imports —
   a different mechanism. The identity triple + bincode request/response
   are transport-independent, so a future `fidius:host-call` import can be
   added without breaking this API; wasm builds compile the table
   machinery out today.

## Consequences

- One `Arc` + one table are intentionally leaked per bind (process
  lifetime), so no call can ever observe a dangling table.
- The generated `<Trait>Binding::bind` is gated on the interface crate's
  `host` feature (same convention as the generated `{Trait}Client`);
  `bind_in_process` is available ungated for tests/embedding.
- Streaming, async signatures, `#[optional]`, and `#[wire(raw)]` are
  rejected on host interfaces in v1 — request/response only.
- Fixing the pre-existing pump-thread/`dlclose` race surfaced by the new
  export (the streaming pump thread now holds the `Arc<Library>` it
  executes code from) rode along in the same release.
