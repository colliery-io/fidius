---
id: harden-the-coarse-network-sockets
level: task
title: "Harden the coarse `network`/`sockets` grant for untrusted WASM guests"
short_code: "FIDIUS-T-0143"
created_at: 2026-06-19T20:04:46.875508+00:00
updated_at: 2026-06-19T20:33:31.212216+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Harden the coarse `network`/`sockets` grant for untrusted WASM guests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

> Surfaced during the WASM security audit in [[FIDIUS-I-0027]]; referenced from `docs/explanation/wasm-capabilities.md`.

## Objective **[REQUIRED]**

Constrain (or clearly fence) the coarse `network`/`sockets` capability, which today does `WasiCtxBuilder::inherit_network()` — raw `wasi:sockets` TCP/UDP to **anywhere** + DNS, with **no** per-host filter and **no** broker hook. It is a strict superset of the `wasi:http` egress ([[FIDIUS-I-0027]]) with none of its controls: an untrusted connector granted `sockets` bypasses every HTTP-layer policy (it can open a raw socket straight to `169.254.169.254`).

**Options to decide:** (a) add a `socket_addr_check`-based allow-list/denylist hook (wasmtime exposes a per-connect callback) so raw sockets get an egress policy like HTTP; and/or (b) formally mark `sockets` **trusted-tier-only** and steer untrusted REST connectors to `http`. At minimum, document the blast radius (the docs already point here).

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

### Type
- [x] Tech Debt — security

### Priority
- [x] P2 — the un-brokered egress escape hatch; lower urgency than `env` only because most connectors should use `http` instead.

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

### 2026-06-19 — fixed ✅ (baseline SSRF floor; full per-host policy deferred)
Chose the proportionate fix: a **safety floor**, not a new policy mechanism (keeps the "mechanism not policy" line — per-host egress policy is `http`'s job).
- `build_wasi_ctx` `network`/`sockets` arm now installs `WasiCtxBuilder::socket_addr_check(...)` rejecting `is_blocked_ip(addr.ip())` — loopback / link-local (incl. metadata `169.254.169.254`) / RFC-1918 / unique-local / unspecified / broadcast. Checked on the **resolved** `SocketAddr`, so DNS-rebind-to-internal is caught.
- `is_blocked_ip` helper (v4 + v6, incl. v4-mapped) + `ssrf_tests` unit tests (blocks internal/metadata, allows public).
- Docs: `wasm-capabilities.md` raw-sockets section documents the floor + steers REST connectors to `http`.
- **Verified**: ssrf_tests 2/2; wasm_executor 23/23; lint clean. Committed `947d391`.
- **NOT done (deferred):** a full per-host allow-list / `SocketPolicy` embedder hook for raw sockets, and a "trusted-tier-only" formalization. The floor blocks the worst SSRF targets but still allows any public host; revisit if untrusted raw-socket connectors need tighter control. Most connectors should use `http` instead.