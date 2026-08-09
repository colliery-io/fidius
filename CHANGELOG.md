<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# Changelog

## 0.5.7 — host functions: a plugin → host callback channel

### Added

- **`#[fidius::host_interface(version = N)]`** — declare a set of host
  functions from a trait, the reverse direction of `#[plugin_interface]`
  (ADR-0011, `docs/explanation/host-functions.md`). From one trait fidius
  generates:
  - a plugin-side typed client (`<Trait>Client::bound()` +
    per-method calls, bincode args/returns on the existing wire), and
  - a host-side `<Trait>Binding` that wraps an `Arc<dyn Trait>` in a
    C-ABI function table and installs it into a loaded plugin
    (`bind(&LoadedLibrary, host)`, `bind_plugin`, `bind_in_process`).
- `fidius::host_ffi` — the `#[repr(C)]` contract types
  (`HostFunctionTable`, `HostImportDescriptor`, `HostImportRegistry`),
  the plugin-side error surface (`HostCallError`), bind statuses, and the
  per-thread `host_callback_depth()` reentrancy probe.
- `fidius_host::host_import` — host-import discovery
  (`LoadedLibrary::host_imports()` / `LoadedPlugin::host_imports()`) and
  the generic bind gate `bind_host_interface`.
- `fidius_plugin_registry!()` now additionally emits the **optional**
  `fidius_get_host_imports` export advertising the host interfaces a
  plugin can consume. Plugins with none export an empty registry; hosts
  treat a missing symbol (pre-0.5.7 plugins) identically.
- Worked example `examples/08_host_functions.rs` and end-to-end tests
  (`crates/fidius-host/tests/host_functions_*.rs`) covering the reentrant
  host → plugin → host path, both panic directions, the unbound case, and
  the load-time gates.

### Version-gate behavior (important)

The host-function surface is versioned independently of the plugin
interface, on two axes: the declared `version = N` and an FNV-1a hash of
the method signatures. Both are checked **at bind time (plugin load)** on
the host side (`LoadError::HostInterfaceVersionMismatch` /
`HostInterfaceHashMismatch`) *and* re-validated defensively by the
plugin-side bind shim. A plugin built against host-interface v2 loading
into a v1 host fails loudly at load; a mismatched or unbound table can
never be dispatched through — plugin-side calls surface the typed
`HostCallError::NotBound` instead. Bincode is positional, so this gate is
what makes signature drift a load error rather than silent argument
corruption.

Threading contract (short form; full text in the macro rustdoc and
`docs/explanation/host-functions.md`): host functions are synchronous at
the boundary and may arrive on any plugin thread — including the host's
own thread while a host → plugin call is live on that stack. Hosts must
not hold locks (or async-runtime worker threads) across a plugin call
that a host function could need.

v1 of the channel is **dylib-only**; the API is shaped so a WASM
import-based variant can be added later without breaking changes.

### Fixed

- cdylib streaming: the pump thread now holds the plugin's `Arc<Library>`
  for its lifetime, closing a race where dropping the last plugin handle
  could `dlclose` the dylib while the pump thread was still executing the
  guest's `next`/`drop_fn` (segfault under unlucky timing).

### Compatibility

Released as a **patch** on purpose: `ABI_VERSION` stays at 500, so every
existing 0.5.x plugin loads unchanged. Plugins that declare no host
interface are completely unaffected — no new required symbols, no init
changes. (A 0.6.0 bump would have force-rejected all deployed 0.5.x
plugins via the pre-1.0 ABI gate — see ADR-0002/ADR-0011.)

## 0.5.6 and earlier

See the git history and `.metis/` ADRs/initiatives.
