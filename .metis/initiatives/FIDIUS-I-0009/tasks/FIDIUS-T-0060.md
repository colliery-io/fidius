---
id: integration-test-pack-unpack-round
level: task
title: "Integration test — pack/unpack round-trip in full pipeline"
short_code: "FIDIUS-T-0060"
created_at: 2026-04-01T00:09:59.286380+00:00
updated_at: 2026-04-01T00:30:24.003957+00:00
parent: FIDIUS-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0009
---

# Integration test — pack/unpack round-trip in full pipeline

## Parent Initiative

[[FIDIUS-I-0009]]

## Objective

Extend the existing full pipeline test (`fidius-cli/tests/full_pipeline.rs`) to exercise the pack/unpack flow: scaffold → write manifest → sign → pack → unpack to new location → build from unpacked → load → call.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Full pipeline test includes `fidius package pack` after signing
- [ ] Packed `.fid` file exists and is non-empty
- [ ] `fidius package unpack` extracts to a fresh directory
- [ ] Build and load succeeds from the unpacked directory
- [ ] Test verifies unsigned pack emits warning to stderr
- [ ] All existing tests continue to pass

## Implementation Notes

### Files to modify
- `fidius-cli/tests/full_pipeline.rs` — extend existing test or add new test function
- `fidius-cli/tests/cli.rs` — add focused pack/unpack CLI tests

### Dependencies
- Blocked by FIDIUS-T-0057, FIDIUS-T-0059

## Status Updates

- 2026-03-31: Extended full pipeline test with 4 new steps (8-11): pack signed package, unpack to fresh dir, verify unpacked signature, and test unsigned warning on stderr. All 12 steps pass. Full test suite (`angreal test`) passes with no regressions.