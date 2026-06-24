---
id: cover-the-python-backend-dispatch
level: task
title: "Cover the Python backend dispatch/error paths (fidius-python handle/loader/value_bridge)"
short_code: "FIDIUS-T-0191"
created_at: 2026-06-23T22:04:35.206460+00:00
updated_at: 2026-06-23T22:04:35.206460+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Cover the Python backend dispatch/error paths (fidius-python handle/loader/value_bridge)

## Objective

The FIDIUS-I-0033 coverage baseline (2026-06-23, native + streaming) shows the
Python backend as the weakest-covered area of the workspace:

- `fidius-python/src/handle.rs` — **26%** region
- `fidius-python/src/loader.rs` — **46%**
- `fidius-python/src/value_bridge.rs` — **64%**

These are mostly dispatch and error-mapping paths (PyErr → CallError, value
conversion failures, loader discovery errors). Add targeted tests so the Python
backend's failure modes are exercised, not just the happy path. (Some paths need an
embedded interpreter; scope to what the existing pyo3 test setup can reach.)

## Acceptance Criteria

- [ ] `handle.rs` / `loader.rs` / `value_bridge.rs` error paths gain meaningful
      coverage (well above the baseline).
- [ ] Tests cover PyErr→CallError mapping and value-conversion failures.

## Notes

Filed from the FIDIUS-I-0033 Phase 1 baseline (T-0180). Report-only — no gate; this
is a prioritized gap, not a regression.
