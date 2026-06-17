---
id: backend-performance-benchmarks-vs
level: initiative
title: "Backend performance — benchmarks vs microservice transports (+ WASM per-call optimization)"
short_code: "FIDIUS-I-0024"
created_at: 2026-06-17T15:24:40.867118+00:00
updated_at: 2026-06-17T16:39:52.654896+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: backend-performance-benchmarks-vs
---

# Backend performance — benchmarks vs microservice transports (+ WASM per-call optimization) Initiative

## Context **[REQUIRED]**

Adopters push back that a plugin architecture is "too slow / too costly vs microservices." We had no data to answer with. This initiative adds a reproducible cross-backend benchmark and an honest perf doc, and (motivated by what it found) tracks a WASM per-call optimization.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A criterion benchmark comparing cdylib / wasm-JIT / wasm-AOT against microservice-style transports (localhost TCP, Unix-socket IPC, HTTP/1.1) on the same ops.
- A perf explanation doc with the numbers + an honest reading.
- Act on the finding (WASM per-call latency) with an instance-reuse optimization (separate task).

**Non-Goals:**
- A general load/soak/concurrency suite (single-call latency + throughput only).
- Tuning the network baselines (they're deliberate *lower bounds* for a microservice).

## Detailed Design **[REQUIRED]**

`crates/fidius-host/benches/backends.rs` (criterion, `--features wasm`): `add(i64,i64)` + `echo(bytes)` at 64 B/4 KiB/256 KiB, across cdylib (in-process FFI), wasm JIT/AOT, and three persistent-connection transports (raw TCP, UDS, HTTP/1.1). Results → `docs/explanation/performance.md`.

### Findings (2026-06-17, dev machine, medians)
- **cdylib wins by 2–3 orders of magnitude**: `add` ~42 ns vs ~9–24 µs for any local transport; 256 KiB echo ~15 µs vs ~100 µs–6.9 ms. The "too slow vs microservices" claim is false for native plugins.
- **WASM (current impl) is NOT faster than a local microservice**: `add` ~86–124 µs (vs ~10–25 µs HTTP/UDS), 256 KiB echo ~6.7 ms. Root cause: the executor builds a **fresh `Store` + re-instantiates per call** and **copies the payload through `Value↔Val`**. This is the likely source of the "too slow" feedback — and it's a fixable artifact, not fundamental.
- **Cost/footprint**: a plugin has no standing process; a microservice does. Plugins win footprint + ops regardless of latency.

### Recommended optimization (separate task)
Cache `InstancePre` / reuse a pooled `Store` for trusted long-lived plugins (removes the per-call instantiation floor), and write `#[wire(raw)]` bulk bytes straight into guest memory (removes the large-payload copy). Expected to recover most of the WASM gap. Keep fresh-instance-per-call as the safe default; reuse is opt-in for trusted plugins.

## Alternatives Considered **[REQUIRED]**

- **Hand-rolled timing loop** instead of criterion — rejected; criterion's warmup/outlier handling is more credible to skeptics.
- **Heavy HTTP stack (hyper/reqwest) for the baseline** — rejected; a hand-rolled keep-alive HTTP/1.1 + raw TCP + UDS span the realistic transport cost as clear lower bounds without dep weight. A real microservice is strictly slower.
- **Optimize WASM before measuring** — rejected; measure first, let the data justify the optimization (it does).

## Implementation Plan **[REQUIRED]**

1. Benchmark + perf doc (FIDIUS-T-0119) — **done**.
2. WASM instance-reuse + raw-bytes fast path (future task) — optional, motivated by the findings; pursue if WASM per-call latency matters for a target workload.

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete if not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, etc.}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system should do - use unique identifiers}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system should behave}
  - NFR-001: {Performance requirement}
  - NFR-002: {Security requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

{Delete if not user-facing}

### Use Case 1: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

### Use Case 2: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete if not technically complex}

### Overview
{High-level architectural approach}

### Component Diagrams
{Describe or link to component diagrams}

### Class Diagrams
{Describe or link to class diagrams - for OOP systems}

### Sequence Diagrams
{Describe or link to sequence diagrams - for interaction flows}

### Deployment Diagrams
{Describe or link to deployment diagrams - for infrastructure}

## Detailed Design **[REQUIRED]**

{Technical approach and implementation details}

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

{Delete if no UI components}

### User Interface Mockups
{Describe or link to UI mockups}

### User Flows
{Describe key user interaction flows}

### Design System Integration
{How this fits with existing design patterns}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete if covered by separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}
- **Data Management**: {Test data strategy}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}
- **Performance Testing**: {Load and stress testing}

### Test Selection
{Criteria for determining what to test}

### Bug Tracking
{How defects will be managed and prioritized}

## Alternatives Considered **[REQUIRED]**

{Alternative approaches and why they were rejected}

## Implementation Plan **[REQUIRED]**

{Phases and timeline for execution}