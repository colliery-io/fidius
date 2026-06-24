---
id: combined-egress-streaming-wasm
level: task
title: "Combined egress × streaming wasm fixture (a sandboxed streaming connector that does egress)"
short_code: "FIDIUS-T-0193"
created_at: 2026-06-23T23:13:31.465084+00:00
updated_at: 2026-06-23T23:13:31.465084+00:00
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

# Combined egress × streaming wasm fixture (a sandboxed streaming connector that does egress)

## Objective

The FIDIUS-I-0033 Phase 5 matrix review (T-0190) found the cross-product is otherwise
comprehensively covered, with **one genuine gap**: there is no fixture/test exercising
**egress and streaming together** — i.e. a sandboxed WASM connector that *streams*
results while making capability-gated outbound calls (`wasi:http` or `tcp`). The two
axes are each well-covered in isolation:

- streaming: `wasm_streaming_e2e`, `wasm_client_stream_e2e`, `wasm_bidi_stream_e2e`, …
- egress: `wasm_egress_e2e`, `macro_egress_e2e`, `tcp_egress_e2e`

…but never jointly. A real connector (e.g. a DB or API source streaming rows over a
policy-gated socket) is exactly that combination, so it's worth a dedicated fixture.

## Acceptance Criteria

- [ ] A wasm fixture that returns `Stream<T>` *and* performs gated egress within the
      stream production.
- [ ] An e2e test asserting the two-key egress gate still holds on the streaming path
      (allowed → items flow; denied → fails closed) under the embedder's policy.

## Notes

Filed from FIDIUS-I-0033 / T-0190. Needs a new component fixture (WIT + build), so it's
a bounded feature-test effort rather than a quick add — deferred out of the initiative
deliberately, not silently skipped.
