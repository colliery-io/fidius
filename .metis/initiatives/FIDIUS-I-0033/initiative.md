---
id: test-quality-coverage-measurement
level: initiative
title: "Test quality & coverage — measurement, fuzzing, mutation, property testing"
short_code: "FIDIUS-I-0033"
created_at: 2026-06-23T17:27:58.195600+00:00
updated_at: 2026-06-23T17:27:58.195600+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: test-quality-coverage-measurement
---

# Test quality & coverage — measurement, fuzzing, mutation, property testing

## Context **[REQUIRED]**

Fidius is being picked up by more downstream consumers (host apps + white-label re-export
crates). The library is solid and well-tested by feel — `fidius-host` alone has ~42 test
files, `fidius-macro` ships `trybuild` compile-fail tests, and there are polyglot WASM
fixtures in Rust/Python/JS/Go/C — but **test quality is currently unmeasured and
unenforced**. As adoption grows we need objective signal on where the gaps are and a
pipeline that keeps quality from sliding.

Today there is:
- **No coverage measurement** anywhere (no `cargo-llvm-cov` / `tarpaulin`).
- **No generative or adversarial testing** (no `proptest`, `cargo-fuzz`, `cargo-mutants`).
- A `fidius` facade crate with **0 tests of its own** (only re-export compile-guards).

This initiative establishes coverage as the baseline instrument, then layers adversarial
strategies (fuzzing, mutation, property testing) onto the highest-value surfaces — the
untrusted-input boundaries (wire format, descriptor parsing, `.fid` archive extraction) and
the core logic the macro generates against.

**Decisions locked with the maintainer (2026-06-23):**
1. **Coverage gate posture:** *report-only first*. Measure and publish every run; do **not**
   fail CI. Establish real baselines and find dead paths before committing to a number or a
   ratchet. (Ratchet/threshold can be a later follow-on.)
2. **Reporting/visibility:** *in-CI summary + artifact*. Render the per-crate coverage table
   into the GitHub Actions job summary and upload HTML/lcov as a build artifact. **No
   external service** (no Codecov), no token, no third-party data sharing.
3. **Advanced strategies (all four, phased in this order):** fuzzing the wire/FFI boundary →
   mutation testing the core → property tests → expand the backend fixture matrix.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Coverage is measured on every CI run with a per-crate breakdown in the job summary and a
  downloadable HTML/lcov artifact, runnable locally via an `angreal coverage` task. Report-only.
- A documented coverage baseline (per-crate numbers + an explicit list of the worst gaps),
  with the worst gaps filed as follow-up backlog items.
- A `cargo-fuzz` harness with targets on the untrusted-input surfaces (wire decode,
  `PluginDescriptor` parse/validation, `.fid` archive extraction), smoke-run in CI.
- A `cargo-mutants` baseline on `fidius-core` (and a scoped pass on `fidius-macro`) with
  high-value surviving mutants killed by new tests; mutation run wired into CI on a
  schedule (it is slow), report-only.
- `proptest` round-trip / invariant tests for the wire format, WIT type mapping, and
  multi-arg tuple packing.
- Expanded fixture/permutation coverage across backends (cdylib / Python / WASM) and the
  streaming × egress × config feature axes, plus first-class tests for the `fidius` facade.

**Non-Goals:**
- A hard coverage threshold or ratchet gate (deferred; revisit after baselines exist).
- External coverage services / dashboards (Codecov etc.) — explicitly declined.
- Rewriting existing tests or chasing 100% on FFI/`unsafe`/platform-specific paths that are
  legitimately hard to cover; coverage is a *map*, not a target.
- New runtime features — this initiative adds only tests, harnesses, and pipeline wiring.

## Detailed Design **[REQUIRED]**

Decomposed into five phases. Phase 1 is the instrument everything else reads against;
Phases 2–5 are independent and can proceed in parallel once Phase 1 lands.

### Phase 1 — Coverage measurement (report-only, in-CI summary + artifact)
- Tool: **`cargo-llvm-cov`** (source-based LLVM coverage). Chosen over tarpaulin for accurate
  source-based coverage and clean multi-crate workspace support across the OS matrix.
- An `angreal coverage` task: `cargo llvm-cov --workspace` with the feature surface that
  matters (incl. `wasm` / `streaming`), emitting HTML + lcov locally.
- CI job: install `cargo-llvm-cov`, run, render the per-crate table into
  `$GITHUB_STEP_SUMMARY`, upload HTML+lcov as an artifact. **Never fails the build.** Must
  account for the existing dev/release/wasm job split (the `wasm` feature needs the component
  toolchain, mirroring the existing `wasm` CI job).
- Baseline pass: capture per-crate numbers, write them down, enumerate the worst gaps (notably
  the `fidius` facade with 0 tests and known error-path gaps), file follow-ups.

### Phase 2 — Fuzzing the wire/FFI boundary
- `cargo-fuzz` (libFuzzer, nightly) in a `fuzz/` workspace member.
- Targets, highest-value untrusted surfaces first:
  - wire-format decode (+ decode→encode→decode round-trip stability),
  - `PluginDescriptor` parse / bounds / validation,
  - `.fid` archive extraction (the safe-extraction path from FIDIUS-T-0084 — untrusted tar).
- Seed corpora committed. CI runs a **time-boxed smoke** per PR (find crashers fast without
  long jobs); longer campaigns are documented for local/scheduled runs.

### Phase 3 — Mutation testing the core
- `cargo-mutants` on `fidius-core` first (wire, descriptor, FNV-1a hashing) — verifies the
  *existing* tests actually catch logic changes, not just execute lines.
- Triage the baseline missed-mutant report; add targeted tests to kill high-value survivors.
- Scoped pass on `fidius-macro` (codegen/IR logic is higher-effort — bound the scope).
- Wire into CI **on a schedule** (nightly/weekly), report-only — mutation runs are slow.

### Phase 4 — Property tests
- `proptest` dev-dependency.
- Invariants: `decode(encode(v)) == v` over arbitrary `Value` trees; WIT type-mapping
  round-trips; multi-arg tuple-pack/unpack over arbitrary arities. Generative tests subsume
  many hand-written cases and tend to surface the exact inputs fuzzing/mutation also probe.

### Phase 5 — Expand the backend matrix
- First-class tests for the `fidius` facade crate (currently 0 own tests) across feature
  combos (`host`, `wasm`, `streaming`).
- Broaden fixtures/permutations across backends × streaming × egress × config so the
  cross-product the library actually supports is exercised, not just the happy paths.

## Implementation Plan / Task Decomposition **[REQUIRED]**

Vertical slices. Phase 1 is sequential (1→2→3 within the phase); Phases 2–5 parallelizable
after Phase 1.

> Task short codes below are assigned by Metis on creation (the original authoring assumed
> `T-0178…T-0190`); the live codes are whatever the created task documents carry.

| Phase | Task |
| ----- | ---- |
| 1 | `angreal coverage` task via `cargo-llvm-cov` (workspace + wasm/streaming features; local HTML+lcov) + dev-setup docs |
| 1 | CI coverage job — install llvm-cov, run, render per-crate table to `$GITHUB_STEP_SUMMARY`, upload HTML+lcov artifact, report-only (handle wasm/component-toolchain split) |
| 1 | Baseline analysis — capture per-crate numbers, document worst gaps, file follow-up backlog items |
| 2 | `cargo-fuzz` harness crate + first targets: wire decode round-trip, `PluginDescriptor` parse/validation; committed seed corpora |
| 2 | Fuzz target for `.fid` archive extraction (untrusted tar / FIDIUS-T-0084 path) + corpus |
| 2 | CI fuzz smoke — time-boxed per-PR run, corpus persistence, docs for longer local/scheduled campaigns |
| 3 | `cargo-mutants` on `fidius-core` — baseline survivor report + kill high-value survivors with new tests |
| 3 | `cargo-mutants` scoped pass on `fidius-macro` (IR/codegen) — baseline + targeted kills |
| 3 | CI mutation run on a schedule (nightly/weekly), report-only |
| 4 | `proptest` + wire-format round-trip invariants over arbitrary `Value` trees |
| 4 | `proptest` for WIT type mapping + multi-arg tuple packing invariants |
| 5 | First-class tests for the `fidius` facade crate across feature combos (currently 0 own tests) |
| 5 | Expand fixture/permutation matrix across backends × streaming × egress × config |

## Acceptance Criteria **[REQUIRED]**

- [ ] `angreal coverage` produces a per-crate HTML+lcov report locally; documented in dev setup.
- [ ] Every CI run publishes a per-crate coverage table in the job summary + an HTML/lcov
      artifact, and never fails on coverage.
- [ ] A written coverage baseline exists with the worst gaps enumerated and filed as follow-ups.
- [ ] `cargo-fuzz` targets exist for wire decode, descriptor parse, and `.fid` extraction, with
      committed corpora and a time-boxed CI smoke run.
- [ ] `cargo-mutants` baseline captured for `fidius-core` (+ scoped `fidius-macro`); high-value
      survivors killed; a scheduled report-only CI mutation run is wired in.
- [ ] `proptest` round-trip/invariant tests cover the wire format, WIT type mapping, and
      multi-arg tuple packing.
- [ ] The `fidius` facade has first-class tests; the backend × streaming × egress × config
      permutation matrix is meaningfully broadened.

## Risks & Mitigations

- **WASM/component-toolchain coupling in CI coverage** — the `wasm` feature needs the full
  component toolchain. Mitigation: model the coverage job on the existing `wasm` CI job; if
  instrumenting the wasm path is too heavy, scope coverage to the native feature set first and
  add the wasm slice as a follow-on (documented, not silently dropped).
- **Fuzz/mutation runtime blowing up CI time** — both are slow. Mitigation: per-PR is a
  time-boxed *smoke* only; full campaigns run scheduled/local. Report-only, never gating.
- **Nightly toolchain for `cargo-fuzz`** — pin the nightly used and isolate it to the fuzz job.
- **Report-only coverage gets ignored** — accepted trade-off per the maintainer's call; the
  baseline + follow-up backlog items are the forcing function until a ratchet is added later.

## Status Updates **[REQUIRED]**

- **2026-06-23 — Initiative authored.** Strategic decisions locked with maintainer (report-only
  coverage; in-CI summary+artifact, no external service; all four advanced strategies in the
  order fuzz → mutation → property → matrix). Decomposed into 13 tasks across 5 phases. Lands
  as documentation alongside the FIDIUS-I-0033 (TCP egress) PR; implementation happens off main
  afterward.
- **2026-06-23 — Imported into Metis.** Originally authored as a standalone root markdown
  because the Metis MCP DB was locked at creation time; now filed as a proper initiative with
  its 13 tasks created under it.
- **2026-06-23 — Phase 1 COMPLETE (coverage measurement).** `angreal coverage` task
  (T-0178) + report-only CI coverage job (T-0179) + baseline (T-0180) all done.
  Baseline: **TOTAL 76.56% region** (native + `streaming`). Worst gaps filed: `fidius`
  facade 0% → T-0189 (in-plan); Python dispatch/error paths → [[FIDIUS-T-0191]]; cdylib
  executor error paths → [[FIDIUS-T-0192]]. **Key finding/decision:** the `wasm` feature
  can't be instrumented by cargo-llvm-cov — its tests build `wasm32-wasip2` component
  fixtures at test time and those sub-builds reject `-C instrument-coverage`. Per the
  initiative's own risk mitigation, coverage is scoped to the **native + streaming**
  surface (task + CI + docs all reflect this); `executor/wasm.rs` is a documented blind
  spot and full wasm coverage is a follow-on. Now starting Phase 2 (fuzzing); cargo-fuzz +
  cargo-mutants installed.
- **2026-06-23 — Phases 4 & 5 COMPLETE (done out of order, before finishing
  mutation, per maintainer).** Phase 4 (property tests): `proptest` added to
  `fidius-guest` + `fidius-wit`; wire bincode round-trip + `Value` bridge over
  arbitrary concrete trees, WIT type-mapping (model-driven), and tuple-pack arities
  0–4 (T-0187/8). **Key finding:** `Value` can't cross the bincode wire
  (`deserialize_any` is unsupported), so the wire invariants are over concrete types
  — this also corrected the T-0181 `wire_value` fuzz target. Phase 5: 4 facade test
  files across host/wasm/streaming combos (T-0189, was 0 tests); matrix mapped +
  `validate_runtime` config×backend cells filled (T-0190); the one real gap
  (egress×streaming) filed as [[FIDIUS-T-0193]]. Remaining: finish Phase 3 mutation
  (T-0185 macro triage — baseline already run, 7 survivors; T-0186 CI schedule —
  workflow drafted).
- **2026-06-23 — Phase 3 COMPLETE (mutation testing); ALL 13 TASKS DONE.** T-0184:
  fidius-core baseline (61 mutants) → 6 high-value `.fid`-safety survivors killed.
  T-0185: scoped fidius-macro `ir.rs` baseline (50 mutants, 7 missed) → 4 IR-helper
  survivors killed (7→3; the 3 are the 64-optional-method bound, documented). T-0186:
  scheduled report-only `mutation.yml` (weekly + dispatch). **Initiative ready for
  maintainer review** — all five phases delivered; 3 follow-up gaps filed to backlog
  ([[FIDIUS-T-0191]] python coverage, [[FIDIUS-T-0192]] cdylib executor coverage,
  [[FIDIUS-T-0193]] egress×streaming fixture).
- **2026-06-23 — Phase 2 COMPLETE (fuzzing).** Standalone `cargo-fuzz` crate at
  `crates/fidius-host/fuzz/` with 4 targets: `wire_value` (bincode Value decode +
  round-trip), `frame_read` (framed streaming wire / bounds), `manifest_validate`
  (manifest parse + `validate_runtime`), `fid_extract` (`.fid` safe extraction, T-0084).
  All build + smoke-ran clean — **no crashers** across ~17M+ execs; committed seed
  corpora (~490 KB). Per-PR `fuzz-smoke` CI job (60s/target, corpus cached, report-only)
  + a Fuzzing how-to. **Note:** `PluginDescriptor` is `#[repr(C)]` FFI (not
  byte-parseable), so the manifest-validate path stands in for "descriptor parse/
  validation". Starting Phase 3 (mutation testing).
