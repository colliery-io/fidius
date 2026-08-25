<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# Changelog

## 0.5.8 — hostname-carrying TCP egress: resolve-and-pin (FIDIUS-I-0034)

### Added

- **`EgressPolicy::authorize_tcp_target(&TcpTarget)`** — name-aware TCP
  authorization. `TcpTarget` carries both the **hostname the guest actually
  dialed** and the resolved peer, so an embedder can allow-list databases by
  *name* (`allowed_hosts: ["db.example.internal"]`) instead of by IPs that
  rotate under managed endpoints (RDS / Cloud SQL / Azure). The default
  implementation delegates to `authorize_tcp(&target.addr)`, so every
  existing policy keeps byte-identical behavior with zero changes.
- **Resolve-and-pin, owned by fidius.** The hostname is recovered via a
  fidius-owned shadow of `wasi:sockets/ip-name-lookup`
  (`executor/name_lookup.rs`): guest lookups resolve host-side and pin
  `name ↔ IPs` per store; `socket_addr_check` hands the policy
  `TcpTarget { host: Some(name), addr }` for hostname dials and
  `host: None` for IP-literal dials (no lookup → no pin — a name-keyed
  policy denies those, the honest default). Names are pinned
  ASCII-lowercase, IPs canonicalized (v4-mapped-v6 can't dodge a pin);
  re-resolution replaces a name's pins wholesale, so a rotated-away IP
  loses the name's authority immediately. The shadow installs only under
  the existing two-key gate (`tcp`/`udp` grant AND an embedder policy) —
  without it, upstream's lookup stands untouched.
- **`EgressPolicy::authorize_dns(&str)`** — a hook on the lookup itself,
  consulted before resolution. Defaults to **allow** (lookup was already
  open whenever the tier is granted; connects stay gated regardless);
  overriding it stops a granted guest from probing arbitrary DNS — a
  denial fails the lookup exactly like an unresolvable name
  (`permanent-resolver-failure`), resolving and pinning nothing.
- `TcpTarget` exported alongside `EgressPolicy`/`EgressDenied` from
  `fidius_host` and the `fidius` facade. E2E coverage in
  `crates/fidius-host/tests/hostname_egress_e2e.rs`: name-keyed
  allow-list, same-IP/two-names pin correctness (no IP fallthrough),
  rotation (stale pin loses authority) — including on a **configured
  resident instance**, where pins persist and rotate across separate
  calls on the persistent store — pin attribution for literal dials to a
  pinned IP, case-insensitive matching, multi-address resolution with
  connect fallback, the `PluginHost::builder()` path, and both
  `authorize_dns` polarities.

### Changed

- `authorize_tcp` docs no longer punt resolve-and-pin to the embedder —
  fidius ships the mechanism; the docs (`docs/explanation/wasm-capabilities.md`)
  now show the hostname allow-list pattern. The pin narrows lookup→connect
  TOCTOU to "an address this instance was actually given for that name";
  connects to a pinned IP (even dialed as a literal) carry that name's
  authority for the store's lifetime or until re-resolution replaces it.
- `wasmtime-wasi-io` (same major as wasmtime-wasi) is now a dependency of
  the `wasm` feature — one more crate in the lockstep wasmtime pin
  (relevant to the pin-bump automation, FIDIUS-T-0159).

### Compatibility

Released as a **patch**: `ABI_VERSION` unchanged, no guest-facing change of
any kind (`std::net` dialing untouched, no WIT bump), and an embedder that
overrides only the old `authorize_tcp` observes byte-identical behavior
(proven by test). UDP semantics unchanged.

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

**WASM plugins are served too**, through the `fidius:host-call` component
import (same generated `<Trait>Client` surface; hosts bind with
`<Trait>Binding::bind_wasm(&handle, host)`, reusing the same
`HostFunctionTable`). Because a component's host-interface usage can't be
introspected at instantiation, the wasm gate is enforced **on every
call**: the guest sends the identity triple (name, version, hash) with
each dispatch and a skew returns the typed
`HostCallError::VersionMismatch` / `HashMismatch` / `NotBound` — same
never-mis-dispatch guarantee, surfacing at the first call instead of at
load. The import is always linked and harmless for components that don't
use it.

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
