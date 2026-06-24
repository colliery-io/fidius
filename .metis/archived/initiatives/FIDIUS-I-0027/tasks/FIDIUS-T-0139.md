---
id: e1-wasi-http-egress-mechanism
level: task
title: "E1 — wasi:http egress mechanism: capability + required embedder hook (two-key, fail-closed)"
short_code: "FIDIUS-T-0139"
created_at: 2026-06-19T19:26:08.674575+00:00
updated_at: 2026-06-19T19:40:11.770738+00:00
parent: FIDIUS-I-0027
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0027
---

# E1 — wasi:http egress mechanism: capability + required embedder hook (two-key, fail-closed)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0027]]

## Objective **[REQUIRED]**

Ship the **mechanism** for sandboxed WASM outbound HTTP: the `wasi:http` capability + a **required embedder-supplied egress hook**, wired fail-closed. fidius supplies **no policy** (allow-list/SSRF/credentials are the embedder's hook). Full design in [[FIDIUS-I-0027]].

**Design (locked):**
- New host API: `PluginHost`/`WasmComponentExecutor` accept an embedder egress hook. Proposed fidius-native trait (hides wasmtime types behind the `http` crate):
  ```rust
  pub trait EgressPolicy: Send + Sync {
      /// Called before EVERY outbound request. Inspect uri/method, mutate headers
      /// (inject creds), or return Err to deny (guest gets an HTTP error).
      fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied>;
  }
  ```
  fidius wraps this in a `WasiHttpHooks` impl: `send_request` splits the `http::Request` into parts, calls `authorize`, then `default_send_request` (or returns the deny error).
- **Two-key gating**: `add_to_linker_sync` (wasi:http) is wired **iff** the package declares the `http` capability **AND** the host supplied an `EgressPolicy`. Missing either -> imports absent -> guest fails closed.
- `KNOWN_CAPABILITIES` gains `"http"`; `HostState` gains `http_ctx: WasiHttpCtx` + the boxed policy; `impl WasiHttpView for HostState`.
- `wasmtime-wasi-http = "45"` (lockstep), gated on the `wasm` feature.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] `wasmtime-wasi-http` 45 added under the `wasm` feature; workspace builds.
- [ ] `EgressPolicy` trait (or equivalent closure adapter) in `fidius-host`, expressed in `http`-crate types — no wasmtime types leak into the public signature.
- [ ] A builder seam to supply the policy (e.g. `PluginHost::builder().egress(policy)` / `WasmComponentExecutor` carries an `Option<Arc<dyn EgressPolicy>>`).
- [ ] `"http"` recognized by `validate_capabilities` (unknown still fails loud).
- [ ] **Two-key wiring**: wasi:http linker imports added only when (`http` declared) AND (policy supplied); otherwise absent → guest fails closed.
- [ ] fidius's `WasiHttpHooks::send_request` calls `authorize` before `default_send_request`; deny → guest sees an error, no dispatch.
- [ ] Gating logic covered (no-capability and capability-but-no-policy both fail closed); full live E2E lives in [[FIDIUS-T-0140]].
- [ ] Additive: existing non-HTTP WASM plugins + deny-all default unchanged; `angreal lint` + default suite green.

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

### 2026-06-19 — mechanism implemented, compiles + green
- **Dep**: `wasmtime-wasi-http = "45"` + `http = "1"` under the `wasm` feature (default features give `default-send-request` + `p2`; pulls rustls/tokio-rustls for TLS egress). Researched the exact 45.0.2 API from source first.
- **`EgressPolicy` + `EgressDenied`** (public, in `executor::wasm`, re-exported): `fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied>`. Pure `http`-crate types — **no wasmtime types leak**.
- **`EgressHooks`** impl `WasiHttpHooks`: `send_request` splits the request, calls `authorize` (deny → `ErrorCode::HttpRequestDenied`), else `default_send_request`. `HostState` gained `http_ctx: WasiHttpCtx` + `hooks`; `impl WasiHttpView`.
- **Two-key gating** in `build()`: `add_only_http_to_linker_sync` wired iff `capabilities` contains `"http"` AND `egress.is_some()`. Else http imports absent → guest fails closed at `instantiate_pre`. `"http"` added to `KNOWN_CAPABILITIES` (no-op in `build_wasi_ctx`).
- **API**: `WasmComponentExecutor::from_component_bytes_with_egress(.., egress: Option<Arc<dyn EgressPolicy>>, ..)`; existing constructors pass `None` (unchanged). Executor carries the `egress` field (satisfies AC "builder seam — or executor carries it").
- **Verified**: `cargo build --features wasm` clean; `angreal lint` clean; existing `wasm_executor` suite **21/21**; default workspace **50/0** (egress code is wasm-gated).

**Deferred to [[FIDIUS-T-0140]] (needs the fixture):** the behavioral fail-closed tests (no-capability / capability-but-no-policy) require a wasm component that imports `wasi:http` to observe the unresolved-import failure. **Deferred (optional follow-on):** a `PluginHost::builder().egress()` convenience that threads the policy into `load_wasm` — E2 uses the executor constructor directly, so not needed for the mechanism.