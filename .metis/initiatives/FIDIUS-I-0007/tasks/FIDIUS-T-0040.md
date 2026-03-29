---
id: r-06-fix-detect-architecture-to
level: task
title: "R-06: Fix detect_architecture to read only header bytes"
short_code: "FIDIUS-T-0040"
created_at: 2026-03-29T16:29:48.852407+00:00
updated_at: 2026-03-29T16:41:40.739552+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-06: Fix detect_architecture to read only header bytes

**Addresses**: PRF-001, COR-08, COR-12, LEG-08

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Fix `detect_architecture` to read only header bytes instead of the entire file. Discovery scans every dylib in all search paths, and each call currently reads the entire file into memory just to inspect 20 header bytes. This causes massive unnecessary memory allocation for directories with many large dylibs. Additionally, IO errors (e.g., permission denied) are misclassified as "library not found."

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `std::fs::read(path)` replaced with `File::open` + `read_exact` for a 20-byte buffer
- [ ] IO errors properly mapped: `NotFound` maps to `LibraryNotFound`, other errors map to a new `LoadError::Io` variant
- [ ] Function uses `&buf[..n]` instead of reading the full file contents
- [ ] `LoadError::Io` variant added if one does not exist

## Implementation Notes

### Technical Approach

1. In `fidius-host/src/arch.rs`, replace:
   ```rust
   let bytes = std::fs::read(path).map_err(|_| ...)?;
   ```
   with:
   ```rust
   let mut buf = [0u8; 20];
   let mut file = std::fs::File::open(path).map_err(|e| {
       if e.kind() == std::io::ErrorKind::NotFound {
           LoadError::LibraryNotFound { path: path.display().to_string() }
       } else {
           LoadError::Io { path: path.display().to_string(), source: e.to_string() }
       }
   })?;
   let n = file.read(&mut buf).map_err(|e| ...)?;
   ```
2. Add a `LoadError::Io` variant if one does not exist, or reuse an appropriate variant.
3. Update the rest of the function to use `&buf[..n]` instead of `&bytes`.

### Dependencies

None.

## Status Updates

*To be added during implementation*