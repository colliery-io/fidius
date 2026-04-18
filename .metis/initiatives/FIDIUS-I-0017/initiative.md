---
id: fidius-test-harness-in-process
level: initiative
title: "fidius-test Harness — In-Process + Dylib Test Infrastructure for Downstream Users"
short_code: "FIDIUS-I-0017"
created_at: 2026-04-17T13:40:45.387506+00:00
updated_at: 2026-04-18T00:47:54.576122+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: fidius-test-harness-in-process
---

# fidius-test Harness — In-Process + Dylib Test Infrastructure for Downstream Users Initiative

## Context

Fidius ships zero testing infrastructure for downstream users. The five crates (`fidius-core`, `fidius-macro`, `fidius-host`, `fidius-cli`, `fidius`) have extensive internal tests, but nothing reusable is exported. The scaffolds produced by `fidius init-plugin` and `fidius init-interface` generate `lib.rs` and `Cargo.toml` with no `#[cfg(test)]` block, no example test, and no guidance. `docs/tutorials/` has tutorials for authoring, signing, and packaging but nothing on testing.

**What internal tests do that downstream users can't:**

1. **Build a plugin from source inside a test.** `build_test_plugin()` is copy-pasted across `fidius-host/tests/integration.rs:24-52`, `fidius-host/tests/e2e.rs:24-52`, `fidius-cli/tests/cli.rs:27-51`, `fidius-host/tests/package_e2e.rs`. The code is ~30 lines of `Command::new("cargo").arg("build").arg("--manifest-path")...` plus platform-specific dylib-extension handling. Every downstream integration test has to reinvent this.

2. **Invoke a shim in-process without dylib loading.** `fidius-macro/tests/impl_basic.rs:64-96` demonstrates the pattern — serialize input tuple with `wire::serialize`, cast `desc.vtable` to `*const Trait_VTable`, call the raw function pointer, deserialize output, call `free_buffer`. This is ~30 lines of unsafe per method. The framework knows how to do it; nothing exposes it as a helper.

3. **Sign a fixture dylib.** `fidius-host/tests/e2e.rs:65-71` hardcodes `SigningKey::from_bytes(&[seed; 32])` and manually writes a `.sig` file. No helper.

**The cost to downstream users:**

A plugin author's first test is either:
- (a) 30s+ dev loop: write plugin, run `cargo build --release`, write a separate host harness that dlopens the output, run that harness — rediscovering `build_test_plugin` from scratch, or
- (b) Learn the framework internals well enough to instantiate the generated `__fidius_Trait::Trait_VTable` and invoke raw function pointers — essentially the framework-level unsafe dance.

A host author testing business logic that consumes plugins has no in-memory fake — every test needs a real compiled dylib on disk.

## Goals & Non-Goals

**Goals:**
- A new `fidius-test` crate with public, documented helpers for the most common test patterns
- In-process invocation: call a plugin method on an impl without compiling to dylib
- Dylib fixtures: build + cache a plugin's cdylib from a source path, reusable across tests
- Signing fixtures: deterministic keypair + sign helpers
- Scaffold updates: `fidius init-plugin` generates a test module using the in-process helper; `init-host` (from FIDIUS-I-0012) generates a test using the dylib fixture
- New `fidius test <plugin-dir>` CLI: build + load + invoke each method with a zero-value input, report pass/fail per method
- New `docs/how-to/test-plugins.md` tutorial

**Non-Goals:**
- Mocking arbitrary traits at the host layer (would compete with `mockall`; users can reach for that directly if needed)
- Replacing `trybuild` for compile-fail tests (trybuild is already the right tool; we just point users to it)
- Snapshot testing of wire format bytes (cross-cuts; separate initiative if wanted)
- Property-based testing framework (users pull in `proptest` themselves)
- Test parallelism / sandboxing (users compose with standard `cargo test`)

## Detailed Design

### New crate: `fidius-test`

Workspace member at `fidius-test/`. Depends on `fidius-host` (full API including signing), `tempfile` (fixtures), `ed25519-dalek` (keys). Declared as a `dev-dependency`-friendly crate — users add it under `[dev-dependencies]`.

```toml
# fidius-test/Cargo.toml
[package]
name = "fidius-test"
version = "0.1.0"
edition = "2021"
description = "Testing helpers for fidius plugin authors and hosts"
license = "Apache-2.0"

[dependencies]
fidius-host = { path = "../fidius-host", version = "0.0.5" }
fidius-core = { path = "../fidius-core", version = "0.0.5" }
ed25519-dalek = { workspace = true }
tempfile = "3"
```

### Helper 1: `in_process!` — call without dlopen

Plugin authors want this:

```rust
use fidius_test::in_process;

#[plugin_impl(Calculator)]
impl Calculator for BasicCalculator { /* ... */ }

#[test]
fn add_works() {
    let out: i64 = in_process!(BasicCalculator, add_direct, 3i64, 7i64).unwrap();
    assert_eq!(out, 10);
}

#[test]
fn version_works() {
    let v: String = in_process!(BasicCalculator, version).unwrap();
    assert_eq!(v, "1.0.0");
}
```

The macro expands to the same dance `fidius-macro/tests/impl_basic.rs:64-96` does, but generated from type + method name:

```rust
macro_rules! in_process {
    ($ty:ty, $method:ident $(, $arg:expr)* $(,)?) => {{
        // 1. Get the registry — inventory has already been populated by $ty's plugin_impl
        let reg = ::fidius::registry::get_registry();

        // 2. Find descriptor by plugin_name == stringify!($ty)
        let desc = $crate::internal::find_descriptor(reg, stringify!($ty))
            .ok_or("plugin not in registry — did you #[plugin_impl] it?")?;

        // 3. Find method index by looking up the trait's METHOD_* const
        //    This requires knowing the trait at compile time — user passes it via a
        //    separate macro form, or we resolve via the generated companion module
        //    (design detail below)

        // 4. Serialize input tuple
        let input = ($($arg,)*);
        let input_bytes = ::fidius::wire::serialize(&input)?;

        // 5. Call vtable fn via desc.vtable pointer
        let fn_ptr = /* resolve fn_ptr at method_index from vtable */;
        let mut out_ptr = std::ptr::null_mut();
        let mut out_len = 0u32;
        let status = unsafe { fn_ptr(input_bytes.as_ptr(), input_bytes.len() as u32, &mut out_ptr, &mut out_len) };

        // 6. Check status, deserialize output, free buffer
        $crate::internal::finish_call(status, out_ptr, out_len, desc.free_buffer)
    }};
}
```

Design note: resolving the method index requires the trait name. Simplest API: require the user to pass it:

```rust
in_process!(BasicCalculator: Calculator, add_direct, 3i64, 7i64)
```

That gives us `__fidius_Calculator::METHOD_ADD_DIRECT` unambiguously.

Alternative: generate an in-process client from `plugin_interface` (similar to the typed Client in FIDIUS-I-0012 but calling directly rather than through PluginHandle):

```rust
let client = BasicCalculatorInProcess::new();  // generated struct
let out = client.add_direct(3, 7)?;            // typed
```

This is the cleaner API. It can be emitted under `#[cfg(feature = "test")]` in the interface crate, same mechanism as the `host` feature. **Recommendation: ship the macro form first (simpler), and if adopted, add the generated in-process client as a follow-up within this initiative.**

### Helper 2: `dylib_fixture` — build + cache a plugin

```rust
use fidius_test::dylib_fixture;

#[test]
fn host_loads_plugin() {
    let fixture = dylib_fixture(env!("CARGO_MANIFEST_DIR"))
        .with_release(false)  // default: debug
        .build();

    let host = PluginHost::builder()
        .search_path(fixture.dir())
        .build()
        .unwrap();

    let plugins = host.discover().unwrap();
    assert!(!plugins.is_empty());
}
```

Implementation: shell out to `cargo build --manifest-path ...` (same as `build_test_plugin` today), but cache the built dylib across tests in the same process using a `OnceLock<HashMap<PathBuf, DylibFixture>>`. Within a single `cargo test` invocation, the plugin is built exactly once even when used by N tests.

Also provides `.signed_with(&key)`:

```rust
let (sk, pk) = fixture_keypair();
let fixture = dylib_fixture("path/to/plugin")
    .signed_with(&sk)
    .build();

let host = PluginHost::builder()
    .search_path(fixture.dir())
    .trusted_keys(&[pk])
    .require_signature(true)
    .build()?;
```

### Helper 3: signing fixtures

```rust
/// Deterministic keypair derived from a seed byte. Use 1u8 for the default test key.
pub fn fixture_keypair_with_seed(seed: u8) -> (SigningKey, VerifyingKey);

/// Convenience: fixture_keypair_with_seed(1).
pub fn fixture_keypair() -> (SigningKey, VerifyingKey);

/// Sign a dylib file in place, writing the .sig next to it.
pub fn sign_dylib(path: &Path, key: &SigningKey) -> std::io::Result<()>;
```

These are 5-line functions but every test writes them today.

### Helper 4: `fidius test <plugin-dir>` CLI

New CLI subcommand. Flow:

```
$ fidius test ./my-plugin
Building my-plugin...
Loading BasicCalculator...
  add(AddInput { a: 0, b: 0 }) -> AddOutput { result: 0 } ✓
  add_direct(0, 0) -> 0 ✓
  version() -> "1.0.0" ✓
  multiply (optional, capability bit 0) -> skipped (not configured)

3/3 methods passed (1 optional skipped)
```

For each method, invoke with a `Default`-valued input (requires input types to derive Default, or the test skips and notes it). This is a smoke test — not a behavioral assertion, just "does the FFI round-trip work end to end." Catches broken builds, mangled descriptors, missing registry export, serialization panics.

Implementation in `fidius-cli/src/commands.rs`: calls `fidius-host::load_library`, iterates plugins and methods, invokes each with an empty tuple input (plugin-side will fail to deserialize if the method needs non-default inputs — reported as a non-fatal warning, not a failure).

### Scaffold updates

**`fidius init-plugin`** — append to generated `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use fidius_test::in_process;

    #[test]
    fn example_method_in_process() {
        // TODO: replace with your method
        // let out: ReturnType = in_process!(YourStruct: YourTrait, your_method, args).unwrap();
        // assert_eq!(out, expected);
    }
}
```

Add `fidius-test = "0.1"` to generated `[dev-dependencies]`.

**`fidius init-host`** (new, from FIDIUS-I-0012) — generate a test using `dylib_fixture` + the typed Client.

### Documentation

New `docs/how-to/test-plugins.md` covers:
- Writing in-process tests with `in_process!`
- Writing integration tests with `dylib_fixture`
- Testing signed-plugin flows with `fixture_keypair` + `signed_with`
- Running `fidius test` as a smoke loop
- When to use each layer (unit → in-process → dylib → full host)

### Files to create / modify

**New files:**
- `fidius-test/Cargo.toml`
- `fidius-test/src/lib.rs` — public API (in_process macro, dylib_fixture, signing fixtures)
- `fidius-test/src/internal.rs` — helpers used by the macro (find_descriptor, finish_call)
- `fidius-test/src/dylib.rs` — DylibFixture + builder + cache
- `fidius-test/src/signing.rs` — fixture_keypair, sign_dylib
- `fidius-test/tests/smoke.rs` — self-test that the harness itself works
- `docs/how-to/test-plugins.md` — tutorial

**Modified files:**
- `Cargo.toml` workspace members — add `fidius-test`
- `fidius-cli/src/main.rs` — add `Test` subcommand
- `fidius-cli/src/commands.rs` — `cmd_test(dir: &Path)` implementation; update `init_plugin` and `init_host` scaffolds
- `fidius-host/tests/integration.rs`, `e2e.rs`, `package_e2e.rs`, `fidius-cli/tests/cli.rs` — migrate from copy-pasted `build_test_plugin` to `fidius_test::dylib_fixture` (dogfooding)
- `docs/tutorials/your-first-plugin.md` — add closing section pointing to testing tutorial
- `mkdocs.yml` — add the new how-to page

### Interaction with other initiatives

- **FIDIUS-I-0012 (typed Client)**: once the Client lands, `in_process!` can become trivially typed (`client.method(args)` rather than index-based). Initially ship `in_process!` as macro form; add generated in-process Client after I-0012.
- **FIDIUS-I-0013 (Box<[u8]>)**: no interaction — test harness uses public APIs.
- **FIDIUS-I-0014 (Arena)**: test harness needs to handle Arena strategy in dylib_fixture's host builder configuration. Minor.
- **FIDIUS-I-0015 (InvalidMethodIndex)**: `fidius test` CLI should surface this error cleanly.
- **FIDIUS-I-0016 (bincode-only)**: simplifies `in_process!` — no wire-format dispatch needed.

Recommend landing **after I-0015 (trivial) and I-0016 (simplifies wire)** but either of **I-0012 (typed client)** or I-0017 can go first; they converge naturally.

## Alternatives Considered

- **Build testing on top of `mockall` / `faux`:** those mock the Rust trait, not the FFI path — users would test their trait impl without exercising fidius at all. Misses the whole point. The FFI round-trip is what's worth testing.
- **Leave it as docs-only ("here's the pattern, copy it"):** fails the readability bar. Users don't want to copy 30 lines of unsafe into every test.
- **Ship helpers inside `fidius-host` under a `test` feature:** possible but pollutes the host crate's public surface and couples test infra to host-loading. A separate crate is cleaner.
- **Generate test code from the `#[plugin_impl]` macro directly:** tempting but increases macro surface, and tests belong in the test module, not mixed with production code. Keep macro focused.
- **Skip `fidius test` CLI, expose only library helpers:** library helpers require the user to write tests. The CLI is the one-liner that works before they've written any tests — highest first-contact value.

## Implementation Plan

1. Create `fidius-test` crate skeleton with Cargo.toml, lib.rs, README
2. Implement `fidius_test::signing` — `fixture_keypair`, `sign_dylib` — smallest piece, proves the pattern
3. Implement `fidius_test::dylib` — `DylibFixture` + `dylib_fixture()` builder, with cargo build, caching, signing option
4. Implement `in_process!` macro — descriptor lookup, vtable invocation, wire roundtrip
5. Add `fidius-test/tests/smoke.rs` — exercises all three helpers against a fixture plugin
6. Migrate `fidius-host/tests/integration.rs`, `e2e.rs`, `package_e2e.rs`, `cli.rs` to use `fidius_test::dylib_fixture` — delete four copies of `build_test_plugin`
7. Add `fidius test <plugin-dir>` CLI subcommand
8. Update `init-plugin` scaffold to emit a `tests` module + `fidius-test` dev-dep
9. Update `init-host` scaffold (from FIDIUS-I-0012) similarly
10. Write `docs/how-to/test-plugins.md` with the four use cases: unit, in-process, dylib, CLI smoke
11. Update `docs/tutorials/your-first-plugin.md` with a closing "now test it" section
12. Publish `fidius-test 0.1.0` alongside next fidius release

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- `fidius-test` crate exists and is a workspace member
- `in_process!(MyStruct: MyTrait, method, args)` compiles and calls the method via the generated shim without dlopen
- `dylib_fixture(path).build()` produces a cacheable fixture; second invocation in the same test binary does not re-run `cargo build`
- `dylib_fixture(path).signed_with(&key).build()` produces a signed fixture that `PluginHost::require_signature(true)` accepts
- `fixture_keypair()` returns a deterministic (SigningKey, VerifyingKey) pair
- `fidius test ./tests/test-plugin-smoke` reports 3/3 required methods pass (optional multiply skipped) without errors
- `fidius init-plugin` scaffold includes `#[cfg(test)] mod tests` and `fidius-test` dev-dep
- `docs/how-to/test-plugins.md` exists, covers all four layers (unit, in-process, dylib, CLI smoke), and is linked from `mkdocs.yml`
- All four internal test files (`fidius-host/tests/integration.rs`, `e2e.rs`, `package_e2e.rs`, `fidius-cli/tests/cli.rs`) no longer contain `build_test_plugin()` — they use `fidius_test::dylib_fixture` instead
- `fidius-test` docs include a crate-level module doc with a five-line quickstart for each helper