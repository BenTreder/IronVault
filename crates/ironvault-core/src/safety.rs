//! Safety checks for IronVault

use crate::{IronVaultError, Result, SafetyConfig};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use tracing::info;

/// Safety checker
pub struct SafetyChecker {
    config: SafetyConfig,
}

impl SafetyChecker {
    pub fn new(config: &SafetyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Run all safety checks
    pub fn check_all(&self, repo_path: &Path, sources: &[&Path]) -> Result<()> {
        self.check_root()?;
        self.check_repo_mount(repo_path)?;
        self.check_free_space(repo_path)?;
        self.check_lock()?;
        self.check_pacman()?;
        self.check_repo_in_sources(repo_path, sources)?;
        Ok(())
    }

    /// Check if running as root (if required)
    pub fn check_root(&self) -> Result<()> {
        if self.config.require_root && !self.config.is_root() {
            return Err(IronVaultError::Safety(
                "Root privileges are required for this operation".to_string(),
            ));
        }
        Ok(())
    }

    /// Check if repository is mounted
    pub fn check_repo_mount(&self, repo_path: &Path) -> Result<()> {
        if !self.config.require_repo_mount {
            return Ok(());
        }

        let mount_point = &self.config.repo_mount_point;
        if !mount_point.exists() {
            return Err(IronVaultError::Safety(format!(
                "Repository mount point does not exist: {}",
                mount_point.display()
            )));
        }

        // Check if repo_path is within the mount
        let canonical_repo = repo_path
            .canonicalize()
            .unwrap_or_else(|_| repo_path.to_path_buf());
        let canonical_mount = mount_point
            .canonicalize()
            .unwrap_or_else(|_| mount_point.clone());

        if !canonical_repo.starts_with(&canonical_mount) {
            return Err(IronVaultError::Safety(format!(
                "Repository path {} is not within the configured mount point {}",
                repo_path.display(),
                mount_point.display()
            )));
        }

        info!("Repository mount check passed");
        Ok(())
    }

    /// Check available disk space
    pub fn check_free_space(&self, repo_path: &Path) -> Result<()> {
        if !self.config.minimum_free_space_gb > 0 {
            return Ok(());
        }

        let free_bytes = get_free_space(repo_path)?;
        let required_bytes = self.config.minimum_free_space_gb * 1024 * 1024 * 1024;

        if free_bytes < required_bytes {
            return Err(IronVaultError::Safety(format!(
                "Insufficient disk space: {} GB required, {:.2} GB available",
                self.config.minimum_free_space_gb,
                free_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )));
        }

        info!("Free space check passed");
        Ok(())
    }

    /// Check lock file
    pub fn check_lock(&self) -> Result<()> {
        if !self.config.prevent_if_backup_already_running {
            return Ok(());
        }

        let lock_file = &self.config.lock_file;
        if lock_file.exists() {
            let pid: String =
                std::fs::read_to_string(lock_file).unwrap_or_else(|_| "unknown".to_string());

            return Err(IronVaultError::Safety(format!(
                "Another backup operation is running (lock file exists: {})",
                pid
            )));
        }

        Ok(())
    }

    /// Check if pacman is running
    pub fn check_pacman(&self) -> Result<()> {
        if !self.config.prevent_if_pacman_running && !self.config.prevent_if_pacman_lock_exists {
            return Ok(());
        }

        // Check for pacman processes
        if self.config.prevent_if_pacman_running {
            let output = std::process::Command::new("pgrep")
                .arg("-x")
                .arg("pacman")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    return Err(IronVaultError::Safety(
                        "Pacman is currently running. Please wait for it to complete.".to_string(),
                    ));
                }
            }
        }

        // Check for pacman lock file
        if self.config.prevent_if_pacman_lock_exists {
            let lock_paths = [
                "/var/lib/pacman/db.lck",
                "/var/run/pacman/PkgsSyncTransition.lock",
            ];

            for lock_path in &lock_paths {
                if Path::new(lock_path).exists() {
                    return Err(IronVaultError::Safety(format!(
                        "Pacman lock file exists: {}",
                        lock_path
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check that repository is not inside source directories
    pub fn check_repo_in_sources(&self, repo_path: &Path, sources: &[&Path]) -> Result<()> {
        for source in sources {
            if repo_path.starts_with(source) || source.starts_with(repo_path) {
                return Err(IronVaultError::Safety(format!(
                    "Repository path {} cannot be inside or contain source path {}",
                    repo_path.display(),
                    source.display()
                )));
            }
        }
        Ok(())
    }
}

/// Get free space on a filesystem
pub fn get_free_space(path: &Path) -> Result<u64> {
    use nix::sys::statvfs::statvfs;

    let stat = statvfs(path)
        .map_err(|e| IronVaultError::Safety(format!("Failed to get filesystem stats: {}", e)))?;

    let block_size = stat.block_size() as u64;
    let available_blocks = stat.blocks_available() as u64;

    Ok(block_size * available_blocks)
}

/// Check if a path is safe to restore to
pub fn validate_restore_target(target: &Path) -> Result<()> {
    if !target.exists() {
        return Err(IronVaultError::Restore(format!(
            "Restore target does not exist: {}",
            target.display()
        )));
    }

    // Never restore to root
    if target == Path::new("/") {
        return Err(IronVaultError::Restore(
            "Restoring directly to / is not allowed. Use a target directory.".to_string(),
        ));
    }

    // Check if target is a subdirectory of /
    let canonical = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    if canonical == Path::new("/") {
        return Err(IronVaultError::Restore(
            "Cannot restore to the root filesystem".to_string(),
        ));
    }

    Ok(())
}

/// Check if a path should be skipped (pseudo filesystems, etc.)
pub fn is_dangerous_path(path: &Path) -> bool {
    let dangerous = ["/proc", "/sys", "/dev", "/run", "/tmp", "/var/tmp"];
    let path_str = path.to_string_lossy();

    for dangerous in &dangerous {
        if path_str == *dangerous || path_str.starts_with(&format!("{}/", dangerous)) {
            return true;
        }
    }
    false
}

/// Check if a file type should be skipped
pub fn should_skip_file(metadata: &std::fs::Metadata) -> bool {
    // Skip special files
    metadata.file_type().is_socket() || metadata.file_type().is_fifo()
}
