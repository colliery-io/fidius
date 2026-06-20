---
id: client-streaming-stream-lt-t-gt-in
level: initiative
title: "Client-streaming — Stream&lt;T&gt; in argument position (host produces, plugin pulls)"
short_code: "FIDIUS-I-0030"
created_at: 2026-06-20T14:44:31.831323+00:00
updated_at: 2026-06-20T14:47:19.406612+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/design"


exit_criteria_met: false
estimated_complexity: M
initiative_id: client-streaming-stream-lt-t-gt-in
---

# Client-streaming — Stream&lt;T&gt; in argument position (host produces, plugin pulls) Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

> **Status: specced, not scheduled.** Design of record: [[FIDIUS-A-0007]]. Implementation plan to execute when an adopter needs plugin-controlled consumption; stays in design until then.

## Context **[REQUIRED]**

The dual of shipped server-streaming ([[FIDIUS-I-0026]]): `Stream<T>` in **argument**
position — the host produces, the plugin pulls/consumes. Refused today (compile-fail,
`ir.rs:519`). Lets a writer/sink plugin control its own consumption (batch, look-ahead,
backpressure) rather than the host pushing one unary call per item. The pull-channel
ABI design is fixed in [[FIDIUS-A-0007]]; this is the build plan.

**Goals:** allow `fn m(&self, s: fidius::Stream<T>) -> R`; a host-provided pull channel
per backend (cdylib callback table, WASM imported `fidius-stream-next`, Python host-backed
generator); a distinct `<stream` interface-hash marker; host `call_client_streaming`;
drop=cancel + inverted backpressure; E2E on all 3 backends; flip the compile-fail.

**Non-Goals:** bidirectional (separate, later — two concurrent pumps + deadlock-avoidance,
own ADR); changing server-streaming or the `Vec<T>` chunked-unary arg.

**UC — bulk-load writer:** `fn load(&self, rows: Stream<Row>) -> LoadReport` pulls rows in
batches of 1000, flushes each, paces the host to its flush rate (plugin-controlled
backpressure the host-pump can't express).

**Plan (decompose after sign-off; parked):**
- CS2.1 — macro/IR: accept `Stream<T>` in arg position; `<stream` hash marker.
- CS2.2 — cdylib pull-callback ABI + generated `Iterator` wrapper + host producer; E2E.
- CS2.3 — WASM imported `fidius-stream-next` + executor producer wiring; E2E.
- CS2.4 — Python host-backed generator; E2E.
- CS2.5 — host `call_client_streaming`; flip the compile-fail; docs.

Alternatives (see [[FIDIUS-A-0007]]): host-pump-only (no plugin control), `Vec<T>` arg
(unbounded memory), full bidirectional (highest cost). Pull-channel chosen.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- {Primary objective 1}
- {Primary objective 2}

**Non-Goals:**
- {What this initiative will not address}

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