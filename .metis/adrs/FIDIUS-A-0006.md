---
id: 001-configured-plugin-instances
level: adr
title: "Configured plugin instances — unified construct/destroy ABI (singleton = construct-with-unit)"
number: 1
short_code: "FIDIUS-A-0006"
created_at: 2026-06-20T01:41:51.054842+00:00
updated_at: 2026-06-20T01:43:57.294218+00:00
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

# ADR-1: Configured plugin instances — unified construct/destroy ABI (singleton = construct-with-unit)

## Context **[REQUIRED]**

Today a fidius plugin is a **singleton**: the registry constructs one instance and
the cdylib vtable methods dispatch on it; methods take their args per call. There
is no way to **configure** a plugin once and call it many times — a connector that
needs `{url, page_size, creds}` must take the config as a per-call argument, which
(a) re-marshals it across the boundary on every call and (b) gives the plugin no
place to do config-bound setup once (open a pool, compile a schema). This is the
"partial application" gap (see [[FIDIUS-I-0029]]).

The fix is host-constructed plugin **instances** (`configure(cfg) -> handle`, then
`handle.method()`, drop = destroy). The question this ADR settles is the **cdylib
ABI shape**, because that's the load-bearing, hardest-to-change surface.

## Decision **[REQUIRED]**

**Unify on a construct/destroy ABI — every plugin is constructed; the old
singleton becomes "construct with `()` config." One path, not two.**

- The descriptor gains `construct(cfg_ptr, cfg_len) -> *mut instance` and
  `destroy(*mut instance)`. The macro wires `construct` to the author's
  `fn configure(cfg: C) -> Self` (declared via `#[plugin_impl(Trait, config = C)]`);
  a plugin with no config is constructed from unit `()`.
- **Vtable methods take a leading `instance: *mut c_void`** instead of dispatching
  on a baked-in static. The host obtains an instance handle (from `construct`) and
  passes it to every method call; drop calls `destroy`.
- This **changes the cdylib calling convention → an ABI break.** It ships in
  **0.5.0**, where the pre-1.0 minor bump is *already* an ABI break
  (`ABI_VERSION` 400→500, ADR-0002) — so it costs no breakage beyond what the
  version bump implies. cdylib plugins recompile against 0.5.0 regardless.
- Cross-backend: **WASM** = a Component-Model `resource` (`configure -> handle`,
  methods on the resource, destructor on drop — reuses the streaming-resource
  machinery, FIDIUS-I-0026); **Python** = a class instantiated with config. Same
  author/host surface on all three.
- The trait stays **object-safe**: the constructor is declared on the `impl`
  (an inherent `fn configure(cfg) -> Self`), never as a `-> Self` trait method.
- Host API: `host.configure::<C>(name, &desc, cfg) -> ConfiguredHandle`; methods on
  the handle; the singleton path is `configure(.., ())` (or kept as sugar).

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|--------|------|------|------|------|
| **(A) Unify: every plugin constructed (chosen)** | One ABI path; singleton = construct-with-unit; the 0.5.0 break pays for it once; cleanest long-term | Every cdylib plugin recompiles for 0.5.0 (already required by the minor bump) | Medium | M |
| (B) Keep singleton vtable, add construct/destroy as a parallel optional path | No vtable change for singleton-only plugins | **Two ABI shapes to maintain forever**; dispatch branches; the complexity never amortizes | Medium | M |
| (C) Singleton `init(cfg)` + interior mutability | Fits the current ABI, no break | One config per loaded dylib; mutable state in a "stateless" model; can't have N differently-configured instances | Low | S |

## Rationale **[REQUIRED]**

An ABI break is expensive precisely because it forces a recompile; the way to
spend one well is to come out the *far* side simpler, not carrying a second shape.
(A) collapses "singleton" and "configured" into one concept — a plugin is always
constructed, sometimes from `()`. (B) saves a recompile we're already taking in
0.5.0 and charges interest (two code paths) forever. (C) doesn't actually deliver
configured *instances* (one config per dylib, no `read()`-with-bound-setup), so it
fails the requirement. We only break the ABI when a minor bump already does
(ADR-0002), so 0.5.0 is the right and only-for-free moment to land (A).

## Consequences **[REQUIRED]**

### Positive
- One construction model across all three backends; the host API and macro surface
  are uniform.
- Config crosses the boundary **once**; config-bound init runs **once**.
- N differently-configured instances of one plugin in a single host.
- Reuses the WASM streaming-`resource` machinery rather than inventing a lifecycle.

### Negative
- cdylib plugins recompile for 0.5.0 (already required by the `ABI_VERSION` 400→500
  minor bump — no *additional* breakage, but real).
- The descriptor + vtable + macro all change; the layout/offset tests + ABI docs
  must be updated in lockstep.

### Neutral
- "Singleton" stops being a distinct kind — it's `configure(.., ())`. Existing
  zero-config plugins keep their authoring shape (no `config =`); the macro
  supplies the unit constructor.

## Review Schedule

### Review Triggers
- A 1.0 ABI freeze (this is the moment to lock the construct/destroy convention).
- Evidence the `instance`-pointer-per-call dispatch measurably costs hot paths
  (it shouldn't — it's one pointer arg).