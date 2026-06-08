//! Compression utilities for IronVault

use crate::Result;
use zstd::stream::{decode_all, encode_all};

/// Compression level (zstd)
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 10;

/// Compress data using zstd
pub fn compress(data: &[u8], level: i32) -> Result<Vec<u8>> {
    encode_all(std::io::Cursor::new(data), level)
        .map_err(|e| crate::IronVaultError::Compression(e.to_string()))
}

/// Compress data using zstd (simple wrapper)
pub fn compress_simple(data: &[u8], level: i32) -> Result<Vec<u8>> {
    compress(data, level)
}

/// Decompress zstd data
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    decode_all(std::io::Cursor::new(data))
        .map_err(|e| crate::IronVaultError::Compression(e.to_string()))
}

/// Get compressed size ratio
pub fn compression_ratio(original: usize, compressed: usize) -> f64 {
    if original == 0 {
        return 0.0;
    }
    (compressed as f64 / original as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, World! This is a test of compression.";
        let compressed = compress_simple(data, DEFAULT_COMPRESSION_LEVEL).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = compression_ratio(1000, 500);
        assert_eq!(ratio, 50.0);
    }
}
