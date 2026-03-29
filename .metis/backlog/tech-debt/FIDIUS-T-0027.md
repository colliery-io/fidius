---
id: architecture-detection-for-wrong
level: task
title: "Architecture detection for wrong-platform dylib rejection"
short_code: "FIDIUS-T-0027"
created_at: 2026-03-29T12:20:40.262140+00:00
updated_at: 2026-03-29T12:43:15.547870+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Architecture detection for wrong-platform dylib rejection

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Add architecture detection to the plugin load sequence. Before `dlopen`, read the first few bytes of the dylib to check the binary format (ELF/Mach-O/PE) and target architecture. Reject wrong-platform dylibs with a clear error instead of letting `dlopen` fail with a cryptic OS error.

## Technical Debt Impact

- **Current Problems**: Loading a Linux `.so` on macOS produces an opaque `libloading` error. Users have to figure out the platform mismatch themselves.
- **Benefits of Fixing**: Clear error: "expected Mach-O aarch64, got ELF x86_64" — immediately tells the user what's wrong.
- **Risk Assessment**: Low — additive check before the existing load path.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `check_architecture(path) -> Result<(), LoadError>` reads first 16 bytes and validates
- [ ] Detects: ELF (magic `\x7fELF`), Mach-O (magic `0xFEEDFACE`/`0xFEEDFACF`), PE (magic `MZ`)
- [ ] Extracts target arch from headers (x86_64, aarch64, etc.)
- [ ] Returns `LoadError::ArchitectureMismatch { expected, got }` on mismatch
- [ ] Called in `loader::load_library()` before `Library::new()`
- [ ] Unit tests with crafted minimal headers

## Implementation Notes

### Technical Approach

File: `fidius-host/src/arch.rs`

Read the magic bytes and architecture fields from the binary headers. No need for a full parser — just enough to extract format + arch.

- ELF: bytes 0-3 = `\x7fELF`, byte 18 = e_machine (0x3E = x86_64, 0xB7 = aarch64)
- Mach-O: bytes 0-3 = magic, bytes 4-7 = cputype (7 = x86_64, 12 = aarch64)
- PE: bytes 0-1 = `MZ`, COFF header at offset from PE signature

## Status Updates

- **2026-03-29**: Implemented in `fidius-host/src/arch.rs`. Detects ELF, Mach-O (both endiannesses), PE. Extracts x86_64/aarch64 arch. `check_architecture()` wired into `load_library()` before dlopen. `LoadError::ArchitectureMismatch` added. 4 unit tests (ELF, Mach-O LE, PE, unknown). All 18 fidius-host tests pass.