---
id: callerror-semantics-distinct
level: initiative
title: "CallError Semantics — Distinct InvalidMethodIndex Variant"
short_code: "FIDIUS-I-0015"
created_at: 2026-04-17T13:23:35.083878+00:00
updated_at: 2026-04-17T17:40:06.584348+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XS
initiative_id: callerror-semantics-distinct
---

# CallError Semantics — Distinct InvalidMethodIndex Variant Initiative

## Context

`PluginHandle::call_method` in `fidius-host/src/handle.rs:114-117` does a bounds check against the vtable method count:

```rust
if index >= self.method_count as usize {
    return Err(CallError::NotImplemented { bit: index as u32 });
}
```

This returns `CallError::NotImplemented { bit }`, but **that variant semantically means "the capability bit for this optional method is not set"** — it's the error the generated typed client will return when `has_capability(bit) == false` for an optional method. The two situations are different:

- `NotImplemented { bit: 3 }`: the plugin declared support for optional method 3 via its interface but chose not to implement it — valid plugin, method absent
- Out-of-range index: caller passed an invalid index (programming error), nothing to do with capability bits at all

Mixing them means host code that catches "this plugin doesn't support this optional method" will also silently catch "I typo'd the vtable index," and a malformed index of, say, 999 gets reported as `NotImplemented { bit: 999 }` — which is nonsense since capability bits cap at 63.

This is small, isolated, and worth fixing before the typed Client (FIDIUS-I-0012) lands — the Client will rely on `NotImplemented { bit }` meaning exactly one thing.

## Goals & Non-Goals

**Goals:**
- Add `CallError::InvalidMethodIndex { index, count }` variant
- Return it from the bounds check instead of `NotImplemented`
- Keep `NotImplemented { bit }` reserved for capability-bit-not-set case
- Provide a clear error message that tells callers what went wrong

**Non-Goals:**
- Change FFI-level status codes
- Restructure the rest of `CallError`
- Change the public signature of `call_method`

## Detailed Design

### New variant

```rust
// fidius-host/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    // ... existing variants ...

    #[error("invalid method index {index} (plugin has {count} method(s))")]
    InvalidMethodIndex { index: usize, count: u32 },

    // ... existing NotImplemented variant stays as-is ...
}
```

### Updated bounds check

```rust
// fidius-host/src/handle.rs:114-117
if index >= self.method_count as usize {
    return Err(CallError::InvalidMethodIndex {
        index,
        count: self.method_count,
    });
}
```

### Test coverage

```rust
#[test]
fn out_of_bounds_vtable_index_returns_invalid_method_index() {
    // existing test at fidius-host/tests/integration.rs:212-233
    // currently asserts CallError::NotImplemented — update to InvalidMethodIndex
}
```

## Alternatives Considered

- **Rename `NotImplemented` to `OptionalMethodAbsent`.** Clearer but wider change. `NotImplemented` is fine once `InvalidMethodIndex` exists; the name ambiguity only shows up when both cases funnel through it.
- **Use `CallError::Panic(String)` for the bounds violation.** No — it's a caller error, not a plugin failure. Panic is reserved for the FFI `catch_unwind` path.
- **Panic directly on bounds violation.** The current design correctly returns a Result; keep that.

## Implementation Plan

1. Add `InvalidMethodIndex { index: usize, count: u32 }` variant to `CallError` in `fidius-host/src/error.rs`
2. Update `PluginHandle::call_method` in `fidius-host/src/handle.rs:115` to return the new variant
3. Update the existing out-of-bounds test in `fidius-host/tests/integration.rs:212-233` to assert the new variant
4. Grep callers for any pattern-match on `CallError::NotImplemented` that implicitly relies on the old (incorrect) semantics — update them
5. Doc-comment on `NotImplemented` clarifies it is only for capability-bit-not-set case

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- `handle.call_method(999, &input)` on a 5-method plugin returns `CallError::InvalidMethodIndex { index: 999, count: 5 }`
- Test in `fidius-host/tests/integration.rs` asserts `InvalidMethodIndex`, not `NotImplemented`
- `cargo doc` on `CallError::NotImplemented` explicitly says "capability bit not set" — no ambiguity with out-of-range indices
- No production code path returns `NotImplemented { bit: index }` where `bit` exceeds 63