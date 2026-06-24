---
id: p3-cargo-mutants-on-fidius-core
level: task
title: "P3 — cargo-mutants on fidius-core: baseline survivor report + kill high-value survivors with new tests"
short_code: "FIDIUS-T-0184"
created_at: 2026-06-23T17:32:38.867340+00:00
updated_at: 2026-06-23T22:50:07.334967+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P3 — cargo-mutants on fidius-core: baseline survivor report + kill high-value survivors with new tests

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 3 (mutation testing the core).

## Objective

Run `cargo-mutants` on `fidius-core` (wire, descriptor, FNV-1a hashing) to verify
the *existing* tests actually catch logic changes (not just execute lines). Triage
the baseline missed-mutant report and add targeted tests to kill high-value
survivors.

## Acceptance Criteria

## Acceptance Criteria

- [x] A `cargo-mutants` baseline survivor report is captured for `fidius-core`.
- [x] High-value surviving mutants are killed by new targeted tests.
- [x] The triage (which survivors were killed vs. accepted, and why) is documented.

## Implementation Notes

Start with `fidius-core` — the highest-value, most-depended-on logic. Mutation runs
are slow; CI scheduling is T-0186.

### Dependencies
Phase 1 baseline (T-0180) helps prioritize but isn't a hard blocker.

## Status Updates

**2026-06-23 — baseline + kills done.** `cargo mutants --package fidius-core`:
**61 mutants → 38 caught, 10 missed, 13 unviable** (11 min). All 10 survivors were
in `package.rs`. Added 5 targeted tests (`package.rs` test module) and re-ran
(`-f '**/package.rs'`): **10 → 4 missed** — every high-value survivor killed.

**Killed (6):**
- `676` entry-count cap `count > max_entries` (`==` + `>=`) → `unpack_entry_count_is_an_exact_boundary` (exactly-max succeeds).
- `725` size cap `total > max_decompressed` (`==` + `>=`) → `unpack_size_budget_is_an_exact_boundary`.
- `731:53` ratio guard `max_ratio > 0` (`>=`) → `unpack_zero_max_ratio_disables_ratio_check`.
- `138` `PackageHeader::runtime` `Some("wasm")` arm → `runtime_string_maps_to_each_variant`.

These are the `.fid` safe-extraction safety limits (T-0084) — tests now pin the exact
off-by-one, not just "way over".

**Accepted (4), with rationale:**
- `136` delete `None | Some("rust")` arm — **equivalent**: falls through to the
  `_ => Rust` default, identical result.
- `731:28` `compressed_size > 0` → `>=` — **equivalent**: `compressed_size` is a real
  file size, always `> 0`, so `>` and `>=` never differ.
- `731:66` `total > ratio_cap` → `>=` — ratio off-by-one. `ratio_cap = compressed_size
  × max_ratio` depends on nondeterministic bzip2 output, so pinning `total == ratio_cap`
  exactly is impractical; the ratio *rejection* is covered (`unpack_rejects_ratio_bomb`).
- `418` `name_str == ".git"` → `!=` in `collect_files` — feeds the package **content
  hash**'s directory walk (a `.git` edge), not the packed tar or any security check.
  Low value; left as-is.

fmt + clippy clean. Mutants output dirs are git-ignored. CI scheduling is T-0186.