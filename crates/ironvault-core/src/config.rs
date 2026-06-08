//! Configuration types and parsing for IronVault

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repo: RepoConfig,
    pub backup: BackupConfig,
    pub retention: RetentionConfig,
    pub safety: SafetyConfig,
    pub excludes: ExcludesConfig,
    pub metadata: MetadataConfig,
    pub notifications: NotificationsConfig,
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo: RepoConfig::default(),
            backup: BackupConfig::default(),
            retention: RetentionConfig::default(),
            safety: SafetyConfig::default(),
            excludes: ExcludesConfig::default(),
            metadata: MetadataConfig::default(),
            notifications: NotificationsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, crate::IronVaultError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::IronVaultError::Config(format!("Failed to read config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| crate::IronVaultError::Config(format!("Failed to parse config: {}", e)))
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<(), crate::IronVaultError> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::IronVaultError::Config(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, content)
            .map_err(|e| crate::IronVaultError::Config(format!("Failed to write config: {}", e)))
    }
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoConfig {
    pub path: PathBuf,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupConfig {
    pub sources: Vec<PathBuf>,
    pub one_file_system: bool,
    pub follow_symlinks: bool,
    pub parallelism: String,
    pub chunk_size: String,
    pub compression: String,
    pub compression_level: i32,
}

impl BackupConfig {
    /// Parse chunk size string (e.g., "4MiB")
    pub fn parse_chunk_size(&self) -> Result<usize, crate::IronVaultError> {
        let s = self.chunk_size.to_lowercase();
        let (number, unit) = if s.ends_with("mib") {
            (s.trim_end_matches("mib").parse::<usize>()?, 1024 * 1024)
        } else if s.ends_with("gib") {
            (
                s.trim_end_matches("gib").parse::<usize>()?,
                1024 * 1024 * 1024,
            )
        } else if s.ends_with("kib") {
            (s.trim_end_matches("kib").parse::<usize>()?, 1024)
        } else if s.ends_with("b") {
            (s.trim_end_matches("b").parse::<usize>()?, 1)
        } else {
            (s.parse::<usize>()?, 1)
        };
        Ok(number * unit)
    }

    /// Get parallelism level
    pub fn parallelism(&self) -> usize {
        if self.parallelism == "auto" {
            rayon::current_num_threads()
        } else {
            self.parallelism.parse().unwrap_or(1)
        }
    }
}

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetentionConfig {
    pub keep_hourly: usize,
    pub keep_daily: usize,
    pub keep_weekly: usize,
    pub keep_monthly: usize,
    pub keep_yearly: usize,
}

impl RetentionConfig {
    /// Calculate total snapshots to keep
    pub fn total_keep(&self) -> usize {
        self.keep_hourly + self.keep_daily + self.keep_weekly + self.keep_monthly + self.keep_yearly
    }
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafetyConfig {
    pub require_root: bool,
    pub require_repo_mount: bool,
    pub repo_mount_point: PathBuf,
    pub minimum_free_space_gb: u64,
    pub prevent_if_pacman_running: bool,
    pub prevent_if_pacman_lock_exists: bool,
    pub prevent_if_backup_already_running: bool,
    pub lock_file: PathBuf,
    pub never_restore_to_root: bool,
}

impl SafetyConfig {
    /// Check if running as root
    pub fn is_root(&self) -> bool {
        cfg!(target_os = "linux") && nix::unistd::geteuid().as_raw() == 0
    }
}

/// Exclusion configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExcludesConfig {
    pub paths: Vec<PathBuf>,
}

/// Metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataConfig {
    pub save_package_list: bool,
    pub save_enabled_services: bool,
    pub save_block_devices: bool,
    pub save_kernel_info: bool,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub desktop_notifications: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    pub log_dir: PathBuf,
    pub log_level: String,
}

impl LoggingConfig {
    /// Get the log level
    pub fn level(&self) -> tracing::Level {
        self.log_level.parse().unwrap_or(tracing::Level::INFO)
    }
}

/// Default exclusions that are always applied
pub fn default_exclusions() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/proc"),
        PathBuf::from("/sys"),
        PathBuf::from("/dev"),
        PathBuf::from("/run"),
        PathBuf::from("/tmp"),
        PathBuf::from("/var/tmp"),
        PathBuf::from("/mnt"),
        PathBuf::from("/media"),
        PathBuf::from("/lost+found"),
        PathBuf::from("/swapfile"),
    ]
}
