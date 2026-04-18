---
id: smoke-test-compile-and-load-a-real
level: task
title: "Smoke test — compile and load a real cdylib plugin"
short_code: "FIDIUS-T-0012"
created_at: 2026-03-29T00:53:38.360267+00:00
updated_at: 2026-04-17T13:17:13.085218+00:00
parent: FIDIUS-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0002
---

# Smoke test — compile and load a real cdylib plugin

## Parent Initiative

[[FIDIUS-I-0002]]

## Objective

Build a minimal end-to-end proof: define a trait with `#[plugin_interface]`, implement it with `#[plugin_impl]`, compile to a cdylib, then use `libloading` to dlopen it and verify the `FIDIUS_PLUGIN_REGISTRY` is correct. This is not the full fidius-host integration (that's FIDIUS-I-0003/I-0004) — just a raw dlsym smoke test to prove the macro output is a valid, loadable plugin.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A test crate (`tests/test-plugin-smoke/`) with `crate-type = ["cdylib"]` that defines a simple trait + impl using the fidius macros
- [ ] Trait has at least: one required method, one optional method
- [ ] Test binary uses `libloading` to dlopen the built cdylib
- [ ] Test reads `FIDIUS_PLUGIN_REGISTRY` via dlsym, verifies: magic bytes, registry_version, plugin_count >= 1
- [ ] Test reads the descriptor: verifies interface_hash matches expected value, wire_format matches build profile, buffer_strategy == 1 (PluginAllocated)
- [ ] Test calls a vtable function pointer directly (raw FFI, no fidius-host) and verifies correct output
- [ ] Test passes under `cargo test`

## Implementation Notes

### Technical Approach

Structure:
```
tests/
├── test-plugin-smoke/
│   ├── Cargo.toml      # cdylib, depends on fides
│   └── src/lib.rs      # trait + impl
└── smoke_test.rs       # integration test that builds + loads the cdylib
```

The integration test uses `cargo build` as a subprocess to compile the test plugin, then uses `libloading` to load and inspect it. This is a coarser test than trybuild — it proves the full pipeline works, not just compilation.

Alternative: use a build script to pre-compile the cdylib and just load it in the test. Simpler but couples test to build order.

### Dependencies
- All previous macro tasks (T-0006 through T-0010)

### Risk Considerations
- cdylib path varies by platform (.so/.dylib/.dll) — use `libloading`'s platform detection
- Test must build the cdylib before loading — need a build step or subprocess

## Status Updates

- **2026-03-29**: Full pipeline proven. test-plugin-smoke cdylib with Calculator trait (add + optional multiply). smoke_cdylib test: builds cdylib via subprocess, dlopen, dlsym("fidius_get_registry"), reads registry (magic, version, count), reads descriptor (abi_version, buffer_strategy, interface/plugin names), calls add(3,7) through vtable, verifies result=10, frees buffer. All passing.