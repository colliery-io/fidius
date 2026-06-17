---
id: polyglot-wasm-demonstration-a
level: initiative
title: "Polyglot WASM demonstration — a JavaScript guest alongside Rust + Python"
short_code: "FIDIUS-I-0025"
created_at: 2026-06-17T17:06:10.596692+00:00
updated_at: 2026-06-17T17:07:37.870829+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
initiative_id: polyglot-wasm-demonstration-a
---

# Polyglot WASM demonstration — a JavaScript guest alongside Rust + Python Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

The Component Model's selling point is language independence, but the repo only demonstrated Rust + Python guests. To make the polyglot claim concrete, add a third language implementing the **same `greeter` WIT**, loaded through the identical host path.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A JavaScript `greeter` guest (jco/ComponentizeJS) building to a component, loaded + round-tripped through `PluginHost::load_wasm` against the same descriptor as Rust/Python.
- CI builds it; a how-to documents it (incl. the real sandbox gotcha).

**Non-Goals:**
- Go/TinyGo, C#/.NET, C guests — the toolchains aren't in this environment to verify; the Component Model makes them work identically and the docs note that.

## Detailed Design **[REQUIRED]**

`tests/wasm-fixtures/greeter-js/` (greeter.js + build.sh). jco type mapping: s64/u64 → BigInt, list<u8> → Uint8Array, result<T,_> → return/throw; exported interface = ESM named export `greeter`; kebab → lowerCamelCase. Built with `jco componentize --disable http fetch-event` (StarlingMonkey imports `wasi:http` by default, which the deny-all sandbox rejects). Host test `polyglot_js_guest_behaves_identically` loads it via `load_wasm(&GREETER_DESC)` and asserts greet/add/echo match the Rust+Python guests. CI installs Node + builds the fixture. How-to: `docs/how-to/wasm-javascript-plugin.md`.

## Alternatives Considered **[REQUIRED]**

- **Wire `wasi:http` into the linker** to satisfy jco's default engine — rejected; it would grant HTTP against the deny-all sandbox model. Disabling the feature at build time is correct.
- **Add Go/C# too** — deferred; toolchains absent here, can't verify. Documented as supported instead of shipping unverified fixtures.

## Implementation Plan **[REQUIRED]**

1. JS fixture + host test + CI + how-to (FIDIUS-T-0121) — done.

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