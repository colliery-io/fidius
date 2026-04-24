---
id: resolve-unimplemented-timeout
level: task
title: "Resolve unimplemented timeout semantics in PluginError/host API"
short_code: "FIDIUS-T-0083"
created_at: 2026-04-22T12:37:05.454990+00:00
updated_at: 2026-04-23T04:00:34.791884+00:00
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

# Resolve unimplemented timeout semantics in PluginError/host API

## Type

Documentation / API-contract clarification.

## Background — the original premise was wrong

This ticket was originally framed as "remove an unimplemented `PluginError::Timeout(u64)` variant from fidius's public API." That premise was incorrect: fidius's `PluginError` is a struct (`{ code, message, details }`), not an enum, and contains no `Timeout` variant. `CallError` likewise has no timeout-related variants. `grep -rin "timeout"` across `crates/` returns zero hits. The variant I claimed existed was actually from `pluggable-poc/crates/pipeline-types/src/lib.rs::PluginError`, which is a separate codebase used only for the boundary-overhead exploration. I conflated the two when writing the original FR.

So there is no API/reality mismatch in fidius today. There is, however, a real adjacent issue worth a small, focused fix.

## Real (smaller) problem

Fidius has no built-in timeout, cancellation, or deadline semantics. Plugin method calls run to completion or panic. This is the right behaviour for the current cdylib + in-process architecture (Rust cannot safely interrupt a thread mid-FFI-call), but the documentation does not say so explicitly. A consumer reading the public docs has no signal one way or the other and may assume the framework protects them from runaway plugins.

The risk is downstream: cloacina plugins, future fidius-python plugins, or any consumer running untrusted plugin code may carry implicit assumptions about timeout enforcement that fidius does not honour.

## Desired Capability

Make the absence of timeout/cancellation explicit in fidius's public docs so consumers know to add their own watchdog when they need that guarantee.

## Scope

- Add a short note to the `fidius` facade crate's top-level docs covering: no built-in timeouts, no cancellation, plugin calls run to completion, and the suggested consumer-side mitigation (subprocess-isolation watchdog if the threat model requires it).
- Add a one-line cross-reference on `PluginHandle::call_method` and `call_method_raw` pointing readers to that note.
- Note that the upcoming `fidius-python` initiative (FIDIUS-I-0020) is the natural place for fidius itself to grow timeout semantics — only its Process tier can enforce deadlines, per the POC findings.

## Non-Goals

- Implementing subprocess isolation or any real timeout mechanism. That belongs to the fidius-python initiative if/when it ships a Process tier.
- Adding new public types, traits, or error variants.
- Changing any runtime behaviour.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fidius` facade docs include an explicit "no built-in timeouts" subsection.
- [ ] `PluginHandle::call_method` and `call_method_raw` rustdoc point to it.
- [ ] `cargo doc -p fidius` builds without warnings; no broken intra-doc links.
- [ ] `angreal lint` and `angreal check` still clean.

## Related

- Original (incorrect) framing pointed at `PluginError::Timeout` — see Background.
- FIDIUS-I-0020 (fidius-python) is the natural carrier for any future implemented timeout capability via its Process tier.

## Status Updates

### 2026-04-23 — implementation landed

Files touched:

- `crates/fidius/src/lib.rs`: added a "What fidius does *not* provide: timeouts and cancellation" section to the facade crate's top-level docs, explaining the architectural reason (cdylib + in-process synchronous calls; Rust cannot safely interrupt a thread mid-FFI), the recommended consumer-side mitigation (host-process supervisor with SIGKILL on deadline), and noting that fidius-python is the natural future carrier for first-class timeout semantics via a subprocess tier.
- `crates/fidius-host/src/handle.rs`: added a "No timeout" doc subsection to `PluginHandle::call_method`, and a one-line cross-reference on `call_method_raw`.

Verification:

- `cargo doc -p fidius --no-deps` → builds clean, no warnings.
- `cargo doc --no-deps -p fidius -p fidius-host` → no warnings or errors.
- `angreal lint` → clean.
- `angreal check` → clean.

Acceptance criteria:

- [x] Facade docs include explicit "no built-in timeouts" subsection.
- [x] `call_method` and `call_method_raw` rustdoc point to it.
- [x] `cargo doc` builds without warnings; no broken intra-doc links.
- [x] `angreal lint` and `angreal check` clean.