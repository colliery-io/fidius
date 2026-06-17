---
id: pg-3-c-wasi-sdk-polyglot-greeter
level: task
title: "PG.3 — C (wasi-sdk) polyglot greeter guest; .NET assessed"
short_code: "FIDIUS-T-0123"
created_at: 2026-06-17T19:56:00.444959+00:00
updated_at: 2026-06-17T19:56:24.170039+00:00
parent: FIDIUS-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0025
---

# PG.3 — C (wasi-sdk) polyglot greeter guest; .NET assessed

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0025]]

## Objective **[REQUIRED]**

A C (wasi-sdk) `greeter` guest implementing the same WIT, loaded through the same host path — the leanest guest. Also assess .NET/C#.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `tests/wasm-fixtures/greeter-c/` (greeter_impl.c + build.sh) builds a valid ~18 KB component via wit-bindgen + wasi-sdk clang (`wasm32-wasip2`, links straight to a component — no adapter).
- [x] `polyglot_c_guest_behaves_identically` loads via `load_wasm(&GREETER_DESC)`; greet/add/echo == the other guests.
- [x] CI builds it (wit-bindgen-cli + wasi-sdk-25); how-to `docs/how-to/wasm-c-plugin.md`.
- [x] .NET/C# assessed (see status).

## Status Updates **[REQUIRED]**

**2026-06-17 — C COMPLETE.** wit-bindgen 0.44 + wasi-sdk @33 clang. `wasm32-wasip2` + `-mexec-model=reactor` links directly to a component via wasm-component-ld (no preview1 adapter). ~18 KB — smallest of all guests. Verified green with Python/JS/Go (4 polyglot tests pass).

**.NET/C# — assessed, NOT shipped (user opted to skip).** componentize-dotnet (0.8.0-preview, .NET 10) works via NativeAOT-LLVM; the C# compiled cleanly against the generated `IGreeterExports` (impl class `GreeterExportsImpl`). Blocker: Microsoft publishes the NativeAOT-LLVM **host** compiler only for linux-x64/win-x64 — `runtime.osx-arm64.Microsoft.DotNet.ILCompiler.LLVM` does not exist, so it can't build on this macOS arm64 box. It would build in CI/linux or a linux container; the user chose to skip rather than go CI-only. Note for future: MacPorts `dotnet-cli` is only the host muxer (no SDK); use the official installer + the dotnet-experimental NuGet feed.

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

*To be added during implementation*