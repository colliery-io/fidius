---
id: cdylib-streaming-arena-style-item
level: task
title: "cdylib streaming: arena-style item buffer (kill per-item alloc + free_buffer FFI)"
short_code: "FIDIUS-T-0138"
created_at: 2026-06-19T17:56:57.847559+00:00
updated_at: 2026-06-19T18:11:43.485209+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# cdylib streaming: arena-style item buffer (kill per-item alloc + free_buffer FFI)

> Follow-up from [[FIDIUS-T-0137]]. The real cdylib-streaming perf fix (T-0137's typed-bincode change was a ~4% win; this is the ~0.2 µs/item structural cost).

## Objective **[REQUIRED]**

Eliminate cdylib streaming's **per-item heap alloc + `free_buffer` FFI round-trip** so cdylib reclaims the fastest-per-item position. Today each `next()` item is a guest-`malloc`'d `Box<[u8]>` the host frees through a *second* FFI crossing — 2 crossings + alloc + free per item. wasm does 1 call and manages memory internally; that's the ~0.2 µs/item (≈14%) gap measured in `stream_drain`.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [x] Tech Debt — performance/ABI

### Priority
- [x] P3 — cdylib streaming works at ~1.6 µs/item; this is to win the per-item crown for high-throughput in-process streams.

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] cdylib streaming `next()` writes the item into a **host-provided reusable buffer** instead of `malloc`ing a fresh `Box<[u8]>` per item — no per-item alloc, no `free_buffer` crossing. (Mirror the unary **Arena** buffer strategy: `next(handle, buf_ptr, buf_cap, *out_len)`, grow-and-retry once on too-small.)
- [ ] Measured per-item cdylib ≤ wasm/Rust in `stream_drain` (the goal T-0137 missed), or documented why not.
- [ ] Drop-cancel still runs `drop_fn`; error/end framing preserved; no regression to Python/WASM streaming.

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### 2026-06-19 — implemented + zero-alloc; perf win UNPROVEN on this hardware ⚠️
**Implemented (correct, all green):**
- `status`: `STATUS_STREAM_END = -5` (distinct end signal vs a real zero-byte item).
- `fidius-guest`: `FidiusStreamHandle.next` → `(handle, buf_ptr, buf_cap, *out_len)`. Added `wire::serialized_size` + `wire::serialize_into` (alloc-free). `StreamState<T>` holds the **item value** (not bytes) pending → happy path serializes **directly into the host buffer, zero `Vec` alloc**; `BUFFER_TOO_SMALL` re-serializes the same item.
- Macro `next`/init/drop use `StreamState`. Host pump owns **one reusable buffer**, grows+retries once, decodes from it — **no per-item `free_buffer` crossing**.
- cdylib streaming E2E 3/3; default 50/0; python/wasm streaming unaffected.

**Result — could NOT demonstrate the win.** Across 4 `stream_drain` variants (JSON → bincode → arena-vec → arena-zerocopy) cdylib stayed *above* wasm/Rust **every run**, and the optimizations didn't move its relative position. The bench is too noisy: **wasm/Rust @10k swung 1.06–1.41 µs across runs with zero wasm changes** — environment drift (thermal/load from back-to-back benches) exceeds the sub-0.3 µs delta. cdylib @10k wandered 1.61–1.81 µs, no trend.

**AC:** #1 (reusable buffer, no per-item alloc/free) ✅; #3 (drop-cancel + no regression) ✅; **#2 (cdylib ≤ wasm) — documented-why-not:** unmeasurable on this hardware; the change does objectively *fewer ops/item*, so kept on engineering merit. The persistent gap across all variants means the real floor is elsewhere (per-`next` `catch_unwind` + FFI dispatch + the shared tokio-channel hop), not item encoding/alloc.

**Decision:** keep it (strictly fewer per-item ops, clean ABI). A real answer needs a controlled bench env (pinned cores, no throttle) + profiling — not worth it unless cdylib streaming throughput becomes a hard requirement. Stop optimizing against a noisy machine.