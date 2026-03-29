---
id: critical-safety-fixes-architecture
level: initiative
title: "Critical Safety Fixes — Architecture Review Phase 1"
short_code: "FIDIUS-I-0007"
created_at: 2026-03-29T16:28:44.596997+00:00
updated_at: 2026-03-29T17:08:46.650061+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: critical-safety-fixes-architecture
---

# Critical Safety Fixes — Architecture Review Phase 1

## Context

The architecture review (see `review/09-report.md` and `review/10-recommendations.md`) identified 13 Critical findings across 7 lenses, with 12 being runtime issues at the FFI trust boundary. This initiative addresses all Critical findings plus closely related Major findings, corresponding to recommendations R-01 through R-06, R-13, R-15, and R-16 from the review.

## Goals & Non-Goals

**Goals:**
- Fix `free_buffer` capacity mismatch (UB on every method call) — R-01
- Add `method_count` to `PluginDescriptor` + bounds-check vtable access — R-02
- Add null-pointer check on output buffer — R-03
- Move signature verification before `dlopen` — R-04
- Replace panics with Result returns for descriptor parsing — R-05
- Fix `detect_architecture` to read only header bytes — R-06
- Fix `verify` command `process::exit` → error return — R-13
- Restrict secret key file permissions — R-15
- Fix `LoadPolicy::Lenient` signature semantics — R-16

**Non-Goals:**
- Phase 2 (usability), Phase 3 (structural), Phase 4 (architectural) — separate initiatives
- Typed host proxy (R-21) — structural improvement, not safety fix
- ABI evolution strategy (R-19) — design work, deferred
- Observability/tracing (R-09) — important but not safety-critical

## Implementation Plan

Independent fixes first (no dependencies), then dependent fixes:

```
Batch 1 (no deps):    R-01, R-03, R-05, R-06, R-13, R-15
Batch 2 (after R-01): R-02 (method_count + bounds checking)
Batch 3 (after R-05): R-04 (signature before dlopen)
Batch 4 (after R-04): R-16 (Lenient semantics)
```