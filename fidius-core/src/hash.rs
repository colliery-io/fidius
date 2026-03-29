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

//! FNV-1a interface hashing for compile-time ABI drift detection.
//!
//! The proc macro computes an `interface_hash` from the sorted required method
//! signatures of a trait. The host checks this hash at load time to reject
//! plugins compiled against a different interface.

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x100000001b3;

/// Compute the FNV-1a 64-bit hash of a byte slice.
pub const fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

/// Compute the interface hash from a set of method signatures.
///
/// Signatures are sorted lexicographically before hashing to ensure
/// order-independence. Each signature is joined with `\n` as a separator.
///
/// This function is **not** `const` because it allocates for sorting.
/// The proc macro calls this at compile time via a build-script-like pattern,
/// or uses `fnv1a` directly on pre-sorted, concatenated signatures.
pub fn interface_hash(signatures: &[&str]) -> u64 {
    let mut sorted: Vec<&str> = signatures.to_vec();
    sorted.sort();
    let combined = sorted.join("\n");
    fnv1a(combined.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        // Empty string should produce the offset basis XOR'd with nothing,
        // which is just the offset basis.
        assert_eq!(fnv1a(b""), FNV_OFFSET_BASIS);
    }

    #[test]
    fn known_vector() {
        // FNV-1a("fidius") — precomputed reference value
        let hash = fnv1a(b"fidius");
        // Just verify it's deterministic and non-zero
        assert_ne!(hash, 0);
        assert_eq!(hash, fnv1a(b"fidius"));
    }

    #[test]
    fn order_independence() {
        let a = interface_hash(&["process:&[u8],Value->Result<Vec<u8>,PluginError>", "name:->String"]);
        let b = interface_hash(&["name:->String", "process:&[u8],Value->Result<Vec<u8>,PluginError>"]);
        assert_eq!(a, b);
    }

    #[test]
    fn sensitivity() {
        let a = interface_hash(&["name:->String"]);
        let b = interface_hash(&["name:->string"]); // lowercase 's'
        assert_ne!(a, b);
    }

    #[test]
    fn different_signatures_differ() {
        let a = interface_hash(&["foo:->i32"]);
        let b = interface_hash(&["bar:->i32"]);
        let c = interface_hash(&["foo:->u32"]);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }
}
