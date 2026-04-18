---
id: dogfood-fidius-test-in-internal
level: task
title: "Dogfood fidius-test in internal test suites"
short_code: "FIDIUS-T-0073"
created_at: 2026-04-17T18:20:47.394466+00:00
updated_at: 2026-04-17T19:57:50.502876+00:00
parent: FIDIUS-I-0017
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0017
---

# Dogfood fidius-test in internal test suites

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0017]]

## Objective

Replace the copy-pasted `build_test_plugin()` helper across three test files with `fidius_test::dylib_fixture`. Replace hardcoded `SigningKey::from_bytes(&[N; 32])` + local `sign_dylib` helper in `e2e.rs` with `fidius_test::{fixture_keypair_with_seed, sign_dylib}`. Drops ~100 lines of duplicated scaffolding, and the shared `OnceLock` cache makes per-test-file builds run once instead of per-test-function.

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

- [x] `fidius-test` added as dev-dep to `fidius-host` and `fidius-cli`
- [x] `fidius-host/tests/integration.rs` uses `dylib_fixture` (no local `build_test_plugin`)
- [x] `fidius-host/tests/e2e.rs` uses `dylib_fixture` + `fixture_keypair_with_seed` + `sign_dylib` (no local helpers)
- [x] `fidius-cli/tests/cli.rs` uses `dylib_fixture` (no local `build_test_plugin`)
- [x] All tests pass; no regressions (full `angreal test` clean)
- [x] `angreal lint` clean
- [x] Measurable speedup: integration.rs tests went from ~3.5s to ~0.25s because the cargo build is shared across all 10 tests via the process-wide cache

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

## Status Updates

- **2026-04-17**: Completed.

  **Dev-deps added:**
  - `fidius-host/Cargo.toml` — `fidius-test = { path = "../fidius-test", version = "0.0.5" }`
  - `fidius-cli/Cargo.toml` — same

  **Files migrated:**

  1. `fidius-host/tests/integration.rs` — deleted ~30 lines of `build_test_plugin` fn + `Command::new("cargo")` boilerplate. Replaced with `plugin_source_dir()` + `plugin_dir() -> &'static Path` wrapping a `OnceLock<PathBuf>` of `dylib_fixture(...).build().dir().to_path_buf()`. `client()` helper and all 10 tests now reference `plugin_dir()` directly.

  2. `fidius-host/tests/e2e.rs` — complete rewrite. Removed `build_test_plugin`, local `sign_dylib`, and hardcoded `SigningKey::from_bytes(&[N; 32])` across 6 tests. Now uses `fixture_keypair_with_seed(N)` returning `(sk, pk)` tuples and `fidius_test::sign_dylib(&dylib_path(), &sk)`. File is ~60 lines shorter.

  3. `fidius-cli/tests/cli.rs` — deleted `build_test_plugin` fn, replaced with `plugin_dir()` helper mirroring integration.rs.

  **Scope note — smoke_cdylib.rs:** The `fidius-macro/tests/smoke_cdylib.rs` test has an inline `cargo build` pattern but migrating it would require adding `fidius-test` as dev-dep of `fidius-macro` (a proc-macro crate). That creates a dev-dep cycle (fidius-macro → fidius-test → fidius-macro). Cargo can resolve this but the cost/benefit is poor for a single test. Left as-is.

  **Scope note — full_pipeline.rs:** Uses `cargo build` only indirectly via the `fidius` CLI subcommands it invokes — it scaffolds fresh crates and builds them, doesn't reuse test-plugin-smoke. Not a migration target.

  **Observed perf win:**
  Before: integration.rs finished in ~3.5s (each of 10 tests ran its own `cargo build` subprocess, then cargo's own up-to-date check short-circuited work but still paid the subprocess cost).
  After: ~0.21s. The `OnceLock<HashMap>` cache in `fidius-test::dylib` ensures exactly one subprocess invocation per test binary. Similar speedups observed in e2e.rs and cli.rs.

  **Validation:**
  - `cargo test -p fidius-host --test integration` — 10/10 pass in 0.21s
  - `cargo test -p fidius-host --test e2e` — 6/6 pass in 0.25s
  - `cargo test -p fidius-cli --test cli` — 6/6 pass in 0.89s
  - Full `angreal test` — all 27 test-result groups pass; no flaky e2e this run (the shared build cache likely reduces parallel filesystem contention that caused the previous flake)
  - `angreal lint` clean