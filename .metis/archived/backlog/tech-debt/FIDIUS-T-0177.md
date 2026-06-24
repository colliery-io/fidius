---
id: escape-wit-reserved-keywords-in
level: task
title: "Escape WIT reserved keywords in fidius-wit identifier emission"
short_code: "FIDIUS-T-0177"
created_at: 2026-06-21T13:06:30.023867+00:00
updated_at: 2026-06-21T14:52:22.605108+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Escape WIT reserved keywords in fidius-wit identifier emission

## Type
- [x] Tech Debt / Bug — generated WIT is un-buildable for keyword-named fields

## Priority
- [x] P1 — silently breaks any guest interface whose type has a keyword field

## Objective

`fidius-wit` emits WIT identifiers (record field names, variant case names,
record/variant names, method/param names, interface name) verbatim from
kebab-cased Rust identifiers. When an identifier collides with a WIT reserved
keyword (`record`, `stream`, `from`, `type`, `result`, `list`, …) the generated
`wit/` fails to parse, so `fidius_build::emit_wit()` dies in the build script
before the component compiles (`cargo build --target wasm32-wasip2`).

Escape keyword identifiers with a leading `%` (WIT source syntax; the semantic
name is unchanged) at every WIT-text emission site. Because the host descriptor
WIT and the guest `emit_wit` WIT already flow through the **same** `fidius-wit`
functions, fixing it there is the single source of truth — they cannot drift.

## Root-cause notes

- `WitType` derive is a no-op marker; ALL WIT generation (host descriptor via
  `render_wit`, guest `emit_wit` via `generate.rs`) flows through `fidius-wit`.
  Neither side escaped — the host only "compiled fine" because the cdylib path
  never feeds WIT through a parser.
- Interface hash is computed from Rust **signature strings**
  (`fidius_core::hash::interface_hash`), NOT WIT text — so `%` escaping does not
  affect the hash; host↔guest hashes match by construction.
- `%` is pure WIT source syntax: `parse_id` accepts `ExplicitId` and strips the
  `%`; keyword-matching only applies to bare ids. So runtime export-name lookups
  (`WasmMethodDesc.name`, kebab, no `%`) stay correct and must NOT be escaped.
- WIT keyword set taken from wit-parser 0.236.1 `ast/lex.rs`.
- Also strip a leading `r#` raw-ident prefix before kebab-casing so the WIT
  keywords that are *also* Rust keywords (`type`→`r#type`, `enum`, `use`,
  `static`, `as`, `async`) round-trip as `%type`, etc.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] A guest type with keyword-named fields (`record`, `stream`, `from`,
      `r#type`) emits as `%record`, … and the generated WIT parses. — proven by
      the `keyword-fields` wasm fixture building for `wasm32-wasip2`.
- [x] Emitted WIT is identical between host descriptor and guest `emit_wit`
      (same `fidius-wit` code path) → interface-hash matches host↔guest. —
      proven by `keyword_field_record_round_trips` (host loads + calls the
      keyword interface; load checks the hash).
- [x] Keyword-escaping logic is a single shared helper in `fidius-wit`
      (`is_wit_keyword` / `wit_ident`), consumed by every emission site.
- [x] Round-trip parse test (wit-parser) over a keyword-heavy interface
      (`keyword_heavy_interface_parses`).

## Status Updates

**DONE.** Shipped in `fidius-wit` (single source of truth) + a regression caught
by the real wasm build.

### Changes
- `crates/fidius-wit/src/lib.rs`: `WIT_KEYWORDS` + `is_wit_keyword` + `wit_ident`
  (`%`-escape); `to_kebab_case` strips a leading `r#`. Applied `wit_ident` at every
  WIT-text emission site — record/variant names, field names, variant case names,
  user-type references, method func names, param names, and the interface/package/
  export names. Compound names (`<name>-stream`, `<enum>-<case>`, `<iface>-plugin`)
  can't collide so are left bare.
- `crates/fidius-wit/src/generate.rs`: `render_conversions` now distinguishes the
  *author* field ident (`r#type`) from the *wit-bindgen* mirror ident (`type_`).
  wit-bindgen mangles Rust-keyword field names with a trailing `_`, not `r#` — so
  the `From` impls must address the generated field by that name. Added
  `RUST_KEYWORDS` + `field_idents`. (The end-to-end wasm build caught a missing
  `"type"` in this list that the unit tests had not.)
- `crates/fidius-wit/Cargo.toml`: `wit-parser` dev-dep for the round-trip test.
- `tests/wasm-fixtures/keyword-fields/`: new fixture, every author identifier a
  WIT keyword. Builds for `wasm32-wasip2`.
- `crates/fidius-host/tests/keyword_fields_wasm.rs`: builds + loads + round-trips
  the fixture.
- `.gitignore`: ignore the fixture's generated `wit/`.

### Verification
- `cargo test -p fidius-wit` (24 unit incl. parser round-trip + conversion-
  mangling), `-p fidius-build` (3), `-p fidius-macro` green.
- `cargo test -p fidius-host --features wasm --test keyword_fields_wasm` green
  (real wasm32-wasip2 build + load + call).
- `--test records_wasm` (4) green → non-keyword conversions un-regressed.
- `cargo clippy --workspace` clean; `cargo fmt` applied.

### Note for next time
The interface hash derives from Rust **signature strings**, not WIT text, so `%`
escaping is hash-neutral by construction — the host↔guest match was never at risk
from escaping, only the *parse* was. The real risk surface was the asymmetry
between wit-bindgen's `type_` field mangling and the author's `r#type`; unit tests
over the WIT text alone can't see it — only a true wasm build compiles the
generated conversions. Keep the fixture in CI.