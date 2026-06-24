<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# How to Work on Fidius

This guide covers the development tools and workflows for contributing to fidius.

## Prerequisites

- Rust toolchain (stable)
- [angreal](https://github.com/angreal/angreal) — task runner
- [pre-commit](https://pre-commit.com/) — git hook manager

Working on the WASM execution backend additionally needs the component build
toolchain — see [How to Set Up the WASM Component Toolchain](wasm-component-toolchain.md).
Not required for the cdylib or Python backends.

## Install Pre-commit Hooks

```bash
pre-commit install
```

This enables three hooks that run on every commit:

- **license-header** — checks all `.rs` files have the Apache 2.0 copyright header
- **rustfmt** — checks code formatting
- **clippy** — runs clippy with `-D warnings`

## Angreal Tasks

| Command | Purpose |
|---------|---------|
| `angreal build` | `cargo build --workspace` |
| `angreal build --release` | Release build |
| `angreal test` | `cargo test --workspace` |
| `angreal test --release` | `cargo test --workspace --release` |
| `angreal check` | `cargo check --workspace` + `cargo clippy --workspace` |
| `angreal lint` | `cargo fmt --all --check` + clippy |
| `angreal coverage` | Test coverage (per-crate summary + HTML + lcov) via cargo-llvm-cov |
| `angreal license-header` | Add Apache 2.0 headers to all `.rs` files |
| `angreal license-header --check` | Check headers without modifying files |

The wire format is **bincode in both debug and release builds** (0.1.0+).
There's no longer a profile-specific wire path to regression-test, so
`angreal test` alone is sufficient for CI; `--release` is only useful when
you want optimized builds in the loop.

## Measuring Test Coverage

Coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
(source-based LLVM instrumentation). Install it once:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

Then run:

```bash
angreal coverage         # workspace defaults + streaming (native surface)
angreal coverage --open  # also open the HTML report in a browser
angreal coverage --wasm  # best-effort: also instrument the wasm feature (see note)
```

This prints a per-crate summary table and writes two artifacts under
`target/coverage/` (git-ignored):

- `target/coverage/html/index.html` — browsable HTML report
- `target/coverage/lcov.info` — lcov for editors/CI tooling

The default surface is the workspace defaults plus `streaming`; `python` is left
off to match `angreal test`.

> **The `wasm` feature is not instrumented by default.** Its tests build
> `wasm32-wasip2` component fixtures *at test time*, and those sub-builds inherit
> cargo-llvm-cov's `-C instrument-coverage` flags — which the wasm target rejects
> (instrument-coverage isn't supported there), so the run fails. The wasm path's
> correctness is covered by the non-instrumented `wasm` CI job and `angreal test`;
> instrumenting it cleanly is a follow-on (it would need the fixture sub-builds to
> drop the coverage `RUSTFLAGS`). `--wasm` attempts it anyway, best-effort.

> **Report-only.** Coverage is a *map* of what the tests exercise, not a gate —
> `angreal coverage` never fails on a threshold. The same measurement runs in CI
> (report-only, native surface, published to the job summary). See initiative
> **FIDIUS-I-0033**.

## Fuzzing

Fidius fuzzes its untrusted-input boundaries with
[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). The harness is
a standalone, nightly-only crate at `crates/fidius-host/fuzz/` (detached from the
main workspace), with these targets:

| Target | Surface |
|--------|---------|
| `wire_value` | bincode `Value` decode + decode→encode→decode round-trip |
| `frame_read` | framed streaming-wire decode (length-prefix bounds) + round-trip |
| `manifest_validate` | `PackageManifest` parse + `validate_runtime` |
| `fid_extract` | `.fid` (tar+bzip2) safe extraction — `unpack_fid` |

Install and run locally (needs a nightly toolchain):

```bash
cargo install cargo-fuzz
cd crates/fidius-host

cargo fuzz list                       # show targets
cargo fuzz run wire_value             # fuzz until you Ctrl-C (a long campaign)
cargo fuzz run fid_extract -- -max_total_time=60   # time-boxed (CI does this)
```

Committed **seed corpora** live under `crates/fidius-host/fuzz/corpus/<target>/`
and seed every run. Build artifacts and any crash findings (`fuzz/target/`,
`fuzz/artifacts/`) are git-ignored.

- **Nightly CI fuzz** (`fuzz` job in `.github/workflows/nightly.yml`): runs each
  target for 120s on a nightly schedule + release (`v*`) tags + `workflow_dispatch`
  — *not* per-PR (fuzz is too expensive for the fast gate). Report-only: a crasher is
  surfaced in the job summary + uploaded as an artifact but does **not** gate
  (FIDIUS-I-0033 posture; flip by removing `continue-on-error`).
- **Longer campaigns**: run a target with no time limit locally, or on a schedule.
  Minimize a grown corpus before committing with
  `cargo fuzz cmin <target>`. Reproduce a CI crasher with
  `cargo fuzz run <target> fuzz/artifacts/<target>/<crash-file>`.

## The Test Plugin

The `tests/test-plugin-smoke/` directory contains a Calculator plugin used by integration tests. It is excluded from the workspace (`Cargo.toml` `exclude` field) and built by tests via `cargo build --manifest-path`.

To build it manually:

```bash
cd tests/test-plugin-smoke && cargo build
```

## Regenerating API Docs

API reference docs are generated by [plissken](https://github.com/colliery-io/plissken) from doc comments in source code:

```bash
plissken render -v
```

This writes to `docs/api/`. Re-run after changing doc comments.

## Project Layout

```
fidius-core/       Shared types (both host and plugin depend on this)
fidius-macro/      Proc macros (#[plugin_interface], #[plugin_impl])
fidius-host/       Host-side loading, validation, calling
fidius-cli/        CLI binary (fidius)
fidius-test/       Testing helpers (dylib_fixture, signing fixtures)
fidius/            Facade crate re-exporting core + macro
tests/             Test fixtures (test-plugin-smoke)
docs/              Documentation (tutorials, how-to, reference, explanation, api)
.angreal/          Angreal task definitions
```
