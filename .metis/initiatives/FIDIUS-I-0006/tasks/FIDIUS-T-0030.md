---
id: packagemanifest-types-and-toml
level: task
title: "PackageManifest types and TOML parsing"
short_code: "FIDIUS-T-0030"
created_at: 2026-03-29T14:00:03.455060+00:00
updated_at: 2026-03-29T14:35:48.847203+00:00
parent: FIDIUS-I-0006
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0006
---

# PackageManifest types and TOML parsing

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0006]]

## Objective

Define the core package types in fidius-core: `PackageManifest<M>` (generic over host-defined metadata), `PackageHeader` (fixed fields), and `PackageError`. Add `toml` as a dependency. Parse `package.toml` files into these types with schema validation via serde.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PackageManifest<M: DeserializeOwned>` struct with `package: PackageHeader`, `dependencies: BTreeMap<String, String>`, `metadata: M`
- [ ] `PackageHeader` struct with `name`, `version`, `interface`, `interface_version: u32`, `source_hash: Option<String>`
- [ ] `PackageError` enum: `ManifestNotFound`, `ParseError(toml::de::Error)`, `SchemaValidation(String)`
- [ ] `load_manifest<M>(dir: &Path) -> Result<PackageManifest<M>, PackageError>` reads `package.toml` from dir
- [ ] `toml` added as workspace dependency to fidius-core
- [ ] Unit tests: valid manifest parses, missing required field fails, extra fields ignored (serde default), missing `package.toml` returns `ManifestNotFound`

## Implementation Notes

### Technical Approach

File: `fidius-core/src/package.rs`

```rust
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageManifest<M> {
    pub package: PackageHeader,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    pub metadata: M,
}
```

The generic `M` is the host's schema type. `toml::from_str::<PackageManifest<M>>(content)?` validates the fixed header AND the metadata section in one deserialization.

### Dependencies
- `toml` crate (add to workspace deps)

## Status Updates

- **2026-03-29**: Implemented in `fidius-core/src/package.rs`. `PackageManifest<M>`, `PackageHeader`, `PackageError` types. `load_manifest::<M>()` and `load_manifest_untyped()`. 7 unit tests: valid parse, dependencies, missing required field, missing manifest, extra fields ignored, untyped accepts any metadata, optional source_hash. All pass.