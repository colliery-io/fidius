# fidius-host::arch <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Architecture detection for dylib files.

Reads binary headers to determine format (ELF/Mach-O/PE) and target
architecture before attempting to dlopen.

## Structs

### `fidius-host::arch::BinaryInfo`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`, `PartialEq`, `Eq`

Detected binary format and architecture.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `format` | `BinaryFormat` |  |
| `arch` | `Arch` |  |



## Enums

### `fidius-host::arch::BinaryFormat` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


#### Variants

- **`Elf`**
- **`MachO`**
- **`Pe`**
- **`Unknown`**



### `fidius-host::arch::Arch` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


#### Variants

- **`X86_64`**
- **`Aarch64`**
- **`Unknown`**



## Functions

### `fidius-host::arch::detect_architecture`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn detect_architecture (path : & Path) -> Result < BinaryInfo , LoadError >
```

Detect the binary format and architecture of a file.

<details>
<summary>Source</summary>

```rust
pub fn detect_architecture(path: &Path) -> Result<BinaryInfo, LoadError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LoadError::LibraryNotFound {
                path: path.display().to_string(),
            }
        } else {
            LoadError::Io(e)
        }
    })?;

    let mut bytes = [0u8; 20];
    let n = file.read(&mut bytes).map_err(LoadError::Io)?;

    if n < 16 {
        return Ok(BinaryInfo {
            format: BinaryFormat::Unknown,
            arch: Arch::Unknown,
        });
    }
    let bytes = &bytes[..n];

    // ELF: \x7fELF
    if bytes[0..4] == [0x7f, b'E', b'L', b'F'] {
        let arch = if bytes.len() > 19 {
            let e_machine = u16::from_le_bytes([bytes[18], bytes[19]]);
            match e_machine {
                0x3E => Arch::X86_64,
                0xB7 => Arch::Aarch64,
                _ => Arch::Unknown,
            }
        } else {
            Arch::Unknown
        };
        return Ok(BinaryInfo {
            format: BinaryFormat::Elf,
            arch,
        });
    }

    // Mach-O: 0xFEEDFACE (32-bit) or 0xFEEDFACF (64-bit), or reversed (big-endian)
    let magic32 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if matches!(magic32, 0xFEEDFACE | 0xFEEDFACF | 0xCEFAEDFE | 0xCFFAEDFE) {
        let arch = if bytes.len() > 8 {
            // cputype is at offset 4, 4 bytes
            let is_le = matches!(magic32, 0xCEFAEDFE | 0xCFFAEDFE);
            let cputype = if is_le {
                u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
            } else {
                u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
            };
            match cputype {
                0x01000007 => Arch::X86_64,  // CPU_TYPE_X86_64
                0x0100000C => Arch::Aarch64, // CPU_TYPE_ARM64
                _ => Arch::Unknown,
            }
        } else {
            Arch::Unknown
        };
        return Ok(BinaryInfo {
            format: BinaryFormat::MachO,
            arch,
        });
    }

    // PE: MZ
    if bytes[0..2] == [b'M', b'Z'] {
        return Ok(BinaryInfo {
            format: BinaryFormat::Pe,
            arch: Arch::Unknown, // Would need to parse PE/COFF headers for arch
        });
    }

    Ok(BinaryInfo {
        format: BinaryFormat::Unknown,
        arch: Arch::Unknown,
    })
}
```

</details>



### `fidius-host::arch::check_architecture`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn check_architecture (path : & Path) -> Result < () , LoadError >
```

Check that a dylib matches the current platform's expected format.

<details>
<summary>Source</summary>

```rust
pub fn check_architecture(path: &Path) -> Result<(), LoadError> {
    let info = detect_architecture(path)?;

    let expected_format = if cfg!(target_os = "macos") {
        BinaryFormat::MachO
    } else if cfg!(target_os = "windows") {
        BinaryFormat::Pe
    } else {
        BinaryFormat::Elf
    };

    let expected_arch = if cfg!(target_arch = "x86_64") {
        Arch::X86_64
    } else if cfg!(target_arch = "aarch64") {
        Arch::Aarch64
    } else {
        Arch::Unknown
    };

    // Only reject on clear mismatches — don't reject Unknown
    if info.format != BinaryFormat::Unknown && info.format != expected_format {
        return Err(LoadError::ArchitectureMismatch {
            expected: format!("{} {}", expected_format, expected_arch),
            got: format!("{} {}", info.format, info.arch),
        });
    }

    if info.arch != Arch::Unknown && expected_arch != Arch::Unknown && info.arch != expected_arch {
        return Err(LoadError::ArchitectureMismatch {
            expected: format!("{} {}", expected_format, expected_arch),
            got: format!("{} {}", info.format, info.arch),
        });
    }

    Ok(())
}
```

</details>



