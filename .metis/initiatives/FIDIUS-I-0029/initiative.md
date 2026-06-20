---
id: configured-plugin-instances-bind
level: initiative
title: "Configured plugin instances — bind config once, call many (partial application)"
short_code: "FIDIUS-I-0029"
created_at: 2026-06-20T01:41:51.753814+00:00
updated_at: 2026-06-20T01:44:38.309434+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: configured-plugin-instances-bind
---

# Configured plugin instances — bind config once, call many (partial application) Initiative

> The headline of **0.5.0**. ABI shape decided in [[FIDIUS-A-0006]] (option A — unify). Pairs with the connector arc ([[FIDIUS-I-0028]] guest HTTP, [[FIDIUS-I-0026]] streaming).

## Context **[REQUIRED]**

A fidius plugin is a singleton today: the framework constructs one instance and
methods take their args per call. A **connector** wants the opposite — bind a
config once (`{url, page_size, …}`), then call methods that close over it
(`functools.partial`). Per-call config args (a) re-marshal the config across the
boundary on *every* call and (b) give the plugin nowhere to do config-bound setup
once (open a pool, compile a schema). This initiative adds host-constructed,
configured plugin **instances**.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- `configure(cfg) -> handle`, then `handle.method()`, drop = destroy — on all
  three backends, one uniform author/host surface.
- The unified construct/destroy cdylib ABI from [[FIDIUS-A-0006]] (singleton =
  construct-with-`()`); `ABI_VERSION` 400→500.
- WASM via a Component-Model `resource` (reuse the streaming-resource machinery);
  Python via a class constructed with config.
- Config crosses the boundary once; config-bound init runs once; N differently-
  configured instances of one plugin in one host.

**Non-Goals:**
- Per-*call* statefulness beyond the bound config (no mutable session state across
  calls beyond what the instance holds) — instances are the unit of state.
- Hot-reconfiguration (re-`configure` = drop + construct a new handle).
- Changing the streaming or egress contracts (they compose unchanged).

## Requirements **[REQUIRED]**

### Functional
- REQ-001: A trait impl declares a constructor `fn configure(cfg: C) -> Self` via
  `#[plugin_impl(Trait, config = C)]`; a plugin without `config =` gets a unit
  constructor automatically (the singleton case).
- REQ-002 (cdylib): descriptor gains `construct(cfg_ptr,len) -> *mut instance` +
  `destroy(*mut instance)`; vtable methods take a leading `instance` pointer.
- REQ-003 (WASM): the component exports a `resource` with a constructor taking the
  config and methods on the resource; drop runs the destructor.
- REQ-004 (Python): the plugin class is instantiated with the deserialized config.
- REQ-005 (host): `host.configure::<C>(name, &desc, cfg) -> ConfiguredHandle`;
  methods dispatch on the handle; drop destroys. The existing zero-config load path
  stays ergonomic (sugar over `configure(.., ())`).
- REQ-006: composes with streaming (`handle.read() -> Stream<Record>`) and egress
  (a configured wasm connector still rides the two-key gate).

### Non-Functional
- NFR-001: `ABI_VERSION` 400→500; layout/offset tests + ABI docs updated in
  lockstep; the macro + descriptor + host move together (no half-broken ABI).
- NFR-002: per-call overhead is one pointer arg (no measurable hot-path cost).

## Use Cases **[REQUIRED]**

### UC1: a configured REST connector (the live driver)
- **Actor**: a macro-authored connector `configure(SourceConfig) -> Self` whose
  `read(&self) -> Stream<Record>` paginates `self.cfg.url` via `fidius_guest::http`.
- **Scenario**: host `configure`s it once with `{url, page_size}`; the instance
  opens its client once; `read()` streams records; config never re-crosses.
- **Outcome**: the partial-application connector — bound once, called clean.

## Architecture **[REQUIRED]**

See [[FIDIUS-A-0006]] for the ABI. Construction is the load-bearing piece, so build
**cdylib first** (hardest ABI), then WASM (resource), then Python, then the macro
surface that unifies them, then the host `ConfiguredHandle` + E2E.

## Alternatives Considered **[REQUIRED]**

Per [[FIDIUS-A-0006]]: (A) unify chosen; (B) parallel optional ABI rejected (two
shapes forever); (C) singleton `init(cfg)` rejected (one config/dylib, no real
instances). Host-side partial (hold cfg in host code, pass per call) is the
*current* answer and stays valid for call-once/streaming shapes — this initiative
is for the call-many / config-bound-init shapes where re-passing is waste.

## Implementation Plan **[REQUIRED]**

Decompose after sign-off:
- **CI.1 — cdylib construct/destroy ABI.** Descriptor fields + vtable
  instance-pointer convention + registry/macro `construct`/`destroy`; `ABI_VERSION`
  400→500; layout/offset tests + ABI-spec doc. Singleton = unit construct.
- **CI.2 — host `ConfiguredHandle` + configure() API.** `host.configure::<C>()`,
  per-instance dispatch, drop→destroy; cdylib E2E (config bound once, N instances).
- **CI.3 — WASM resource construction.** Macro emits a `resource` w/ config
  constructor; executor instantiates + dispatches on it; composes with streaming +
  egress; E2E.
- **CI.4 — Python configured class.** Construct the plugin class with config; E2E.
- **CI.5 — docs + the configured-connector example.** `partial`-application story;
  update the streaming/egress connector example to a configured instance.