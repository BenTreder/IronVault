//! Repository manifest for IronVault

use crate::{Result, Snapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Repository manifest - tracks all snapshots and chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub repository_id: String,
    pub created_at: String,
    pub last_modified: String,
    pub snapshots: HashMap<String, SnapshotReference>,
    pub chunks: HashMap<String, ChunkReference>,
}

impl Manifest {
    pub const CURRENT_VERSION: &'static str = "1.0.0";

    pub fn new() -> Self {
        use uuid::Uuid;
        Self {
            version: Self::CURRENT_VERSION.to_string(),
            repository_id: Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_modified: chrono::Utc::now().to_rfc3339(),
            snapshots: HashMap::new(),
            chunks: HashMap::new(),
        }
    }

    /// Add a snapshot reference
    pub fn add_snapshot(&mut self, snapshot: &Snapshot) {
        self.snapshots.insert(
            snapshot.id.clone(),
            SnapshotReference {
                id: snapshot.id.clone(),
                name: snapshot.name.clone(),
                created_at: snapshot.created_at.to_rfc3339(),
                file_count: snapshot.file_count(),
            },
        );
        self.last_modified = chrono::Utc::now().to_rfc3339();
    }

    /// Remove a snapshot reference
    pub fn remove_snapshot(&mut self, id: &str) {
        self.snapshots.remove(id);
        self.last_modified = chrono::Utc::now().to_rfc3339();
    }

    /// Add a chunk reference
    pub fn add_chunk(&mut self, hash: String, path: PathBuf, size: u64) {
        self.chunks.insert(
            hash,
            ChunkReference {
                path: path.to_string_lossy().to_string(),
                size,
                referenced_by: Vec::new(),
            },
        );
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference to a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotReference {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub file_count: usize,
}

/// Reference to a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkReference {
    pub path: String,
    pub size: u64,
    pub referenced_by: Vec<String>, // snapshot IDs
}
