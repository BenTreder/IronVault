//! Error types for IronVault

use thiserror::Error;

/// Main result type for IronVault
pub type Result<T> = std::result::Result<T, IronVaultError>;

/// All possible errors in IronVault
#[derive(Error, Debug)]
pub enum IronVaultError {
    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Safety error: {0}")]
    Safety(String),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Chunk error: {0}")]
    Chunk(String),

    #[error("Snapshot error: {0}")]
    Snapshot(String),

    #[error("Restore error: {0}")]
    Restore(String),

    #[error("Prune error: {0}")]
    Prune(String),

    #[error("Hash error: {0}")]
    Hash(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Metadata error: {0}")]
    Metadata(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation cancelled: {0}")]
    Cancelled(String),
}

impl IronVaultError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            IronVaultError::Io(e) => e.kind() == std::io::ErrorKind::Interrupted,
            IronVaultError::Lock(_) => true,
            _ => false,
        }
    }
}

/// Convenience macro for creating safety errors
#[macro_export]
macro_rules! safety_err {
    ($($arg:tt)*) => {
        IronVaultError::Safety(format!($($arg)*))
    };
}

/// Convenience macro for creating repository errors
#[macro_export]
macro_rules! repo_err {
    ($($arg:tt)*) => {
        IronVaultError::Repository(format!($($arg)*))
    };
}
