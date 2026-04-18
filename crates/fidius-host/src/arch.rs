// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Architecture detection for dylib files.
//!
//! Reads binary headers to determine format (ELF/Mach-O/PE) and target
//! architecture before attempting to dlopen.

use std::path::Path;

use crate::error::LoadError;

/// Detected binary format and architecture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryInfo {
    pub format: BinaryFormat,
    pub arch: Arch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    Elf,
    MachO,
    Pe,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Aarch64,
    Unknown,
}

impl std::fmt::Display for BinaryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFormat::Elf => write!(f, "ELF"),
            BinaryFormat::MachO => write!(f, "Mach-O"),
            BinaryFormat::Pe => write!(f, "PE"),
            BinaryFormat::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::X86_64 => write!(f, "x86_64"),
            Arch::Aarch64 => write!(f, "aarch64"),
            Arch::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detect the binary format and architecture of a file.
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

/// Check that a dylib matches the current platform's expected format.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_elf() {
        // Minimal ELF header: magic + enough bytes for e_machine at offset 18
        let mut bytes = vec![0u8; 20];
        bytes[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
        bytes[18..20].copy_from_slice(&0x3Eu16.to_le_bytes()); // x86_64

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &bytes).unwrap();

        let info = detect_architecture(tmp.path()).unwrap();
        assert_eq!(info.format, BinaryFormat::Elf);
        assert_eq!(info.arch, Arch::X86_64);
    }

    #[test]
    fn detects_macho_le() {
        // Mach-O little-endian 64-bit: 0xCFFAEDFE, cputype ARM64 = 0x0100000C
        let mut bytes = vec![0u8; 16];
        bytes[0..4].copy_from_slice(&0xCFFAEDFEu32.to_be_bytes());
        bytes[4..8].copy_from_slice(&0x0100000Cu32.to_le_bytes()); // ARM64

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &bytes).unwrap();

        let info = detect_architecture(tmp.path()).unwrap();
        assert_eq!(info.format, BinaryFormat::MachO);
        assert_eq!(info.arch, Arch::Aarch64);
    }

    #[test]
    fn detects_pe() {
        let mut bytes = vec![0u8; 16];
        bytes[0..2].copy_from_slice(&[b'M', b'Z']);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &bytes).unwrap();

        let info = detect_architecture(tmp.path()).unwrap();
        assert_eq!(info.format, BinaryFormat::Pe);
    }

    #[test]
    fn unknown_format() {
        let bytes = vec![0u8; 16];

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &bytes).unwrap();

        let info = detect_architecture(tmp.path()).unwrap();
        assert_eq!(info.format, BinaryFormat::Unknown);
    }
}
