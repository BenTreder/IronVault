//! Hash utilities for IronVault

use crate::Result;

/// Compute BLAKE3 hash of data
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Compute BLAKE3 hash and return as hex string
pub fn hash_to_hex(data: &[u8]) -> String {
    let hash = hash_data(data);
    hex::encode(hash)
}

/// Convert hash bytes to path (for object storage)
pub fn hash_to_path(hash: &[u8; 32]) -> String {
    // Use first 2 bytes as directory, rest as filename
    format!("{:02x}/{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}.chunk",
        hash[0], hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
        hash[8], hash[9], hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]
    )
}

/// Trait for hashable objects
pub trait Hashable {
    fn hash(&self) -> Result<[u8; 32]>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data() {
        let data = b"hello world";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_to_hex() {
        let data = b"hello world";
        let hex = hash_to_hex(data);
        assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_hash_to_path() {
        let hash = [0u8; 32];
        let path = hash_to_path(&hash);
        assert_eq!(path, "00/00000000000000000000000000000000.chunk");
    }
}
