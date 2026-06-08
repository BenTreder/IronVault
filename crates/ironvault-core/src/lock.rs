//! Lock management for IronVault

use crate::{IronVaultError, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Lock file manager
pub struct LockManager {
    path: std::path::PathBuf,
    file: Option<File>,
}

impl LockManager {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            file: None,
        }
    }

    /// Acquire the lock
    pub fn acquire(&mut self) -> Result<()> {
        // Check if lock already exists
        if self.path.exists() {
            let pid_content = std::fs::read_to_string(&self.path)?;
            let pid: u32 = pid_content.trim().parse().unwrap_or(0);

            // Check if process is still running
            if self.is_process_running(pid) {
                return Err(IronVaultError::Lock(format!(
                    "Lock file exists and process {} is still running",
                    pid
                )));
            }

            // Stale lock, remove it
            std::fs::remove_file(&self.path)?;
        }

        // Create lock file
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .map_err(|e| IronVaultError::Lock(format!("Failed to create lock file: {}", e)))?;

        // Write PID to lock file
        let pid = std::process::id();
        let mut file = file;
        file.write_all(format!("{}", pid).as_bytes())
            .map_err(|e| IronVaultError::Lock(format!("Failed to write PID: {}", e)))?;

        self.file = Some(file);
        Ok(())
    }

    /// Release the lock
    pub fn release(&mut self) -> Result<()> {
        if let Some(_file) = self.file.take() {
            let _ = std::fs::remove_file(&self.path);
        }
        Ok(())
    }

    /// Check if a process is running
    fn is_process_running(&self, pid: u32) -> bool {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

impl Drop for LockManager {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lock_path() {
        let lock = LockManager::new(&PathBuf::from("/tmp/test.lock"));
        assert_eq!(lock.path.to_str().unwrap(), "/tmp/test.lock");
    }
}
