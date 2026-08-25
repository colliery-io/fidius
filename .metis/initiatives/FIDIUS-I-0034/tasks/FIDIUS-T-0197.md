---
id: hostname-egress-close-out
level: task
title: "Hostname-egress close-out: configured-instance e2e, edge-case tests, how-to docs, backlog notes"
short_code: "FIDIUS-T-0197"
created_at: 2026-08-25T02:19:05.554668+00:00
updated_at: 2026-08-25T02:23:20.502798+00:00
parent: FIDIUS-I-0034
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0034
---

# Hostname-egress close-out: configured-instance e2e, edge-case tests, how-to docs, backlog notes

## Parent Initiative

[[FIDIUS-I-0034]]

## Objective

Close the gaps identified in the post-release review (2026-08-24): the one genuine functional gap (pins on the real configured/resident persistent store, across separate calls) plus a batch of cheap edge-case tests, two how-to doc updates, and backlog cross-references.

## Acceptance Criteria

## Acceptance Criteria

- [x] Configured-instance e2e: tcp-echo gains a no-op `fidius-configure` export; a test configures the executor (persistent store), then across SEPARATE `call_method` invocations proves pins persist and rotate — call 1 dials `db.internal` (→ v4, pinned), call 2 re-dials after resolver rotation (→ v6, pin replaced), call 3 dials the old v4 literal and is DENIED (stale pin gone across calls). *(`configured_instance_pins_persist_and_rotate_across_calls`)*
- [x] Case-insensitivity e2e: guest dials `DB.INTERNAL`, policy lists `db.internal` → allowed, policy sees the lowercased name. *(`mixed_case_dial_matches_lowercase_allow_list`)*
- [x] Unresolvable-name e2e: resolver has no entry → guest gets empty, nothing pinned, `authorize_tcp_target` never called. *(`unresolvable_name_fails_lookup_without_reaching_policy`)*
- [x] Multi-IP e2e: a name resolving to several IPs pins all of them; the guest's connect fallback across them succeeds under the name's authority. *(`multi_ip_resolution_pins_all_candidates` — listener only on the second candidate)*
- [x] Builder-path parity: `PluginHost::builder().egress_policy(name-keyed policy)` + `load_wasm` authorizes a hostname dial. *(`builder_path_hostname_dial_authorized_by_name`)*
- [x] `docs/how-to/host-application.md` egress section retitled "Brokered egress (WASM): HTTP and raw TCP" and covers `authorize_tcp_target` + `authorize_dns`; `docs/how-to/production-connector.md` points raw-TCP connectors at hostname allow-lists.
- [x] FIDIUS-T-0159 notes the two new wasmtime-lockstep touchpoints; FIDIUS-T-0193 notes the hostname dimension for the streaming-egress fixture.
- [x] Full battery green: `angreal test` (89 suites, 0 failed; hostname suite 13/13) / `check` / `lint` / `license-header`. CHANGELOG's e2e paragraph updated to cover the new tests.

## Status Updates

- 2026-08-24: Created from the post-release gap review; executing immediately.
- 2026-08-24: Complete. tcp-echo fixture gained `fidius-configure` (no-op, WIT + guest impl) enabling `configure()` onto the persistent store; hostname_egress_e2e grew from 8 to 13 tests (all listed above), notably proving pins persist and rotate across SEPARATE calls on a configured resident instance — the true weir long-lived-connector scenario. Both how-to docs updated; backlog cross-references written; CHANGELOG amended (same unreleased 0.5.8 entry). Full battery green.