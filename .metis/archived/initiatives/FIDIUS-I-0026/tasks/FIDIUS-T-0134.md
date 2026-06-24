---
id: ws-6-wasm-streaming-per-item-perf
level: task
title: "WS.6 — WASM streaming per-item perf: cached InstancePre reuse + bench extension (NFR-001)"
short_code: "FIDIUS-T-0134"
created_at: 2026-06-19T03:28:12.961271+00:00
updated_at: 2026-06-19T16:34:02.161040+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# WS.6 — WASM streaming per-item perf: cached InstancePre reuse + bench extension (NFR-001)

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 2 · perf. Depends on [[FIDIUS-T-0131]], [[FIDIUS-T-0132]].

## Objective **[REQUIRED]**

Confirm WASM streaming meets NFR-001 — per-item overhead stays competitive — by ensuring the cached `InstancePre` is reused (one instantiate per *stream*, not per *item*) and extending the existing backend benchmarks to per-item streaming throughput/latency.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Verify the streaming path does **one `instantiate()` per stream**, not per item (the per-item cost is just a `next()` call on the live resource); document the cost model.
- [ ] Extend `crates/fidius-host/benches/backends.rs` (FIDIUS-T-0119/0120) with a per-item streaming benchmark: amortized per-item latency for WASM (and Python, for comparison) vs the unary baseline.
- [ ] Record results in the perf doc; flag any per-item regression vs unary that would violate NFR-001, with a follow-up if needed.
- [ ] No correctness regressions; bench is `required-features = ["wasm"]`-gated like the existing one.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Reuse the existing criterion harness in `benches/backends.rs`. Add a "stream N items" bench that opens one stream and pulls N, reporting per-item time.
- Compare WASM resource-`next()` per-item against the Python generator per-item and the unary call baseline; the interesting number is marginal per-item cost (instantiation amortized away).

### Dependencies
- Depends on [[FIDIUS-T-0131]] (host streaming) + [[FIDIUS-T-0132]] (a real component to bench). Last Phase-2 task.

### Risk Considerations
- Benchmarks are environment-sensitive; report relative (vs unary baseline) not absolute, and keep the bench out of the default test gate.

## Status Updates **[REQUIRED]**

### 2026-06-19 — WS.6 complete ✅ — NFR-001 confirmed
- **Cost model verified (one `instantiate()` per stream):** `WasmComponentExecutor::call_streaming` calls `self.instantiate()` **once** at stream start; each pulled item is just a `next()` on the live resource handle (the `InstancePre` is cached at load, so per-stream only pays a fresh `Store` + `instantiate_pre`). Documented in the bench module + `wasm.rs`.
- **Bench added:** a `streaming`-gated `stream_drain` group in `crates/fidius-host/benches/backends.rs` — loads the ticker component and drains streams of 100 / 1k / 10k items via `call_streaming`, reporting **per-item** cost (`Throughput::Elements`). Gated so `cargo bench --features wasm` (no streaming) still compiles. Run: `cargo bench -p fidius-host --features wasm,streaming --bench backends -- stream_drain`.
- **Results** (this machine, JIT, indicative):
  | N | total drain | per-item | thrpt |
  |---|---|---|---|
  | 100 | 208 µs | ~2.08 µs | ~480 K/s |
  | 1,000 | 1.45 ms | ~1.45 µs | ~688 K/s |
  | 10,000 | 14.0 ms | **~1.40 µs** | ~712 K/s |
  Per-item **converges to ~1.4 µs/item (~712 K items/s)** as N grows — exactly the model: a fixed per-stream `instantiate()` (~40–70 µs, amortized away) + a low marginal `next()` (dynamic `Val` call + channel hop + `Value` conversion). At N=100 the fixed cost still shows (2.08 µs/item); by N=10k it's negligible.
- **NFR-001 met:** marginal per-item overhead is competitive (single-digit µs), no per-item regression vs the unary call baseline (the existing `add`/`echo` groups). Numbers are env-sensitive (reported relative/indicative, not a gate), bench kept out of the default test path.
- **Cleanup:** removed the two `post_return` calls I'd added in the streaming path (deprecated no-op in wasmtime 45). One pre-existing `post_return` deprecation remains in the unary `call`/`call_raw` path — out of scope, noted.
- **Verified**: bench builds + runs green (`--bench backends -- stream_drain`); streaming E2E still green after the cleanup.

### 2026-06-19 — extended to a cross-backend/language comparison
Per the AC ("per-item latency for WASM **and Python** vs baseline"), the `stream_drain` group now benches **three** streaming guests side by side on a shared x-axis (criterion compares them directly): `wasm_rust` (`#[plugin_impl]` component resource), `wasm_js` (jco JS guest, same WIT, skipped if unbuilt), and `python` (CPython generator via PyO3, `#[cfg(feature="python")]`). Run: `cargo bench -p fidius-host --features wasm,streaming,python --bench backends -- stream_drain`.

**Results (this machine, JIT; per-item marginal cost, converged at N=10k):**
| Backend / language | per-item | thrpt | per-stream instantiate |
|---|---|---|---|
| **WASM / Rust** | **~1.3 µs** | ~764 K/s | ~40–70 µs |
| **Python** (generator) | ~1.8 µs | ~558 K/s | ~0 (in-process, no component instantiate) |
| **WASM / JavaScript** (jco) | ~134 µs | ~7.5 K/s | **~7 ms** (StarlingMonkey JS-engine init) |

Raw totals (whole N-item drain): wasm_rust 100→203µs / 1k→1.36ms / 10k→13.1ms; python 100→193µs / 1k→1.78ms / 10k→17.9ms; wasm_js 100→7.37ms / 1k→135ms / 10k→1.34s.

**Reading it:**
- **wasm_rust and python are both single-digit µs/item** — native-class marginal cost in/out of the sandbox; Python's GIL-thread + JSON bridge is cheap per item, and it pays *no* per-stream component instantiate (in-process generator).
- The per-stream fixed cost shows only at small N: wasm_rust N=100 = 2.03 µs/item (the ~40–70µs `instantiate()` amortized over 100) → 1.31 µs by N=10k; Python is near-flat.
- **wasm_js ≈ 100× slower per item (~134 µs) + a ~7 ms engine-init per stream** — the cost of authoring connectors in JavaScript (every `next()` crosses a full JS-engine boundary). A genuine "convenience vs throughput" signal for the polyglot tier: fine for low-volume/control-plane connectors, not high-throughput record streams.
- **NFR-001 holds** for the first-class backends (Rust/Python). JS is a documented tradeoff, not a regression. Verified: bench compiles+runs with `--features wasm,streaming,python`; default suite still green.

### 2026-06-19 — expanded to a 6-way comparison (cdylib + 4 wasm languages + embedded python)
Added cdylib (CS.1 [[FIDIUS-T-0136]]) plus **Python-WASM** (componentize-py) and **C-WASM** (wasi-sdk) streaming guests, to "match the languages we have unary examples for." Go is the only gap — TinyGo isn't installed here (no brew) and its component-model resource export is immature. New fixtures: `tests/wasm-fixtures/ticker-py`, `tests/wasm-fixtures/ticker-c`; polyglot E2E in `wasm_streaming_e2e.rs` (Rust/JS/Python/C, 10 tests green).

**6-way `stream_drain` (converged per-item @ N=10k):**
| Backend / language | per-item | thrpt |
|---|---|---|
| wasm / Rust | **1.31 µs** | ~762 K/s |
| wasm / C (wasi-sdk, ~18 KB) | 1.36 µs | ~734 K/s |
| cdylib (native FFI, JSON items) | 1.68 µs | ~595 K/s |
| python (embedded PyO3) | 1.81 µs | ~554 K/s |
| wasm / Python (componentize-py) | 2.41 µs | ~415 K/s |
| wasm / JavaScript (jco) | ~133 µs | ~7.5 K/s |

**Corrected finding (supersedes the earlier 3-way note):** it is **not** "interpreted-in-wasm is slow." **Python-in-WASM is native-class (~2.4 µs/item) — ~55× faster than JS-in-WASM.** The ~133 µs outlier is *specifically* the StarlingMonkey JS engine's per-`next()` boundary (jco), not a general interpreted-language tax. Everything except JS — compiled (Rust/C), native FFI (cdylib), and *both* Python paths (embedded + componentize-py) — sits in the single-digit-µs native class. The per-stream instantiate cost still shows at small N (wasm_py ~1.5 ms CPython init, wasm_js ~7 ms engine init) but amortizes away by N=10k. NFR-001 holds for every backend except the JS-engine path, which remains a documented per-item tradeoff (fine for low-volume/control-plane connectors).