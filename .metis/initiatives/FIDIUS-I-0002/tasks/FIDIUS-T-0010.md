---
id: feature-gated-async-support
level: task
title: "Feature-gated async support"
short_code: "FIDIUS-T-0010"
created_at: 2026-03-29T00:53:36.142982+00:00
updated_at: 2026-03-29T01:11:57.650359+00:00
parent: FIDIUS-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0002
---

# Feature-gated async support

## Parent Initiative

[[FIDIUS-I-0002]]

## Objective

Add async method support to both macros behind a `features = ["async"]` feature flag. When the trait contains `async fn` methods and the feature is enabled, the `#[plugin_impl]` shims create a per-plugin lazy tokio runtime and call `runtime.block_on(instance.method(...))`. The FFI boundary stays synchronous.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fidius-macro/Cargo.toml` and `fidius-core/Cargo.toml` have an `async` feature flag
- [ ] `#[plugin_interface]` accepts `async fn` methods in the trait — vtable signature is the same (FFI stays sync)
- [ ] `#[plugin_impl]` generates a `static RUNTIME: LazyLock<tokio::Runtime>` when any method is async
- [ ] Async shims call `RUNTIME.block_on(INSTANCE.method(...))` inside the catch_unwind
- [ ] Without the `async` feature, `async fn` in the trait produces a clear `compile_error!`
- [ ] Sync methods in the same trait as async methods work normally (no runtime for sync shims)
- [ ] Compiles and works with `--features async`

## Implementation Notes

### Technical Approach

In the IR (T-0006), `MethodIR.is_async` is already tracked. In the shim codegen (T-0008), check `is_async`:

- If true and feature `async` is enabled: generate `RUNTIME.block_on(...)` wrapper
- If true and feature not enabled: `compile_error!("async methods require the 'async' feature")`
- If false: normal sync call

Runtime initialization:
```rust
static RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create fidius async runtime")
});
```

One runtime per dylib (shared across all plugin impls in that dylib).

### Dependencies
- FIDIUS-T-0008 (shim codegen must exist to extend)

## Status Updates

- **2026-03-29**: Implemented. fidius-core has `async` feature flag gating `async_runtime` module with `FIDES_RUNTIME: LazyLock<tokio::Runtime>`. Shim codegen detects `async fn` in impl block and generates `FIDES_RUNTIME.block_on(...)` call. Test passes: async method called through FFI vtable returns correct result. Note: compile_error for async-without-feature not yet implemented (deferred — the feature is always on in dev-deps for tests).