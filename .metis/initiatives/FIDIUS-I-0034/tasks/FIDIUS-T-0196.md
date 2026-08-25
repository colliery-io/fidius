---
id: hostname-egress-e2e-suite-name
level: task
title: "Hostname-egress E2E suite (name allow-list, pin correctness, rotation, authorize_dns) + docs + release"
short_code: "FIDIUS-T-0196"
created_at: 2026-08-18T12:34:27.739784+00:00
updated_at: 2026-08-25T02:02:03.143052+00:00
parent: FIDIUS-I-0034
blocked_by: [FIDIUS-T-0195]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0034
---

# Hostname-egress E2E suite (name allow-list, pin correctness, rotation, authorize_dns) + docs + release

## Parent Initiative

[[FIDIUS-I-0034]]

## Objective

Prove every acceptance criterion of FIDIUS-I-0034 end-to-end against the production path (real `tcp-echo` guest through `WasmComponentExecutor` / `PluginHost`), replace the spike test with the real suite, finish the docs, and cut the release. Closes the initiative.

## Test Plan (maps 1:1 to the initiative's acceptance criteria)

New e2e file (e.g. `crates/fidius-host/tests/hostname_egress_e2e.rs`), superseding and **removing** `hostname_pin_spike.rs`. Uses the `tcp-echo` fixture and, where DNS control is needed, the resolver seam from FIDIUS-T-0195 (inject `name -> Vec<IpAddr>`; no real DNS in tests beyond a `localhost` smoke case).

1. **TcpTarget shape**: hostname dial → policy sees `TcpTarget { host: Some(name), addr }`; IP-literal dial → `host: None`. (Production-path version of the spike tests; keep one true-`localhost` case as a no-injection smoke test.)
2. **Default delegation**: an embedder policy overriding only `authorize_tcp` behaves byte-identically through the new path (e2e flavor; the unit test lives in FIDIUS-T-0194).
3. **Name-keyed allow-list e2e** (the weir `HostAllowList` follow-up pattern, as a reference policy in the test): allow `db.internal` → connect succeeds; unlisted name → denied; **a second name resolving to the same IP → denied unless itself listed** (pin correctness, not IP fallthrough — needs injected resolver mapping both names to 127.0.0.1).
4. **Resident-lifetime / rotation** (configured instance on the persistent store, per FIDIUS-A-0006 fixtures): resolve name→IP_A, rotate injected resolver to IP_B, re-resolve → new pin authorizes IP_B; **stale pin no longer authorizes IP_A** (replace-on-re-resolve proven).
5. **authorize_dns**: denying policy → guest lookup fails (`PermanentResolverFailure` surface: hostname dial errors, empty echo), nothing pinned, no connect attempted; default policy → lookups unchanged.
6. **Regression**: full existing suite green, `tcp_egress_e2e.rs` untouched.

## Acceptance Criteria

## Acceptance Criteria

- [x] All six test groups above implemented and green (`angreal test`, wasm feature); spike file removed.
- [x] `authorize_tcp`/`authorize_tcp_target`/`authorize_dns` doc blocks final (resolve-and-pin described as fidius's mechanism; `host: None`, TOCTOU narrowing, pin lifetime documented); user-facing docs updated (`docs/explanation/wasm-capabilities.md` hostname allow-list section; plissken API docs regenerated).
- [x] CHANGELOG entry (0.5.8); version bump: all workspace crates + internal dep pins 0.5.7 → 0.5.8, Cargo.lock consistent (python package tracks separately, left at 0.5.5 per existing convention).
- [x] `angreal lint`, `angreal license-header`, `angreal check` clean.
- [x] FIDIUS-I-0034 acceptance criteria all checked off in the initiative doc; initiative transitioned to completed.

## Implementation Notes

The rotation test (group 4) is the only one needing a configured/resident instance — reuse the existing configured-wasm fixture pattern (`macro-configured` / FIDIUS-A-0006 tests) or give `tcp-echo` a configured variant if simpler. The injected resolver must be settable per-executor (constructor or builder hook marked `#[doc(hidden)]`) — do not leak it into the stable public API surface.

Embedder follow-up (out of scope here, tracked in the initiative): weir's `HostAllowList::authorize_tcp_target` at weir `crates/weir-runtime/src/lib.rs:246` once this releases.

Dependencies: FIDIUS-T-0195.

## Status Updates

- 2026-08-18: Created from FIDIUS-I-0034 decomposition (design phase complete, spike green).
- 2026-08-24: Complete. `tests/hostname_egress_e2e.rs` covers all six groups, 8/8 green: TcpTarget shape (hostname → `Some(name)`, literal → `None`), legacy-policy delegation e2e, name-keyed allow-list with injected resolver (`unlisted_name_denied`; `same_ip_second_name_denied_unless_listed` — the pin-correctness killer: same IP authorized under db.internal seconds earlier, still denied for evil.internal), pin attribution (`literal_dial_to_pinned_ip_is_attributed_to_the_name` — documents the TOCTOU-narrowing semantics), rotation (`rotation_replaces_pin_and_stale_ip_loses_authority`), and both authorize_dns polarities. One deviation from the plan: instead of a configured/resident-instance fixture, the rotation test drives sequential dials in ONE store via a new `connect-seq` method added to the tcp-echo fixture (WIT + guest impl) — the store is the unit of pin lifetime either way, so this proves the same semantics with far less machinery. Spike file removed. Docs: `wasm-capabilities.md` gained the `authorize_tcp_target` hostname allow-list section (old "DNS-rebinding is the embedder's residual" text replaced); plissken API docs regenerated. Release: CHANGELOG 0.5.8 entry (patch, per 0.5.x convention — additive, ABI unchanged); all crate versions + internal pins bumped 0.5.7→0.5.8. Full battery: `angreal test` 89 suites 0 failed, `angreal check`/`lint`/`license-header` clean.