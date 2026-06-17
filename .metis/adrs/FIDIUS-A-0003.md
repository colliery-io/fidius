---
id: 001-wasm-plugin-wire-boundary-bincode
level: adr
title: "WASM Plugin Wire Boundary — bincode-over-linear-memory vs Component Model + WIT"
number: 1
short_code: "FIDIUS-A-0003"
created_at: 2026-06-17T02:53:20.167387+00:00
updated_at: 2026-06-17T03:12:14.145641+00:00
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

# ADR-1: WASM Plugin Wire Boundary — bincode-over-linear-memory vs Component Model + WIT

> **Status: DECIDED — Path B (Component Model + WIT).** The spike [[FIDIUS-T-0093]] recommended Path A on near-term cost grounds, but the decision-maker chose **Path B**: the deciding factor is **polyglot plugin authoring** (any-language guests), a strategic goal the spike explicitly identified as the one thing that flips the recommendation. The spike's cost/feasibility data still stands and is recorded below as the trade-off knowingly accepted.

## Context **[REQUIRED]**

Fidius is adding **pluggable execution backends** behind a single host-facing plugin API. Two backends already exist as *parallel* implementations:

- **cdylib** — `fidius-host::PluginHandle` dispatches through a `#[repr(C)]` vtable via FFI function pointers, with **bincode** as the wire format for typed args/returns (JSON was removed in FIDIUS-T-0077) and a raw byte passthrough mode for bulk payloads (`#[wire(raw)]`, FIDIUS-T-0082).
- **Python** — `fidius-python::PluginHandle` (shipped under FIDIUS-I-0020) dispatches through an embedded PyO3 interpreter with native type conversion.

The next backend is a **sandboxed WASM executor on wasmtime**. Unlike cdylib (shared address space) and PyO3 (Python objects in-process), a WASM guest runs in an isolated linear memory with no shared pointers. That forces an explicit decision about the **host↔guest data boundary**: how typed method arguments and returns cross into and out of the sandbox.

This ADR captures that fork. It does **not** decide the broader `PluginExecutor` trait refactor (the gating Phase-1 work) — that is a code-structure change, not an architectural wire-format commitment.

## Decision **[REQUIRED]**

**Path B — Component Model + WIT.**

The WASM plugin contract is defined in **WIT** (a language-neutral typed IDL). The plugin is built as a **component** (not a bare core module); the Component Model's Canonical ABI lifts/lowers typed values across the boundary, and capabilities are declared as typed WASI preview2 imports. Guests may be authored in any language with a component toolchain (Rust via `wit-bindgen`/`cargo-component`, plus Go, C/C++, JS, componentize-py, etc.).

This is chosen over the spike-recommended Path A (bincode over linear memory) because the goal is **polyglot authoring**: a single signed `.wasm` component implementing a fidius interface, writable in any component-targeting language. bincode is Rust-specific, so Path A would have left WASM plugins effectively Rust-only — the opposite of the point.

**Accepted trade-offs (from [[FIDIUS-T-0093]]):** higher implementation cost; a new toolchain in the build/pack pipeline (`cargo-component` + `wasm-tools`, absent today); a younger, faster-moving ecosystem than core-module WASM; and a typed boundary that does **not** trivially reuse fidius's existing `call_method_raw` bincode path — see the open design question in Consequences.

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **A — bincode over linear memory** | One wire format across cdylib/Python/WASM; `#[wire(raw)]` maps directly to a `(ptr,len)` memory slice; minimal new ABI surface; fastest to ship | Bespoke per-call marshalling glue (alloc/free dance); not language-agnostic — guest must speak bincode; no capability model for free | Low | Low–Medium |
| **B — Component Model + WIT** | Typed, language-agnostic ABI; canonical capability model (WASI preview2 imports as explicit capabilities); future guests in any component-targeting language | More work; WIT toolchain + bindings codegen in the macro; diverges from the bincode path the other backends use; component tooling still maturing | Medium | High |
| **C — JSON over linear memory** | Trivially debuggable | Re-introduces a wire format fidius deliberately removed (T-0077); slower; no typing benefit over A | Low | Low |

## Rationale **[REQUIRED]**

**Decided on the strategic goal, not the near-term cost.** The spike ([[FIDIUS-T-0093]], full data in `wasm-spike/FINDINGS.md`) recommended Path A because A is cheaper and reuses fidius's existing raw boundary. That recommendation was explicitly *conditional*: the spike flagged that "if polyglot-from-day-one is the real goal, that flips it to B." It is. So B is chosen with eyes open.

Why polyglot authoring outweighs the cost:

1. **bincode is Rust-specific.** Path A's wire is a Rust serialization format with no real cross-language ecosystem. A Go/JS/C# author would have to reimplement fidius's private byte protocol *and* a bincode codec — which means in practice nobody would, leaving WASM plugins Rust-only. That defeats the reason to add a WASM backend at all (the cdylib backend already serves Rust authors).
2. **WIT is language-neutral by construction.** Any language with a component toolchain implements the same `.wit` interface and produces a valid component. This is the one property that makes "write a fidius plugin in your language" true, and it cannot be bolted onto A later without effectively becoming B.
3. **The spike's other findings still hold and de-risk B.** Sandbox + deny-FS-by-default (empty-Linker proof), cheap instantiation (~14 µs), and AOT `.cwasm` loading (~83 µs) are all properties of the same wasmtime runtime and carry over to components. Only the *boundary marshalling* changes, not the isolation/perf story.

What is knowingly given up vs A: lowest cost and the "free" reuse of the raw-bytes `call_method_raw` path. See Consequences for the resulting open design question (reconciling a typed component boundary with the bincode-based cdylib/Python backends under one `PluginExecutor` trait).

## Consequences **[REQUIRED]**

### Positive
- Genuinely language-agnostic, typed plugin ABI: one `.wit`, guests in any component-targeting language, validated by the Canonical ABI rather than fidius's coarser interface-hash + bincode-decode.
- Capabilities become a declared, typed part of the contract (WASI preview2 imports), not imperative `Linker` wiring — more discoverable and auditable.
- Reuses the favourable wasmtime properties the spike measured: sandbox/deny-FS-by-default, ~14 µs instantiation, ~83 µs AOT `.cwasm` load.

### Negative
- Highest implementation cost of the options: WIT authoring, bindings codegen, and a new build/pack toolchain (`cargo-component` + `wasm-tools`) that is **not currently installed**.
- Younger, faster-moving ecosystem than core-module WASM — more churn risk.
- Does **not** reuse the existing raw-bytes `call_method_raw` path for free (see open question).

### Open design question (for the initiative's design phase)
The spike found that the cdylib and Python backends share a **raw-bytes** dispatch (`serialize → call_raw → deserialize`, bincode living in `PluginHandle`). A WIT component's boundary is **typed**, not a byte blob — so Path B does not slot into that raw path automatically. The `PluginExecutor` trait abstraction must be designed to span both worlds. Candidate approaches to evaluate:
- **(i) Typed trait method** — `PluginExecutor` exposes a typed call; cdylib/Python implement it by bincode round-trip internally, the component implements it natively via lifted/lowered WIT values. Cleanest semantically; larger trait + more refactor of the existing backends.
- **(ii) `list<u8>` in WIT** — keep fidius's bincode blob but pass it as a WIT `list<u8>` parameter, preserving a uniform raw-bytes trait. Cheapest to integrate, but forfeits Path B's typed-boundary benefit (you get component packaging + capability model + polyglot *allocation*, but the payload is still bincode — non-Rust authors still need a bincode codec). Likely defeats the polyglot rationale; documented as a tempting trap.
- **(iii) Hybrid** — typed WIT for control-plane args, `list<u8>` only for `#[wire(raw)]` bulk payloads. Probably the pragmatic target; resolve in design.

### Neutral
- Additive to existing backends: cdylib and Python paths are unaffected, and Ed25519 signing is artifact-agnostic so it extends to `.wasm` components nearly for free.

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- The initiative's design phase resolves the open `PluginExecutor` trait question (i / ii / iii above) in a way that materially contradicts this decision.
- Component Model tooling proves immature enough in practice (during Phase 2) to threaten the build/pack pipeline — would trigger reconsidering a Path A interim.
- A concrete need arises to support guests whose languages lack a usable component toolchain.