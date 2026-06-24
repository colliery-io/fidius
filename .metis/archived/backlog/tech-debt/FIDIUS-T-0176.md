---
id: facade-completeness-audit-mirror
level: task
title: "Facade-completeness audit — mirror fidius-host host-app surface (+ egress/http_types)"
short_code: "FIDIUS-T-0176"
created_at: 2026-06-21T02:41:11.558275+00:00
updated_at: 2026-06-21T02:41:56.355609+00:00
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

# Facade-completeness audit — mirror fidius-host host-app surface (+ egress/http_types)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

Make the `fidius` facade a complete boundary for **host applications** so a downstream
(incl. white-label re-export crates like weir's `weir_connector::fidius`) can depend on
`fidius` alone and drop the direct `fidius-host` dependency. Surfaced by a downstream that
couldn't name `EgressPolicy`/`EgressDenied` through the facade; the root cause was the
facade lacking a `wasm` feature + mirroring only part of fidius-host's public surface.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

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

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

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

**DONE — shipped across 0.5.1 + 0.5.2.**
- **0.5.1 (d9d03bc)**: lifted `EgressPolicy`/`EgressDenied` to the `fidius-host` crate root;
  added a facade `wasm` feature (`= ["host", "fidius-host/wasm"]`) re-exporting them. This is
  the primitive that lets a consumer enable `fidius/wasm` instead of depping fidius-host.
- **0.5.2 (8acb102)**: full facade-completeness audit. The facade mirrored only 8 of
  fidius-host's 14 crate-root public types, and `PluginHostBuilder` wasn't even at the
  fidius-host crate root. Added — host: `PluginHostBuilder`, `LoadedLibrary`, `LoadedPlugin`,
  `PluginExecutor`, `PluginRuntimeKind`; wasm: re-export the `http` crate as `http_types` so
  `EgressPolicy::authorize`'s `&mut http::request::Parts` is nameable without a separate
  `http` dep. Compile-test guards (`facade_host_surface` / `facade_wasm_surface`) name every
  re-export so a future drop fails the build.

**Result:** a host app (or white-label boundary) can use `fidius = { features = ["wasm",
"streaming"] }` and drop the direct `fidius-host` dependency entirely — one namespace, no
exceptions. Additive only; ABI stayed 500. Both patches published to crates.io.

**Key insight for next time:** a facade can't re-export a *type* keyed to a dependency's
feature via unification (only *methods* on already-re-exported types ride along) — so the
facade needs its own feature mirroring the backend's, and must explicitly mirror the backend's
public type surface. Worth a periodic facade-vs-fidius-host diff before releases.