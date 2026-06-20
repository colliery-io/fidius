---
id: production-grade-wasm-connectors
level: initiative
title: "Production-grade WASM connectors — typed-record streaming, rich WIT types, HTTP timeouts"
short_code: "FIDIUS-I-0031"
created_at: 2026-06-20T15:20:31.174078+00:00
updated_at: 2026-06-20T15:20:31.174078+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: production-grade-wasm-connectors
---

# Production-grade WASM connectors — typed-record streaming, rich WIT types, HTTP timeouts Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

The WASM connector story works but stops short of production fidelity on fronts surfaced
in the post-0.5 deferred-work review. Real connectors need to (1) stream **typed
records**, not just primitives/JSON strings; (2) express **rich types** (maps, tuples,
nesting) in their interfaces; (3) make outbound HTTP with **timeouts** so a slow upstream
can't hang a call; and (4) surface **structured errors** from Python plugins. This closes
that gap so a connector is "shippable," not just "works." The deny-all sandbox,
capability model, and the streaming/egress primitives are already in place — this is
**expressiveness + robustness** on top of them.

## Goals & Non-Goals **[REQUIRED]**

**Goals:** typed-record items over WASM streaming; `HashMap`/tuple/nested types in WIT;
guest-HTTP request timeouts; Python structured-error round-trip; a worked example.

**Non-Goals:** client/bidirectional streaming ([[FIDIUS-I-0030]]); new capabilities (the
surface is complete — env/fs/http/network/stdio); changes to non-Rust guests beyond what
the WIT type work implies.

## Detailed Design **[REQUIRED]**

**Proposed task breakdown (pending sign-off):**

- **PC.1 — Rich WIT type mapping**: extend `fidius-wit` so `HashMap<K,V>`/`BTreeMap` →
  `list<tuple<k,v>>`, tuples → `tuple<...>`, and nesting (`Vec<Record>`, `Vec<Option<T>>`)
  map cleanly; the reject-list shrinks. Unit tests + the type-mapping doc. *(Foundation.)*
- **PC.2 — Typed-record stream items over WASM** (headline): lift the
  `generate_wasm_adapter` "streaming + `#[derive(WitType)]`" rejection (`impl_macro.rs:435`)
  so the streaming WIT resource's `next()` carries a user record type via the
  emit_wit/build.rs path. New fixture + E2E (`Stream<Record>` over WASM). *Depends on PC.1.*
- **PC.3 — Guest HTTP `RequestOptions` + timeout**: add a timeout (room for connect/
  first-byte) to `fidius_guest::http::{get,post,send}`, wired to `wasi:http` outgoing
  request-options. Mock-server E2E: a slow upstream times out cleanly, doesn't hang.
- **PC.4 — Python structured `PluginError` round-trip**: carry `code`/`message`/`details`
  from a Python plugin's `PluginError` to the host `CallError::Plugin`. E2E.
- **PC.5 — Docs + a production-connector example** tying typed-record streaming + rich
  types + http-timeout together (extends `examples/` + the connector docs).

**Sequencing:** PC.1 → PC.2 (records lean on rich types); PC.3 and PC.4 are independent
(parallel-able); PC.5 last. An ADR is raised only if PC.2 hits a real design fork
(resource-with-next vs a native `stream<record>`); current expectation is it extends the
established WitType + streaming-resource patterns — no new ABI.

## Alternatives Considered **[REQUIRED]**

- **Keep the `Stream<String>`/JSON workaround for records** — status quo; loses type
  safety and double-encodes. Rejected.
- **Ship rich types (PC.1) without record-streaming (PC.2)** — leaves the headline gap.
  Rejected; PC.1 exists to enable PC.2.
- **Fold HTTP timeouts / Python errors elsewhere** — they're small but thematically part
  of "production connectors"; kept together for one coherent arc.

## Implementation Plan **[REQUIRED]**

Decompose into PC.1–PC.5 (above) on sign-off, then execute via `/metis-ralph` per task in
sequence order. Each task carries its own tests; the initiative is done when a connector
can stream typed records with rich types and time-boxed HTTP, with a runnable example.

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