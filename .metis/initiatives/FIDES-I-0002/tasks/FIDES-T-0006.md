---
id: trait-parsing-and-ir-extraction
level: task
title: "Trait parsing and IR extraction"
short_code: "FIDES-T-0006"
created_at: 2026-03-29T00:53:32.149951+00:00
updated_at: 2026-03-29T00:59:03.921803+00:00
parent: FIDES-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0002
---

# Trait parsing and IR extraction

## Parent Initiative

[[FIDES-I-0002]]

## Objective

Build the intermediate representation (IR) layer that parses a `syn::ItemTrait` into a structured `InterfaceIR` — the data model both macros consume. This includes extracting method signatures, detecting `#[optional(since = N)]` attributes, detecting `async fn`, parsing macro attributes (`version`, `buffer`), and building the canonical signature strings used for interface hashing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `InterfaceIR` struct with: trait name, version, buffer strategy, list of `MethodIR`
- [ ] `MethodIR` struct with: method name, argument types (as syn types), return type, is_async flag, optional (with `since` version), signature string for hashing
- [ ] Parse `#[plugin_interface(version = N, buffer = PluginAllocated)]` attributes
- [ ] Parse `#[optional(since = N)]` on individual methods
- [ ] Build canonical signature string per method: `"name:arg_type_1,arg_type_2->return_type"` (using `quote::ToTokens` on syn types)
- [ ] Detect `async fn` methods and set `is_async` flag
- [ ] Error on `&mut self` (stateless plugins only take `&self`)
- [ ] Error on more than 64 optional methods
- [ ] Unit tests: parse a sample trait, verify IR fields

## Implementation Notes

### Technical Approach

File: `fides-macro/src/ir.rs`

Use `syn::parse2::<ItemTrait>` to get the trait AST. Walk `trait_items` extracting each `TraitItemFn`. For each method:
1. Extract `sig.ident` (name), `sig.inputs` (args, skip `self`), `sig.output` (return type)
2. Check for `sig.asyncness`
3. Check attrs for `#[optional(since = N)]`
4. Build signature string by `quote!(#ty).to_string()` for each arg/return type

The `InterfaceAttrs` struct parses the `#[plugin_interface(...)]` attribute arguments using syn's `parse::Parse`.

### Dependencies
- None internal — this is pure syn parsing, no codegen yet

## Status Updates

- **2026-03-29**: Implemented in `fides-macro/src/ir.rs`. InterfaceIR, MethodIR, InterfaceAttrs all working. Parses version/buffer attrs, optional(since=N), async detection, &mut self rejection, 64 optional limit. 6 unit tests pass.