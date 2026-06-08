//! IronVault Core - A deduplicating backup engine for Arch Linux
//!
//! This crate provides the core backup functionality including:
//! - Repository management
//! - File scanning and chunking
//! - Deduplication via content hashing
//! - Snapshot creation and management
//! - Pruning and garbage collection

pub mod chunker;
pub mod compression;
pub mod config;
pub mod errors;
pub mod hash;
pub mod hasher;
pub mod lock;
pub mod manifest;
pub mod metadata;
pub mod prune;
pub mod repository;
pub mod restore;
pub mod safety;
pub mod scanner;
pub mod snapshot;

// Re-export common types
pub use config::{
    BackupConfig, Config, ExcludesConfig, LoggingConfig, MetadataConfig, NotificationsConfig,
    RepoConfig, RetentionConfig, SafetyConfig,
};
pub use errors::{IronVaultError, Result};
pub use manifest::Manifest;
pub use repository::{Repository, RepositoryConfig};
pub use snapshot::{DirectoryEntry, FileEntry, Snapshot, SnapshotManifest, SymlinkEntry};

use std::path::Path;
use tracing::{info, span, Level};

/// Main IronVault client
pub struct IronVault {
    config: Config,
    repo: Repository,
}

impl IronVault {
    /// Create a new IronVault instance
    pub fn new(config: Config) -> Result<Self> {
        let repo = Repository::open(&config.repo.path, &config.safety)?;
        Ok(Self { config, repo })
    }

    /// Run a backup operation
    pub fn backup(&mut self) -> Result<Snapshot> {
        let span = span!(Level::INFO, "backup_operation");
        let _enter = span.enter();
        info!("Starting backup operation");

        let snapshot = self.repo.create_snapshot(&self.config)?;
        info!("Backup completed successfully");
        Ok(snapshot)
    }

    /// Run a dry-run (scan only, no data written)
    pub fn dry_run(&self) -> Result<Vec<String>> {
        let span = span!(Level::INFO, "dry_run_operation");
        let _enter = span.enter();
        info!("Starting dry-run operation");

        let files = self.repo.scan_sources(&self.config)?;
        info!(files_count = files.len(), "Files would be backed up");
        Ok(files)
    }

    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        self.repo.list_snapshots()
    }

    /// Get a specific snapshot by name
    pub fn get_snapshot(&self, name: &str) -> Result<Snapshot> {
        self.repo.get_snapshot(name)
    }

    /// Verify repository integrity
    pub fn verify(&self) -> Result<bool> {
        self.repo.verify()
    }

    /// Prune old snapshots based on retention policy
    pub fn prune(&mut self) -> Result<()> {
        let span = span!(Level::INFO, "prune_operation");
        let _enter = span.enter();
        info!("Starting prune operation");

        let policy = &self.config.retention;
        self.repo.prune(policy)
    }

    /// Compact the repository
    pub fn compact(&self) -> Result<()> {
        self.repo.compact()
    }

    /// Get repository info
    pub fn info(&self) -> Result<RepositoryInfo> {
        self.repo.info()
    }
}

/// Repository information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositoryInfo {
    pub path: String,
    pub total_size: u64,
    pub total_chunks: u64,
    pub snapshot_count: usize,
    pub free_space: u64,
}

/// Initialize a new repository
pub fn init_repo(path: &Path, safety: &SafetyConfig) -> Result<()> {
    Repository::init(path, safety)
}
