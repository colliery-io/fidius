---
id: p2-fuzz-target-for-fid-archive
level: task
title: "P2 — Fuzz target for .fid archive extraction (untrusted tar / FIDIUS-T-0084 path) + corpus"
short_code: "FIDIUS-T-0182"
created_at: 2026-06-23T17:32:35.597980+00:00
updated_at: 2026-06-23T22:17:05.330543+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P2 — Fuzz target for .fid archive extraction (untrusted tar / FIDIUS-T-0084 path) + corpus

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 2 (fuzzing the wire/FFI boundary).

## Objective

Add a `cargo-fuzz` target for `.fid` archive extraction — the safe-extraction path
from FIDIUS-T-0084 (untrusted tar) — with a committed seed corpus.

## Acceptance Criteria

## Acceptance Criteria

- [x] A fuzz target exercises the `.fid` archive extraction / safe-extraction path.
- [x] A seed corpus is committed.
- [x] Crashers (path traversal, decompression bombs, malformed entries) would surface
      as reproducible fuzz findings (mechanism in place; none found — see status).

## Implementation Notes

This is the untrusted-tar boundary hardened in FIDIUS-T-0084; the fuzz target is the
adversarial check that the validation actually holds.

### Dependencies
[[FIDIUS-T-0181]] (shares the `fuzz/` harness). CI wiring is T-0183.

## Status Updates

**2026-06-23 — implemented + verified.** Added `fuzz_targets/fid_extract.rs` (4th
target in the Phase-2 fuzz crate): writes arbitrary bytes to a temp `.fid` and runs
`fidius_host::package::unpack_fid` (the tar+bzip2 safe-extraction path hardened in
FIDIUS-T-0084). Seeded with a real `.fid` (`corpus/fid_extract/valid.fid`, generated
via `pack_package` over a minimal package dir). 20s smoke (`-max_total_time=20`):
**no crashers** — 37,880 runs, +104 corpus units (57 total). No `crash-*`/`leak-*`/
`oom-*` artifacts; `unpack_fid` rejected every malformed/adversarial archive without
panicking or escaping the dest dir — i.e. the T-0084 hardening holds under fuzzing.
`artifacts/` is git-ignored; the curated corpus (228 KB) is committed. CI smoke is
T-0183.