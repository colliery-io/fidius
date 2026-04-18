---
id: fidius-test-crate-library-helpers
level: task
title: "fidius-test crate: library helpers (signing, dylib_fixture, in_process!)"
short_code: "FIDIUS-T-0072"
created_at: 2026-04-17T18:20:46.203123+00:00
updated_at: 2026-04-17T18:26:25.961454+00:00
parent: FIDIUS-I-0017
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0017
---

# fidius-test crate: library helpers (signing, dylib_fixture, in_process!)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0017]]

## Objective

Create the `fidius-test` crate, a workspace member providing the testing helpers downstream users need. Scoped to two primary capabilities for this task:

1. **Signing fixtures** — deterministic Ed25519 keypair helpers and a `sign_dylib` convenience that writes the `.sig` file next to a dylib (matches `fidius sign` naming).
2. **Dylib fixture** — `dylib_fixture(plugin_dir).build()` runs `cargo build --manifest-path ...` once per test-binary process (cached via `OnceLock<Mutex<HashMap>>`) and returns a `DylibFixture` with the output directory and dylib path. Supports `.signed_with(key)` for signed-plugin test flows.

**Scope reduction:** the `in_process!` macro from the initiative is deferred to a follow-up task. Analysis showed it requires either a new proc-macro entry point (for ident concatenation of companion module paths) or a library-less `PluginHandle` constructor — both add non-trivial design surface that's better handled after the core dylib fixture helpers are dogfooded. The dylib fixture alone eliminates the biggest pain point (four copies of `build_test_plugin()` in the existing test suite).

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

- [x] New `fidius-test` crate added as workspace member in root `Cargo.toml`
- [x] `fidius_test::signing::fixture_keypair()` returns a deterministic `(SigningKey, VerifyingKey)` pair (seed = 1)
- [x] `fidius_test::signing::fixture_keypair_with_seed(seed)` derives keypairs from arbitrary seed bytes (different seeds → different keys)
- [x] `fidius_test::signing::sign_dylib(path, key)` writes a verifiable detached signature at `{path}.sig`
- [x] `fidius_test::dylib_fixture(plugin_dir).build()` invokes `cargo build`, locates the cdylib artifact, and returns a `DylibFixture` exposing `.dir()` and `.dylib_path()`
- [x] Subsequent calls to `dylib_fixture(same_dir).build()` return cached fixtures without re-invoking cargo (verified via identity of returned paths)
- [x] `DylibFixtureBuilder::signed_with(&key)` signs the dylib on build
- [x] `DylibFixtureBuilder::with_release(bool)` overrides the debug/release profile choice
- [x] `fidius-test/tests/smoke.rs` exercises all helpers against `test-plugin-smoke` — 5/5 tests pass
- [x] `angreal lint` clean
- [ ] **(deferred)** `in_process!` macro — moved to a follow-up task for post-0.1.0 design work

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

- **2026-04-17**: Completed (reduced scope — `in_process!` deferred).

  **New files:**
  - `fidius-test/Cargo.toml` — workspace member, depends on fidius-core/host/macro, dev-dep on test-plugin-smoke (for self-test) and tempfile
  - `fidius-test/src/lib.rs` — crate root with rustdoc quickstart
  - `fidius-test/src/signing.rs` — `fixture_keypair`, `fixture_keypair_with_seed`, `sign_dylib`
  - `fidius-test/src/dylib.rs` — `DylibFixture`, `DylibFixtureBuilder`, `dylib_fixture()` with `OnceLock<Mutex<HashMap<CacheKey, DylibFixture>>>` process-wide cache
  - `fidius-test/tests/smoke.rs` — 5 tests exercising all helpers

  **Modified files:**
  - Root `Cargo.toml` — added `fidius-test` to workspace members

  **Design notes:**
  - Cache key is `(plugin_dir, release)` so debug+release builds of the same plugin are independently cached.
  - `DylibFixture` is `Clone` (cheap — two PathBufs) so the cache can hand out copies.
  - `signed_with` only runs on first (uncached) build — cached returns are unchanged. Documented in rustdoc. Tests that need re-signing call `fidius_test::signing::sign_dylib` directly on `fixture.dylib_path()`.
  - Release vs debug default follows the test binary's own profile via `cfg!(debug_assertions)` — matches the convention in the original `build_test_plugin()` helpers.

  **Scope deferral — `in_process!`:**
  Two designs were considered: (a) proc-macro in `fidius-macro` for ident concatenation (`__fidius_<Trait>::METHOD_<METHOD_UPPER>`) and (b) a library-less `PluginHandle` constructor that wraps a descriptor for in-process use. Both are non-trivial (a new proc-macro entry or a modification to `PluginHandle::_library: Arc<Library>` → `Option<Arc<Library>>`). Deferred to a follow-up task so T-0072 ships the high-value dylib_fixture path that already unblocks the internal test-suite cleanup (T-0073) and scaffolds (T-0074).

  **Validation:**
  - `cargo test -p fidius-test` — 5/5 smoke tests pass
  - `angreal lint` — clean
  - Full `angreal test` — all suites pass except a pre-existing flaky `e2e.rs` test (`unsigned_plugin_fails_when_signature_required`) that shows test-isolation issues under parallel execution; passes cleanly when run alone (`cargo test -p fidius-host --test e2e`). Not introduced by this change — documented in the T-0015 status update as a known intermittent.