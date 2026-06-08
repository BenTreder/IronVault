//! Snapshot types for IronVault

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A snapshot manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub created_at: DateTime<Utc>,
    pub source_paths: Vec<String>,
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub symlinks: Vec<SymlinkEntry>,
    pub metadata: SnapshotMetadata,
}

impl Snapshot {
    pub fn new(name: String, source_paths: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
            created_at: Utc::now(),
            source_paths,
            files: Vec::new(),
            directories: Vec::new(),
            symlinks: Vec::new(),
            metadata: SnapshotMetadata::default(),
        }
    }

    /// Add a file entry
    pub fn add_file(&mut self, entry: FileEntry) {
        self.files.push(entry);
    }

    /// Add a directory entry
    pub fn add_directory(&mut self, entry: DirectoryEntry) {
        self.directories.push(entry);
    }

    /// Add a symlink entry
    pub fn add_symlink(&mut self, entry: SymlinkEntry) {
        self.symlinks.push(entry);
    }

    /// Total size of files in this snapshot
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Number of files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Number of directories
    pub fn directory_count(&self) -> usize {
        self.directories.len()
    }

    /// Number of symlinks
    pub fn symlink_count(&self) -> usize {
        self.symlinks.len()
    }
}

/// File entry in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub permissions: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime: DateTime<Utc>,
    pub chunk_hashes: Vec<String>,
    pub compression: String,
}

impl FileEntry {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            size,
            permissions: 0o644,
            uid: 0,
            gid: 0,
            mtime: Utc::now(),
            chunk_hashes: Vec::new(),
            compression: "zstd".to_string(),
        }
    }
}

/// Directory entry in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub path: String,
    pub permissions: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime: DateTime<Utc>,
}

impl DirectoryEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            permissions: 0o755,
            uid: 0,
            gid: 0,
            mtime: Utc::now(),
        }
    }
}

/// Symlink entry in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkEntry {
    pub path: String,
    pub target: String,
}

impl SymlinkEntry {
    pub fn new(path: PathBuf, target: PathBuf) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            target: target.to_string_lossy().to_string(),
        }
    }
}

/// Metadata stored in each snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotMetadata {
    pub package_list: Option<Vec<String>>,
    pub enabled_services: Option<Vec<String>>,
    pub block_devices: Option<Vec<String>>,
    pub kernel_info: Option<String>,
    pub arch_info: Option<String>,
}

/// Snapshot manifest (stored separately)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub snapshot: Snapshot,
    pub version: String,
}

impl SnapshotManifest {
    pub const CURRENT_VERSION: &'static str = "1.0.0";

    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            version: Self::CURRENT_VERSION.to_string(),
        }
    }
}
