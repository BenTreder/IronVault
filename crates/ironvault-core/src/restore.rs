//! Restore functionality for IronVault

use crate::{IronVaultError, Result, Snapshot};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, span, Level};

/// Restore plan
#[derive(Debug, Clone)]
pub struct RestorePlan {
    pub snapshot: Snapshot,
    pub target: PathBuf,
    pub files: Vec<RestoreItem>,
}

impl RestorePlan {
    pub fn new(snapshot: Snapshot, target: PathBuf) -> Self {
        Self {
            snapshot,
            target,
            files: Vec::new(),
        }
    }

    /// Add a file to restore
    pub fn add_file(&mut self, item: RestoreItem) {
        self.files.push(item);
    }

    /// Total files to restore
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// A single restore item
#[derive(Debug, Clone)]
pub struct RestoreItem {
    pub source_path: String,
    pub target_path: PathBuf,
    pub size: u64,
    pub permissions: u32,
    pub uid: u32,
    pub gid: u32,
    pub chunk_hashes: Vec<String>,
    pub compression: String,
}

/// Restore manager
pub struct RestoreManager {
    repo_path: PathBuf,
}

impl RestoreManager {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            repo_path: repo_path.to_path_buf(),
        }
    }

    /// Generate a restore plan
    pub fn generate_plan(&self, snapshot: &Snapshot, target: &Path) -> Result<RestorePlan> {
        let _span = span!(Level::INFO, "generate_restore_plan");

        // Validate target
        if target == Path::new("/") {
            return Err(IronVaultError::Restore(
                "Cannot restore to root filesystem".to_string(),
            ));
        }

        let mut plan = RestorePlan::new(snapshot.clone(), target.to_path_buf());

        // Add files
        for file in &snapshot.files {
            let target_path = target.join(&file.path);
            plan.add_file(RestoreItem {
                source_path: file.path.clone(),
                target_path,
                size: file.size,
                permissions: file.permissions,
                uid: file.uid,
                gid: file.gid,
                chunk_hashes: file.chunk_hashes.clone(),
                compression: file.compression.clone(),
            });
        }

        info!(files = plan.file_count(), "Restore plan generated");
        Ok(plan)
    }

    /// Execute a restore plan
    pub fn execute(&self, plan: &RestorePlan) -> Result<usize> {
        let _span = span!(Level::INFO, "execute_restore");

        info!(
            snapshot = plan.snapshot.name,
            target = plan.target.display().to_string(),
            files = plan.files.len(),
            "Starting restore"
        );

        // Create target directory
        fs::create_dir_all(&plan.target)?;

        // Restore files
        let mut restored = 0;
        for item in &plan.files {
            if self.restore_file(item)? {
                restored += 1;
            }
        }

        info!(count = restored, "Files restored");
        Ok(restored)
    }

    /// Restore a single file
    fn restore_file(&self, item: &RestoreItem) -> Result<bool> {
        let target_path = &item.target_path;

        // Create parent directories
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create file from stored chunks
        let mut file = File::create(target_path)?;

        for hash_hex in &item.chunk_hashes {
            let hash_bytes = hex::decode(hash_hex)
                .map_err(|e| IronVaultError::Restore(format!("Invalid chunk hash: {}", e)))?;

            if hash_bytes.len() != 32 {
                return Err(IronVaultError::Restore(format!(
                    "Invalid chunk hash length for {}",
                    hash_hex
                )));
            }

            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);

            let object_rel_path = crate::hasher::hash_to_path(&hash);
            let object_path = self.repo_path.join("objects").join(object_rel_path);
            let stored = fs::read(&object_path)?;

            let data = if item.compression == "zstd" {
                crate::compression::decompress(&stored)?
            } else {
                stored
            };

            file.write_all(&data)?;
        }

        // Set permissions
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(target_path, fs::Permissions::from_mode(item.permissions))?;
        }

        Ok(true)
    }
}

/// Display a restore plan
pub fn display_plan(plan: &RestorePlan) {
    println!("Restore Plan:");
    println!("  Snapshot: {}", plan.snapshot.name);
    println!("  Target: {}", plan.target.display());
    println!("  Files: {}", plan.file_count());
    println!("\nFiles to restore:");
    for item in &plan.files {
        println!("  {} -> {}", item.source_path, item.target_path.display());
    }
}
