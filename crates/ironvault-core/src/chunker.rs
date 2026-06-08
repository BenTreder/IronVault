//! Fixed-size chunker for IronVault

use crate::Result;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, span, Level};

/// Default chunk size: 4 MiB
pub const DEFAULT_CHUNK_SIZE: usize = 4 * 1024 * 1024;

/// Chunker for splitting files into fixed-size chunks
pub struct Chunker {
    chunk_size: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl Chunker {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Read chunks from a file
    pub fn chunk_file(&self, path: &Path) -> Result<Vec<Chunk>> {
        let _span = span!(
            Level::DEBUG,
            "chunk_file",
            path = path.display().to_string()
        );
        debug!("Chunking file: {}", path.display());

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; self.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            chunks.push(Chunk::new(buffer[..bytes_read].to_vec()));
        }

        debug!(count = chunks.len(), "Chunks created");
        Ok(chunks)
    }

    /// Chunk data from a reader
    pub fn chunk_reader<R: Read>(&self, reader: &mut R) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; self.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            chunks.push(Chunk::new(buffer[..bytes_read].to_vec()));
        }

        Ok(chunks)
    }
}

/// A single chunk of file data
#[derive(Debug, Clone)]
pub struct Chunk {
    data: Vec<u8>,
    hash: Option<[u8; 32]>,
}

impl Chunk {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, hash: None }
    }

    /// Get the chunk data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the chunk size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Compute and cache the BLAKE3 hash
    pub fn hash(&mut self) -> [u8; 32] {
        if let Some(hash) = self.hash {
            return hash;
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.data);
        let hash = hasher.finalize();
        self.hash = Some(*hash.as_bytes());
        *hash.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunker() {
        let chunker = Chunker::new(1024);
        let data = vec![0u8; 2500];
        let mut cursor = Cursor::new(&data);

        let chunks = chunker.chunk_reader(&mut cursor).unwrap();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 1024);
        assert_eq!(chunks[1].len(), 1024);
        assert_eq!(chunks[2].len(), 452);
    }
}
