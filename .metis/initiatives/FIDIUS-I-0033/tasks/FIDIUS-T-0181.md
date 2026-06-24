---
id: p2-cargo-fuzz-harness-crate-first
level: task
title: "P2 — cargo-fuzz harness crate + first targets (wire decode round-trip, PluginDescriptor parse/validation) + seed corpora"
short_code: "FIDIUS-T-0181"
created_at: 2026-06-23T17:32:34.248842+00:00
updated_at: 2026-06-23T22:14:02.676479+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P2 — cargo-fuzz harness crate + first targets (wire decode round-trip, PluginDescriptor parse/validation) + seed corpora

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 2 (fuzzing the wire/FFI boundary).

## Objective

Stand up a `cargo-fuzz` (libFuzzer, nightly) harness as a `fuzz/` workspace member
with the first, highest-value targets on the untrusted-input surfaces: wire-format
decode (plus decode→encode→decode round-trip stability) and `PluginDescriptor`
parse / bounds / validation. Commit seed corpora.

## Acceptance Criteria

## Acceptance Criteria

- [x] A `fuzz/` crate exists using `cargo-fuzz` (libFuzzer).
- [x] Fuzz targets for wire decode (+ round-trip stability) and descriptor/manifest
      parse/validation.
- [x] Seed corpora committed for each target.

## Implementation Notes

- Highest-value untrusted surfaces first. Pin the nightly used by the fuzz job and
  isolate it from the rest of CI.

### Dependencies
Phase 1 ideally lands first (so gaps are known), but Phase 2 is parallelizable
after Phase 1. CI wiring is T-0183.

## Status Updates

**2026-06-23 — implemented + verified.** Created `crates/fidius-host/fuzz/` — a
standalone `cargo-fuzz` crate (own `[workspace]`, detached from the main workspace,
nightly-only). Three targets:
- `wire_value` — `wire::deserialize::<Value>` robustness + decode→encode→decode
  round-trip stability.
- `frame_read` — `Frame::read` (length-prefixed streaming wire / bounds) robustness
  + round-trip.
- `manifest_validate` — arbitrary TOML → `PackageManifest` → `validate_runtime`.

**Design note:** `PluginDescriptor` is a `#[repr(C)]` FFI struct (pointers, not
byte-parseable), so the analogous untrusted-parse surface chosen for "descriptor
parse/validation" is the package-manifest parse + `validate_runtime`, plus the
framed wire decoder. Documented in each target's header.

**Verified:** `cargo +nightly fuzz build` green (all 3 link). 15s smoke each — **no
crashers** across ~17M executions total (wire 12.8M, frame 3.6M, manifest 0.5M).
Seed corpora committed (260 KB total): `wire_value` 6 (generated valid `Value`
encodings — the strict bincode decoder finds none from random bytes), `frame_read`
30, `manifest_validate` 29 + a hand-written valid manifest. `fuzz/.gitignore` keeps
`target/`/`artifacts/` out; corpus is tracked. CI smoke wiring is T-0183.

**2026-06-23 — correction (found during Phase 4).** The original `wire_value` target
decoded bytes as `Value`, but `Value` can't be bincode-decoded at all — its
`Deserialize` uses `deserialize_any`, which bincode (non-self-describing) rejects, so
the decode always errored and the round-trip branch was never reached (robustness-only,
vacuous round-trip). Refocused `wire_value` to decode a representative **concrete**
payload (`Vec<(String, i64)>` — the shape that actually crosses the bincode wire) so
the round-trip is real; regenerated its seeds (4 concrete encodings) and re-smoked
(no crashers, round-trip exercised). The exhaustive structural round-trip now lives in
the Phase-4 proptest (`fidius-guest/tests/proptest_wire.rs`).