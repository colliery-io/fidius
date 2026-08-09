<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# Host Functions (plugin → host callbacks)

`#[plugin_interface]` gives you one direction: the host calls indexed methods
on a plugin. **Host functions** add the reverse channel: the host declares a
set of functions it offers, and plugin code calls them **mid-execution** —
while a plugin method is still live on the stack.

The motivating shape is a workflow engine's `defer_until`: a task running
inside a plugin must ask the host to *release its concurrency slot*, wait on
an external condition, then *reclaim* the slot — host-owned state (a
database row, a semaphore) that only the host can touch, interleaved with
user code that only the plugin can run. Data the host can compute up front
can be pushed in the request; deferral genuinely needs a call in the other
direction.

## The model

One trait, both ends generated:

```rust
// Interface crate — shared by host and plugins:
use fidius::{host_interface, PluginError};

#[host_interface(version = 1)]
pub trait CloacinaHost: Send + Sync {
    fn release_slot(&self, task_execution_id: String) -> Result<(), PluginError>;
    fn reclaim_slot(&self, task_execution_id: String) -> Result<(), PluginError>;
    fn set_sub_status(&self, task_execution_id: String, status: String) -> Result<(), PluginError>;
}
```

**Plugin side** — a generated typed client:

```rust
let host = CloacinaHostClient::bound()?;   // Err(HostCallError::NotBound) if never bound
host.release_slot(&task_id)?;              // blocks until the host function returns
```

**Host side** — a generated binding, installed after loading the library:

```rust
let lib = fidius_host::loader::load_library(path)?;
let bound = CloacinaHostBinding::bind(&lib, Arc::new(MyHost { .. }))?;
// bound == false → the plugin doesn't import CloacinaHost (fine);
// Err(HostInterface{Version,Hash}Mismatch) → built against a different revision.
```

For in-process plugins (tests, examples): `CloacinaHostBinding::bind_in_process(host)`.

## Mechanism

Because plugins are `dlopen`'d **in-process**, the channel is a C-ABI
function-pointer table, not RPC. `bind` hands the plugin a
`#[repr(C)] HostFunctionTable` containing one indexed `dispatch` entry point
plus identity fields. Calls are bincode on the same wire conventions as the
forward direction: arguments as a tuple, returns as the bare value, errors
as `PluginError`. Output buffers are host-allocated and host-freed (the
plugin copies them out and calls the table's `free_buffer`), so memory
never crosses allocator boundaries.

Plugins advertise which host interfaces they consume through an *optional*
`fidius_get_host_imports` export (emitted by `fidius_plugin_registry!()`).
Each import carries the interface name, declared version, and an FNV-1a
hash of the method signatures the plugin was **built against**.

## Versioning: fail at load, never mis-dispatch

Bincode is positional — dispatching against a drifted signature set would
silently corrupt arguments. So the surface is gated **twice** before a call
is possible:

1. **Host-side gate** (`bind` / `bind_host_interface`): the plugin's
   declared version must equal the host's, and the signature hashes must
   match. A mismatch returns `LoadError::HostInterfaceVersionMismatch` or
   `LoadError::HostInterfaceHashMismatch` and installs nothing.
2. **Plugin-side re-validation**: the generated bind shim re-checks ABI
   version, hash, version, and function count before storing the table —
   a hand-rolled host that skips the gate still can't install a
   mismatched table.

If no table was installed (mismatch, or the host simply doesn't provide the
interface), plugin calls return the typed `HostCallError::NotBound` — a
recoverable error, never a crash and never a mis-dispatch. Binding is
once-only per loaded library: a second bind fails with
`BIND_ERR_ALREADY_BOUND` rather than swapping the table under in-flight
calls.

## The threading / async / reentrancy contract

This is the sharp edge of any callback channel; the contract is explicit.

**Host functions are synchronous at the FFI boundary.** There is no async
variant in v1. The calling plugin thread blocks until the host function
returns.

**A host implementation must:**

- be `Send + Sync` (the macro requires the supertraits) and tolerate
  concurrent calls from multiple plugin threads;
- tolerate being called on threads it does not own: the plugin's method
  thread (which is the host's own calling thread during the reentrant
  host → plugin → host path), or any thread/runtime the plugin spawned —
  cloacina's plugin shell calls from inside its own tokio `block_on`;
- bridge to async internally when needed, e.g.
  `tokio::runtime::Handle::block_on(...)` on a handle to the host's
  runtime. This is safe precisely because the calling thread is never one
  of the host runtime's workers — which the next rule guarantees;
- **never be needed while the host holds a lock (or a runtime worker
  thread) across a plugin call.** The callback re-enters the host while
  the plugin call is still on the stack; if the host called the plugin
  with lock `L` held and a host function tries to take `L`, that is a
  self-deadlock. Call plugins from dedicated threads or
  `spawn_blocking`-style pools, and treat "what may a host function need?"
  as part of the lock discipline around every plugin call. Host functions
  that block (like `reclaim_slot` waiting for capacity) magnify the cost
  of getting this wrong: the deadlock appears only under load.

**Plugin code may**, while one of its methods executes: call host functions
from the method's own thread or from threads it spawned; make several calls
concurrently; block in a host function that legitimately waits.
**Plugin code must not** cache the client beyond the plugin's lifetime, and
must treat every call as fallible (`HostCallError` covers unbound, host
error, host panic, and serialization faults).

**Detecting violations:** `fidius::host_ffi::host_callback_depth()` is a
per-thread counter maintained by the dispatch shims — `0` outside
callbacks, `1` inside a host function, `>1` if a host function re-entered
plugin code which called back again. Host implementations can
`debug_assert!` invariants against it (e.g. assert a given lock is only
taken at depth 0). Fidius cannot see *which* lock a host holds across a
plugin call, so the lock rule itself remains a contract, not a runtime
check.

## Error propagation

Fallible host functions must return `Result<T, fidius::PluginError>` —
enforced at macro time. `PluginError { code, message, details }` is the
typed error currency in both directions; domain errors ride in
`code`/`details`. On the plugin side every failure mode is one enum:

| Variant | Meaning |
| --- | --- |
| `HostCallError::Host(PluginError)` | the host function returned its typed error |
| `HostCallError::NotBound` | the host never bound this interface |
| `HostCallError::HostPanic(msg)` | the host function panicked (caught at the boundary) |
| `HostCallError::Serialization` / `Deserialization` | wire fault |
| `HostCallError::VersionMismatch` / `HashMismatch` | wasm per-call gate: the host provides a different revision |

Panics never unwind across the FFI boundary in either direction: the
dispatch shim catches host-side panics (→ `HostPanic`), and the existing
plugin-shim guard catches plugin-side panics (→ `CallError::Panic` for the
host).

## Backward compatibility

Plugins that declare no host interface are untouched: the imports registry
export is optional (hosts treat its absence as "no imports"), an empty
registry binds nothing, and no existing symbol, descriptor field, or init
path changed. The channel ships in a **patch** release (0.5.7) precisely
so `ABI_VERSION` stays at 500 and every deployed 0.5.x plugin keeps
loading.

## WASM

WASM plugins get the same channel through the **`fidius:host-call`
component import** instead of a function-pointer table (the sandbox shares
no memory, so pointers can't cross). The generated `<Trait>Client` has the
same surface on both runtimes; only the transport differs:

- The guest dispatches
  `fidius:host-call/host.call(interface-name, expected-version,
  expected-hash, index, args) -> (status, payload)` — the identity triple
  travels with **every call**.
- The host links the import into every component (harmless for plugins
  that don't use it — same pattern as `fidius:stream-pull`), backed by a
  per-executor table registry. Binding:

  ```rust
  let handle = host.load_wasm("my-plugin", &DESCRIPTOR)?;
  MyHostBinding::bind_wasm(&handle, Arc::new(MyHost { .. }))?;
  // requires the interface crate's `host` + `wasm` features
  ```

  The same `HostFunctionTable` built for the dylib path backs the wasm
  dispatch, so a host implementation is written once. Binds are once-only
  per handle, and each loaded component has its own registry (no
  per-library global, unlike the dylib cell).
- **The gate is per-call**, not at instantiation: a component can't be
  introspected for the host interfaces its method bodies use, so the host
  compares the guest's expected version + hash against the bound table on
  every dispatch (a `u64` compare) and returns a typed
  `HostCallError::VersionMismatch` / `HashMismatch` /
  `NotBound` on any skew. Same guarantee as the dylib bind-time gate —
  loud, typed, and **never** a mis-dispatch of positional bincode — the
  failure just surfaces at the first call instead of at load. `bound()` /
  `is_bound()` run the gate eagerly via a reserved probe index.

The threading contract is unchanged; note that wasm host functions run on
the thread driving the component call (components are single-threaded), so
the reentrant host → plugin → host path is always same-stack.

## Worked example

`examples/examples/08_host_functions.rs` is a runnable in-process demo
(release slot → observe host state → reclaim slot). The full dylib path —
including the reentrant round-trip, version/hash mismatch at load, panics
in both directions, and the unbound case — is exercised in
`crates/fidius-host/tests/host_functions_e2e.rs`,
`host_functions_unbound.rs`, and `host_functions_in_process.rs`, driven by
the `tests/test-plugin-hostcall` fixture. The wasm variant — the same
matrix over a real component, including the per-call version/hash gate —
is `wasm_host_functions_e2e.rs`, driven by `tests/wasm-fixtures/hostcall`.
