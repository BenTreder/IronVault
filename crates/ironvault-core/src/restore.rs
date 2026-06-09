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
    pub conflicts: Vec<RestoreConflict>,
}

impl RestorePlan {
    pub fn new(snapshot: Snapshot, target: PathBuf) -> Self {
        Self {
            snapshot,
            target,
            files: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    /// Add a file to restore
    pub fn add_file(&mut self, item: RestoreItem) {
        self.files.push(item);
    }

    /// Add a restore conflict
    pub fn add_conflict(&mut self, conflict: RestoreConflict) {
        self.conflicts.push(conflict);
    }

    /// Total files to restore
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Total directories to restore
    pub fn directory_count(&self) -> usize {
        self.snapshot.directories.len()
    }

    /// Total symlinks to restore
    pub fn symlink_count(&self) -> usize {
        self.snapshot.symlinks.len()
    }

    /// Total restore conflicts
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Whether this plan is safe to execute
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
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

/// A restore conflict found during planning
#[derive(Debug, Clone)]
pub struct RestoreConflict {
    pub source_path: String,
    pub target_path: PathBuf,
    pub kind: String,
}

impl RestoreConflict {
    pub fn new(source_path: String, target_path: PathBuf, kind: impl Into<String>) -> Self {
        Self {
            source_path,
            target_path,
            kind: kind.into(),
        }
    }
}

/// Restore behavior when a target path already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreIfExists {
    /// Refuse to restore over any existing file or symlink target.
    Refuse,
    /// Leave existing targets untouched and restore only missing targets.
    Skip,
}

impl Default for RestoreIfExists {
    fn default() -> Self {
        Self::Refuse
    }
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

        self.collect_target_conflicts(&mut plan)?;

        info!(
            files = plan.file_count(),
            directories = plan.directory_count(),
            symlinks = plan.symlink_count(),
            conflicts = plan.conflict_count(),
            "Restore plan generated"
        );
        Ok(plan)
    }

    /// Collect existing target conflicts for files and symlinks.
    fn collect_target_conflicts(&self, plan: &mut RestorePlan) -> Result<()> {
        let file_conflicts: Vec<_> = plan
            .files
            .iter()
            .filter(|item| Self::restore_target_exists(&item.target_path))
            .map(|item| {
                RestoreConflict::new(
                    item.source_path.clone(),
                    item.target_path.clone(),
                    "file target already exists",
                )
            })
            .collect();

        for conflict in file_conflicts {
            plan.add_conflict(conflict);
        }

        let symlink_conflicts: Vec<_> = plan
            .snapshot
            .symlinks
            .iter()
            .filter_map(|link| {
                let target_path = Self::safe_restore_path(&plan.target, &link.path).ok()?;

                if Self::restore_target_exists(&target_path) {
                    Some(RestoreConflict::new(
                        link.path.clone(),
                        target_path,
                        "symlink target already exists",
                    ))
                } else {
                    None
                }
            })
            .collect();

        for conflict in symlink_conflicts {
            plan.add_conflict(conflict);
        }

        Ok(())
    }

    /// Check whether a restore target already exists, including symlinks.
    fn restore_target_exists(target_path: &Path) -> bool {
        fs::symlink_metadata(target_path).is_ok()
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

    /// Execute a restore plan using the safest default behavior.
    pub fn execute(&self, plan: &RestorePlan) -> Result<usize> {
        self.execute_with_if_exists(plan, RestoreIfExists::Refuse)
    }

    /// Execute a restore plan with explicit existing-target behavior.
    pub fn execute_with_if_exists(
        &self,
        plan: &RestorePlan,
        if_exists: RestoreIfExists,
    ) -> Result<usize> {
        let _span = span!(Level::INFO, "execute_restore");

        info!(
            snapshot = plan.snapshot.name,
            target = plan.target.display().to_string(),
            files = plan.files.len(),
            conflicts = plan.conflict_count(),
            if_exists = ?if_exists,
            "Starting restore"
        );

        if plan.has_conflicts() && if_exists == RestoreIfExists::Refuse {
            let first = &plan.conflicts[0];
            return Err(IronVaultError::Restore(format!(
                "Vault door closed. Restore plan has {} conflict(s). First conflict: {} -> {} ({})",
                plan.conflict_count(),
                first.source_path,
                first.target_path.display(),
                first.kind
            )));
        }

        // Create target directory
        fs::create_dir_all(&plan.target)?;

        if if_exists == RestoreIfExists::Refuse {
            self.preflight_target_conflicts(plan)?;
        }

        self.create_directories(plan)?;

        // Restore files
        let mut restored = 0;
        for item in &plan.files {
            if self.restore_file(item, if_exists)? {
                restored += 1;
            }
        }

        self.restore_symlinks(plan, if_exists)?;
        self.apply_directory_metadata(plan)?;

        info!(
            files = restored,
            directories = plan.snapshot.directories.len(),
            symlinks = plan.snapshot.symlinks.len(),
            "Restore completed"
        );
        Ok(restored)
    }

    /// Refuse to restore files or symlinks over existing targets.
    fn preflight_target_conflicts(&self, plan: &RestorePlan) -> Result<()> {
        for item in &plan.files {
            Self::reject_existing_restore_target(&item.target_path)?;
        }

        for link in &plan.snapshot.symlinks {
            let target_path = Self::safe_restore_path(&plan.target, &link.path)?;
            Self::reject_existing_restore_target(&target_path)?;
        }

        Ok(())
    }

    /// Reject any existing filesystem entry before restore writes to it.
    fn reject_existing_restore_target(target_path: &Path) -> Result<()> {
        if Self::restore_target_exists(target_path) {
            return Err(IronVaultError::Restore(format!(
                "Refusing to overwrite existing restore target: {}",
                target_path.display()
            )));
        }

        Ok(())
    }

    /// Create directories from the snapshot before restoring files.
    fn create_directories(&self, plan: &RestorePlan) -> Result<usize> {
        let mut created = 0;

        for dir in &plan.snapshot.directories {
            let target_path = Self::safe_restore_path(&plan.target, &dir.path)?;
            fs::create_dir_all(&target_path)?;
            created += 1;
        }

        Ok(created)
    }

    /// Apply directory permissions and modified times after file restore.
    fn apply_directory_metadata(&self, plan: &RestorePlan) -> Result<()> {
        for dir in plan.snapshot.directories.iter().rev() {
            let target_path = Self::safe_restore_path(&plan.target, &dir.path)?;

            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&target_path, fs::Permissions::from_mode(dir.permissions))?;
            }

            let mtime = filetime::FileTime::from_unix_time(
                dir.mtime.timestamp(),
                dir.mtime.timestamp_subsec_nanos(),
            );
            filetime::set_file_mtime(&target_path, mtime)?;
        }

        Ok(())
    }

    /// Restore symlinks from the snapshot.
    fn restore_symlinks(&self, plan: &RestorePlan, if_exists: RestoreIfExists) -> Result<usize> {
        let mut restored = 0;

        for link in &plan.snapshot.symlinks {
            Self::validate_symlink_target(&link.target)?;

            let target_path = Self::safe_restore_path(&plan.target, &link.path)?;

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            if Self::restore_target_exists(&target_path) {
                match if_exists {
                    RestoreIfExists::Refuse => Self::reject_existing_restore_target(&target_path)?,
                    RestoreIfExists::Skip => continue,
                }
            }

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&link.target, &target_path)?;
                restored += 1;
            }

            #[cfg(not(unix))]
            {
                return Err(IronVaultError::Restore(
                    "Symlink restore is only supported on Unix-like systems".to_string(),
                ));
            }
        }

        Ok(restored)
    }

    /// Keep restored symlinks conservative and local to the restored tree.
    fn validate_symlink_target(link_target: &str) -> Result<()> {
        let target = Path::new(link_target);

        if target.is_absolute() {
            return Err(IronVaultError::Restore(format!(
                "Refusing to restore symlink with absolute target: {}",
                link_target
            )));
        }

        let mut has_normal_component = false;

        for component in target.components() {
            match component {
                Component::Normal(_) => has_normal_component = true,
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(IronVaultError::Restore(format!(
                        "Refusing to restore symlink with parent traversal target: {}",
                        link_target
                    )));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(IronVaultError::Restore(format!(
                        "Refusing to restore unsafe symlink target: {}",
                        link_target
                    )));
                }
            }
        }

        if !has_normal_component {
            return Err(IronVaultError::Restore(
                "Refusing to restore empty symlink target".to_string(),
            ));
        }

        Ok(())
    }

    /// Restore a single file
    fn restore_file(&self, item: &RestoreItem, if_exists: RestoreIfExists) -> Result<bool> {
        let target_path = &item.target_path;

        if Self::restore_target_exists(target_path) {
            match if_exists {
                RestoreIfExists::Refuse => Self::reject_existing_restore_target(target_path)?,
                RestoreIfExists::Skip => return Ok(false),
            }
        }

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
    println!("  IronVault checked the map before opening the vault.");
    println!("  Snapshot: {}", plan.snapshot.name);
    println!("  Target: {}", plan.target.display());
    println!("  Files: {}", plan.file_count());
    println!("  Directories: {}", plan.directory_count());
    println!("  Symlinks: {}", plan.symlink_count());
    println!("  Conflicts: {}", plan.conflict_count());

    if plan.has_conflicts() {
        println!(
            "\nVault door closed. IronVault found restore conflicts and will not overwrite them."
        );
        println!("\nConflicts:");
        for conflict in &plan.conflicts {
            println!(
                "  {} -> {} ({})",
                conflict.source_path,
                conflict.target_path.display(),
                conflict.kind
            );
        }
    } else {
        println!("\nVault check passed. Safe to restore.");
    }

    println!("\nFiles to restore:");
    for item in &plan.files {
        println!("  {} -> {}", item.source_path, item.target_path.display());
    }

    if !plan.snapshot.directories.is_empty() {
        println!("\nDirectories to restore:");
        for dir in &plan.snapshot.directories {
            println!("  {}", dir.path);
        }
    }

    if !plan.snapshot.symlinks.is_empty() {
        println!("\nSymlinks to restore:");
        for link in &plan.snapshot.symlinks {
            println!("  {} -> {}", link.path, link.target);
        }
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
    #[test]
    fn symlink_target_rejects_absolute_path() {
        let err = RestoreManager::validate_symlink_target("/etc/passwd").unwrap_err();

        assert!(
            err.to_string().contains("absolute target"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn symlink_target_rejects_parent_traversal() {
        let err = RestoreManager::validate_symlink_target("../outside.txt").unwrap_err();

        assert!(
            err.to_string().contains("parent traversal"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn symlink_target_accepts_normal_relative_path() {
        RestoreManager::validate_symlink_target("file.txt").unwrap();
        RestoreManager::validate_symlink_target("subdir/file.txt").unwrap();
    }
}
