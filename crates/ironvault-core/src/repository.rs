//! Repository management for IronVault

use crate::{Config, RepositoryInfo, Result, SafetyConfig, Snapshot};
use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, span, Level};

/// Repository configuration
#[derive(Debug, Clone)]
pub struct RepositoryConfig {
    pub path: PathBuf,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./ironvault-repo"),
        }
    }
}

/// IronVault repository
pub struct Repository {
    config: RepositoryConfig,
    _safety: SafetyConfig,
    db: Connection,
}

impl Repository {
    /// Open an existing repository
    pub fn open(config: &Path, safety: &SafetyConfig) -> Result<Self> {
        Self::open_with_config(
            &RepositoryConfig {
                path: config.to_path_buf(),
            },
            safety,
        )
    }

    /// Open with explicit config
    pub fn open_with_config(config: &RepositoryConfig, safety: &SafetyConfig) -> Result<Self> {
        let _span = span!(
            Level::INFO,
            "open_repository",
            path = config.path.display().to_string()
        );

        // Ensure directory exists
        fs::create_dir_all(&config.path)?;

        // Create subdirectories
        let objects_path = config.path.join("objects");
        let snapshots_path = config.path.join("snapshots");
        let indexes_path = config.path.join("indexes");
        let logs_path = config.path.join("logs");
        let metadata_path = config.path.join("metadata");
        let locks_path = config.path.join("locks");

        fs::create_dir_all(&objects_path)?;
        fs::create_dir_all(&snapshots_path)?;
        fs::create_dir_all(&indexes_path)?;
        fs::create_dir_all(&logs_path)?;
        fs::create_dir_all(&metadata_path)?;
        fs::create_dir_all(&locks_path)?;

        // Open or create database
        let db_path = indexes_path.join("chunks.sqlite");
        let db = Connection::open(&db_path)?;

        // Initialize schema
        Self::init_db(&db)?;

        info!("Repository opened at {}", config.path.display());
        Ok(Self {
            config: config.clone(),
            _safety: safety.clone(),
            db,
        })
    }

    /// Initialize database schema
    fn init_db(db: &Connection) -> Result<()> {
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
                hash TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                size INTEGER NOT NULL,
                compressed_size INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                source_paths TEXT NOT NULL,
                files_count INTEGER NOT NULL,
                size INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS snapshot_chunks (
                snapshot_id TEXT NOT NULL,
                chunk_hash TEXT NOT NULL,
                PRIMARY KEY (snapshot_id, chunk_hash)
            );",
        )?;
        Ok(())
    }

    /// Initialize a new repository
    pub fn init(path: &Path, _safety: &SafetyConfig) -> Result<()> {
        let _span = span!(
            Level::INFO,
            "init_repository",
            path = path.display().to_string()
        );

        if path.exists() {
            return Err(crate::IronVaultError::Repository(
                "Repository already exists".to_string(),
            ));
        }

        // Create directory structure
        fs::create_dir_all(path)?;

        let objects_path = path.join("objects");
        let snapshots_path = path.join("snapshots");
        let indexes_path = path.join("indexes");
        let logs_path = path.join("logs");
        let metadata_path = path.join("metadata");
        let locks_path = path.join("locks");

        fs::create_dir_all(&objects_path)?;
        fs::create_dir_all(&snapshots_path)?;
        fs::create_dir_all(&indexes_path)?;
        fs::create_dir_all(&logs_path)?;
        fs::create_dir_all(&metadata_path)?;
        fs::create_dir_all(&locks_path)?;

        // Create repo.toml
        let repo_config = serde_json::json!({
            "version": "1.0.0",
            "created_at": Utc::now().to_rfc3339(),
            "hostname": gethostname::gethostname().to_string_lossy().to_string(),
        });

        let config_path = path.join("repo.yaml");
        let mut file = fs::File::create(&config_path)?;
        file.write_all(serde_yaml::to_string(&repo_config)?.as_bytes())?;

        // Initialize database
        let db_path = indexes_path.join("chunks.sqlite");
        let db = Connection::open(&db_path)?;
        Self::init_db(&db)?;

        info!("Repository initialized at {}", path.display());
        Ok(())
    }

    /// Scan source directories
    pub fn scan_sources(&self, config: &Config) -> Result<Vec<String>> {
        let scanner = crate::scanner::Scanner::new(config);
        let files = scanner.scan()?;
        Ok(files
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect())
    }

    /// Create a snapshot
    pub fn create_snapshot(&mut self, config: &Config) -> Result<Snapshot> {
        let _span = span!(Level::INFO, "create_snapshot");

        let name = format!("auto-{}", Utc::now().format("%Y-%m-%d_%H-%M-%S"));

        let mut snapshot = Snapshot::new(
            name.clone(),
            config
                .backup
                .sources
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        );

        let files = self.scan_sources(config)?;

        for file_str in files {
            let file_path = std::path::PathBuf::from(&file_str);
            let metadata = std::fs::metadata(&file_path)?;
            let size = metadata.len();

            let chunk_size = config.backup.parse_chunk_size()?;
            let chunker = crate::chunker::Chunker::new(chunk_size);
            let chunks = chunker.chunk_file(&file_path)?;
            let mut chunk_hashes = Vec::new();

            for mut chunk in chunks {
                let hash = chunk.hash();
                let hash_hex = hex::encode(hash);
                let object_rel_path = crate::hasher::hash_to_path(&hash);
                let object_path = self.config.path.join("objects").join(object_rel_path);

                if let Some(parent) = object_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                if !object_path.exists() {
                    let data = chunk.data();
                    let compressed = if config.backup.compression == "zstd" {
                        crate::compression::compress(data, config.backup.compression_level)?
                    } else {
                        data.to_vec()
                    };
                    std::fs::write(&object_path, compressed)?;
                }

                chunk_hashes.push(hash_hex);
            }

            let relative_path = config
                .backup
                .sources
                .iter()
                .find_map(|source| {
                    file_path.strip_prefix(source).ok().map(|stripped| {
                        source
                            .file_name()
                            .map(|name| std::path::PathBuf::from(name).join(stripped))
                            .unwrap_or_else(|| stripped.to_path_buf())
                    })
                })
                .unwrap_or_else(|| {
                    file_path
                        .file_name()
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| file_path.clone())
                });

            let mut entry = crate::FileEntry::new(relative_path, size);
            entry.chunk_hashes = chunk_hashes;
            entry.compression = config.backup.compression.clone();

            snapshot.add_file(entry);
        }

        // Store snapshot
        self.store_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    /// Store a snapshot
    pub fn store_snapshot(&mut self, snapshot: &Snapshot) -> Result<()> {
        let path = self
            .config
            .path
            .join("snapshots")
            .join(format!("{}.snapshot.json", snapshot.name));
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, json)?;
        info!("Snapshot stored: {}", snapshot.name);
        Ok(())
    }

    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        let snapshots_path = self.config.path.join("snapshots");
        let mut snapshots = Vec::new();

        if let Ok(entries) = fs::read_dir(snapshots_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    let content = fs::read_to_string(&path)?;
                    let snapshot: Snapshot = serde_json::from_str(&content)?;
                    snapshots.push(snapshot);
                }
            }
        }

        // Sort by date, newest first
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    /// Get a specific snapshot
    pub fn get_snapshot(&self, name: &str) -> Result<Snapshot> {
        if name == "latest" {
            let snapshots = self.list_snapshots()?;
            return snapshots.into_iter().next().ok_or_else(|| {
                crate::IronVaultError::Repository("No snapshots found".to_string())
            });
        }

        let path = self
            .config
            .path
            .join("snapshots")
            .join(format!("{}.snapshot.json", name));
        let content = fs::read_to_string(&path)?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        Ok(snapshot)
    }

    /// Verify repository integrity
    pub fn verify(&self) -> Result<bool> {
        info!("Verifying repository");
        // TODO: Implement verification
        Ok(true)
    }

    /// Prune old snapshots
    pub fn prune(&mut self, policy: &crate::config::RetentionConfig) -> Result<()> {
        let _span = span!(Level::INFO, "prune_operation");
        info!("Pruning snapshots with policy: {:?}", policy);

        let snapshots = self.list_snapshots()?;
        let to_delete: Vec<_> = snapshots
            .iter()
            .filter(|s| self.should_prune(s, policy))
            .collect();

        for snapshot in to_delete {
            let path = self
                .config
                .path
                .join("snapshots")
                .join(format!("{}.snapshot.json", snapshot.name));
            fs::remove_file(&path)?;
            info!("Deleted snapshot: {}", snapshot.name);
        }

        Ok(())
    }

    /// Check if a snapshot should be pruned
    fn should_prune(&self, _snapshot: &Snapshot, _policy: &crate::config::RetentionConfig) -> bool {
        // TODO: Implement proper retention logic
        // For now, just keep recent snapshots
        false
    }

    /// Compact the repository
    pub fn compact(&self) -> Result<()> {
        let _span = span!(Level::INFO, "compact_operation");
        info!("Compacting repository");
        // TODO: Implement compaction
        Ok(())
    }

    /// Get repository info
    pub fn info(&self) -> Result<RepositoryInfo> {
        let path = &self.config.path;
        let total_size = self.calculate_dir_size(path);
        let total_chunks = self.count_chunks()?;
        let snapshot_count = self.list_snapshots()?.len();
        let free_space = crate::safety::get_free_space(path)?;

        Ok(RepositoryInfo {
            path: path.to_string_lossy().to_string(),
            total_size,
            total_chunks,
            snapshot_count,
            free_space,
        })
    }

    /// Calculate total directory size
    fn calculate_dir_size(&self, path: &Path) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    total += self.calculate_dir_size(&path);
                } else if path.is_file() {
                    total += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
        total
    }

    /// Count chunks in the database
    fn count_chunks(&self) -> Result<u64> {
        let count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
        Ok(count as u64)
    }
}
