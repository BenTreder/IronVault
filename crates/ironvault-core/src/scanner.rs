//! File scanner for IronVault

use crate::{Config, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tracing::{debug, info, span, Level};

/// File scanner that walks source directories
pub struct Scanner {
    config: Config,
}

impl Scanner {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Scan all source directories and return file entries
    pub fn scan(&self) -> Result<Vec<PathBuf>> {
        let span = span!(Level::INFO, "file_scan");
        let _enter = span.enter();
        info!("Starting file scan");

        let mut files = Vec::new();
        let excludes = &self.config.excludes.paths;

        for source in &self.config.backup.sources {
            let mut walker = WalkBuilder::new(source);
            walker
                .follow_links(self.config.backup.follow_symlinks)
                .same_file_system(self.config.backup.one_file_system);

            for entry in walker.build() {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();

                        // Skip excluded paths
                        if self.should_skip(path, excludes) {
                            debug!("Skipping excluded path: {}", path.display());
                            continue;
                        }

                        // Skip directories (we want files)
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            continue;
                        }

                        // Skip special files
                        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                            files.push(path.to_path_buf());
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error scanning path: {}", e);
                    }
                }
            }
        }

        info!(count = files.len(), "Files scanned");
        Ok(files)
    }

    /// Check if a path should be skipped
    fn should_skip(&self, path: &Path, excludes: &[PathBuf]) -> bool {
        let path_str = path.to_string_lossy();

        // Check explicit excludes
        for exclude in excludes {
            let exclude_str = exclude.to_string_lossy();
            if path_str == exclude_str || path_str.starts_with(&format!("{}/", exclude_str)) {
                return true;
            }
        }

        // Check default exclusions
        for exclude in crate::config::default_exclusions() {
            let exclude_str = exclude.to_string_lossy();
            if path_str == exclude_str || path_str.starts_with(&format!("{}/", exclude_str)) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip() {
        let config = Config::default();
        let scanner = Scanner::new(&config);

        assert!(scanner.should_skip(&PathBuf::from("/proc/cpuinfo"), &vec![]));
        assert!(scanner.should_skip(&PathBuf::from("/tmp/test"), &vec![]));
        assert!(!scanner.should_skip(&PathBuf::from("/home/user/file.txt"), &vec![]));
    }
}
