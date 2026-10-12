---
title: "API reference"
description: "The API reference, written by plissken from the source."
editUrl: false
sidebar:
  order: 0
  hidden: true
---

## Rust

- [`fidius`](rust/fidius/): Fidius — a Rust plugin framework for trait-to-dylib plugin systems.
- [`fidius-cli::commands`](rust/fidius-cli/commands/)
- [`fidius-cli::python_stub`](rust/fidius-cli/python_stub/): Python stub generator: turn a `#[plugin_interface]` trait into a `.py` file the plugin author imports. Gives them type-hinted signatures and the `__interface_hash__` constant the host expects at load time.
- [`fidius-core`](rust/fidius-core/)
- [`fidius-core::async_runtime`](rust/fidius-core/async_runtime/): Per-dylib async runtime for plugin methods.
- [`fidius-core::host_registry`](rust/fidius-core/host_registry/): Host-import registry assembly (plugin → host callback channel).
- [`fidius-core::package`](rust/fidius-core/package/): Source package manifest types and parsing.
- [`fidius-core::registry`](rust/fidius-core/registry/): Plugin registry assembly for multi-plugin dylibs.
- [`fidius-guest`](rust/fidius-guest/): `fidius-guest` — the wasm-buildable subset of the Fidius shared types.
- [`fidius-guest::client_stream`](rust/fidius-guest/client_stream/): Guest-side consumer for **client-streaming over WASM** (FIDIUS-I-0030 CS2.3).
- [`fidius-guest::descriptor`](rust/fidius-guest/descriptor/): FFI descriptor and registry types for the Fidius plugin framework.
- [`fidius-guest::error`](rust/fidius-guest/error/): Error types for the Fidius plugin framework.
- [`fidius-guest::frame`](rust/fidius-guest/frame/): Streaming frame format for the server-streaming plugin boundary (FIDIUS-I-0026, design decision D2).
- [`fidius-guest::hash`](rust/fidius-guest/hash/): FNV-1a interface hashing for compile-time ABI drift detection.
- [`fidius-guest::host_call`](rust/fidius-guest/host_call/): Guest-side consumer for **host functions over WASM** (the plugin → host callback channel, wasm variant).
- [`fidius-guest::host_ffi`](rust/fidius-guest/host_ffi/): Host-function (plugin → host callback) FFI types — the stable C ABI contract for the reverse direction (FIDIUS host-callback channel).
- [`fidius-guest::http`](rust/fidius-guest/http/): `fidius::http` — the brokered outbound HTTP client for sandboxed WASM connectors (FIDIUS-I-0028).
- [`fidius-guest::python_descriptor`](rust/fidius-guest/python_descriptor/): Compile-time descriptor of a fidius interface used by the Python loader.
- [`fidius-guest::sockets`](rust/fidius-guest/sockets/): `fidius::sockets` — capability-gated outbound **TCP** for sandboxed WASM connectors (FIDIUS-I-0033).
- [`fidius-guest::status`](rust/fidius-guest/status/): FFI status codes returned by plugin method shims.
- [`fidius-guest::stream_ffi`](rust/fidius-guest/stream_ffi/): cdylib server-streaming FFI handle (FIDIUS-I-0026 Phase 3 / CS.1, T-0138).
- [`fidius-guest::stream_marker`](rust/fidius-guest/stream_marker/): The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).
- [`fidius-guest::value`](rust/fidius-guest/value/): `Value` — a neutral, self-describing value tree for the plugin boundary.
- [`fidius-guest::wasm_descriptor`](rust/fidius-guest/wasm_descriptor/): Compile-time descriptor of a fidius interface used by the WASM loader.
- [`fidius-guest::wire`](rust/fidius-guest/wire/): Wire format serialization for Fidius plugin FFI boundary.
- [`fidius-host`](rust/fidius-host/)
- [`fidius-host::arch`](rust/fidius-host/arch/): Architecture detection for dylib files.
- [`fidius-host::arena`](rust/fidius-host/arena/): Thread-local arena pool for Arena-strategy plugin invocations.
- [`fidius-host::client_stream`](rust/fidius-host/client_stream/): Host-side **producer** for client-streaming (FIDIUS-I-0030 / ADR-0007).
- [`fidius-host::error`](rust/fidius-host/error/): Error types for fidius-host plugin loading and calling.
- [`fidius-host::executor`](rust/fidius-host/executor/): `PluginExecutor` — the dispatch seam across execution backends.
- [`fidius-host::executor::body_tee`](rust/fidius-host/executor/body_tee/): Tee/replay bodies for the egress response hook (FIDIUS-I-0035).
- [`fidius-host::executor::cdylib`](rust/fidius-host/executor/cdylib/): `CdylibExecutor` — the cdylib execution backend: vtable/FFI dispatch with the bincode wire format.
- [`fidius-host::executor::name_lookup`](rust/fidius-host/executor/name_lookup/): Resolve-and-pin for `wasi:sockets` name lookups (FIDIUS-I-0034).
- [`fidius-host::executor::python`](rust/fidius-host/executor/python/): `Pyo3Executor` — the Python execution backend, behind the `python` feature.
- [`fidius-host::executor::wasm`](rust/fidius-host/executor/wasm/): `WasmComponentExecutor` — the sandboxed WASM (Component Model) backend.
- [`fidius-host::handle`](rust/fidius-host/handle/): `PluginHandle` — the unified, caller-facing proxy over a loaded plugin.
- [`fidius-host::host`](rust/fidius-host/host/): PluginHost builder and plugin discovery.
- [`fidius-host::host_import`](rust/fidius-host/host_import/): Host-import discovery and binding (plugin → host callback channel).
- [`fidius-host::loader`](rust/fidius-host/loader/): Core plugin loading and descriptor validation.
- [`fidius-host::package`](rust/fidius-host/package/): Host-side package integration: manifest loading, discovery, and building.
- [`fidius-host::signing`](rust/fidius-host/signing/): Ed25519 signature verification for plugins: dylibs (sign the file bytes) and packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
- [`fidius-host::stream`](rust/fidius-host/stream/): Server-streaming dispatch (FIDIUS-I-0026).
- [`fidius-host::types`](rust/fidius-host/types/): Owned metadata types for loaded plugins.
- [`fidius-macro`](rust/fidius-macro/)
- [`fidius-macro::host_interface`](rust/fidius-macro/host_interface/): Code generation for `#[host_interface]` — the plugin → host callback channel (the reverse direction of `#[plugin_interface]`).
- [`fidius-macro::impl_macro`](rust/fidius-macro/impl_macro/): Code generation for `#[plugin_impl(TraitName)]`.
- [`fidius-macro::interface`](rust/fidius-macro/interface/): Code generation for `#[plugin_interface]`.
- [`fidius-macro::ir`](rust/fidius-macro/ir/): Intermediate representation for parsed plugin interface traits.
- [`fidius-macro::wit`](rust/fidius-macro/wit/): WIT mapping/rendering for the macro's WASM codegen.
- [`fidius-test`](rust/fidius-test/): Testing helpers for Fidius plugin authors and hosts.
- [`fidius-test::dylib`](rust/fidius-test/dylib/): Build-and-cache helpers for plugin cdylib fixtures.
- [`fidius-test::signing`](rust/fidius-test/signing/): Deterministic signing fixtures for tests that exercise Fidius signature verification flows.
- [`fidius-test::stream`](rust/fidius-test/stream/): Composition harness for streaming plugins — **for tests, not production**.
