//! Restore functionality for IronVault

use crate::{IronVaultError, Result, Snapshot};
use chrono::{DateTime, Utc};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
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
    pub mtime: DateTime<Utc>,
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
            let target_path = Self::safe_restore_path(target, &file.path)?;
            plan.add_file(RestoreItem {
                source_path: file.path.clone(),
                target_path,
                size: file.size,
                permissions: file.permissions,
                uid: file.uid,
                gid: file.gid,
                mtime: file.mtime.clone(),
                chunk_hashes: file.chunk_hashes.clone(),
                compression: file.compression.clone(),
            });
        }

        info!(files = plan.file_count(), "Restore plan generated");
        Ok(plan)
    }

    /// Build a safe restore path from a snapshot path.
    ///
    /// Snapshot paths must be relative and must not contain parent directory
    /// traversal. This prevents malicious manifests from writing outside the
    /// requested restore target.
    fn safe_restore_path(target: &Path, snapshot_path: &str) -> Result<PathBuf> {
        let source = Path::new(snapshot_path);

        if source.is_absolute() {
            return Err(IronVaultError::Restore(format!(
                "Refusing to restore absolute path from snapshot: {}",
                snapshot_path
            )));
        }

        let mut clean = PathBuf::new();

        for component in source.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(IronVaultError::Restore(format!(
                        "Refusing to restore path with parent traversal: {}",
                        snapshot_path
                    )));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(IronVaultError::Restore(format!(
                        "Refusing to restore unsafe path from snapshot: {}",
                        snapshot_path
                    )));
                }
            }
        }

        if clean.as_os_str().is_empty() {
            return Err(IronVaultError::Restore(
                "Refusing to restore empty snapshot path".to_string(),
            ));
        }

        Ok(target.join(clean))
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

        // Set modified time
        let mtime = filetime::FileTime::from_unix_time(
            item.mtime.timestamp(),
            item.mtime.timestamp_subsec_nanos(),
        );
        filetime::set_file_mtime(target_path, mtime)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileEntry, Snapshot};

    fn test_snapshot_with_path(path: &str) -> Snapshot {
        let mut snapshot = Snapshot::new("test".to_string(), vec!["source".to_string()]);
        let mut entry = FileEntry::new(PathBuf::from(path), 0);
        entry.chunk_hashes = Vec::new();
        snapshot.add_file(entry);
        snapshot
    }

    #[test]
    fn restore_plan_rejects_parent_traversal() {
        let manager = RestoreManager::new(Path::new("/tmp/repo"));
        let snapshot = test_snapshot_with_path("../../evil.txt");

        let err = manager
            .generate_plan(&snapshot, Path::new("/tmp/restore"))
            .unwrap_err();

        assert!(
            err.to_string().contains("parent traversal"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn restore_plan_rejects_absolute_snapshot_path() {
        let manager = RestoreManager::new(Path::new("/tmp/repo"));
        let snapshot = test_snapshot_with_path("/etc/passwd");

        let err = manager
            .generate_plan(&snapshot, Path::new("/tmp/restore"))
            .unwrap_err();

        assert!(
            err.to_string().contains("absolute path"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn restore_plan_accepts_normal_relative_path() {
        let manager = RestoreManager::new(Path::new("/tmp/repo"));
        let snapshot = test_snapshot_with_path("source/subdir/file.txt");

        let plan = manager
            .generate_plan(&snapshot, Path::new("/tmp/restore"))
            .unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(
            plan.files[0].target_path,
            PathBuf::from("/tmp/restore/source/subdir/file.txt")
        );
    }

    #[test]
    fn restore_plan_rejects_root_target() {
        let manager = RestoreManager::new(Path::new("/tmp/repo"));
        let snapshot = test_snapshot_with_path("source/file.txt");

        let err = manager
            .generate_plan(&snapshot, Path::new("/"))
            .unwrap_err();

        assert!(
            err.to_string().contains("root filesystem"),
            "unexpected error: {}",
            err
        );
    }
}
