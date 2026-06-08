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

        self.collect_structure_entries(config, &mut snapshot)?;

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

            let relative_path = Self::relative_snapshot_path(config, &file_path);

            let mut entry = crate::FileEntry::new(relative_path, size);

            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;

                entry.permissions = metadata.mode() & 0o7777;
                entry.uid = metadata.uid();
                entry.gid = metadata.gid();
            }

            entry.mtime = chrono::DateTime::<chrono::Utc>::from(metadata.modified()?);
            entry.chunk_hashes = chunk_hashes;
            entry.compression = config.backup.compression.clone();

            snapshot.add_file(entry);
        }

        // Store snapshot
        self.store_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    /// Collect directory and symlink entries for the snapshot.
    fn collect_structure_entries(&self, config: &Config, snapshot: &mut Snapshot) -> Result<()> {
        for source in &config.backup.sources {
            let mut walker = ignore::WalkBuilder::new(source);
            walker
                .follow_links(false)
                .same_file_system(config.backup.one_file_system);

            for entry in walker.build() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        tracing::warn!("Error scanning structure path: {}", e);
                        continue;
                    }
                };

                let path = entry.path();

                if Self::should_skip_snapshot_path(config, path) {
                    continue;
                }

                let file_type = match entry.file_type() {
                    Some(file_type) => file_type,
                    None => continue,
                };

                if file_type.is_dir() {
                    let metadata = std::fs::symlink_metadata(path)?;
                    let relative_path = Self::relative_snapshot_path(config, path);
                    let mut dir = crate::DirectoryEntry::new(relative_path);

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;

                        dir.permissions = metadata.mode() & 0o7777;
                        dir.uid = metadata.uid();
                        dir.gid = metadata.gid();
                    }

                    if let Ok(modified) = metadata.modified() {
                        dir.mtime = chrono::DateTime::<chrono::Utc>::from(modified);
                    }

                    snapshot.add_directory(dir);
                } else if file_type.is_symlink() {
                    let target = std::fs::read_link(path)?;
                    let relative_path = Self::relative_snapshot_path(config, path);
                    snapshot.add_symlink(crate::SymlinkEntry::new(relative_path, target));
                }
            }
        }

        Ok(())
    }

    /// Convert an absolute source path into the path stored in a snapshot.
    fn relative_snapshot_path(config: &Config, file_path: &std::path::Path) -> std::path::PathBuf {
        config
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
                    .unwrap_or_else(|| file_path.to_path_buf())
            })
    }

    /// Check explicit and default excludes for structure entries.
    fn should_skip_snapshot_path(config: &Config, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        for exclude in &config.excludes.paths {
            let exclude_str = exclude.to_string_lossy();
            if path_str == exclude_str || path_str.starts_with(&format!("{}/", exclude_str)) {
                return true;
            }
        }

        for exclude in crate::config::default_exclusions() {
            let exclude_str = exclude.to_string_lossy();
            if path_str == exclude_str || path_str.starts_with(&format!("{}/", exclude_str)) {
                return true;
            }
        }

        false
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

        let mut valid = true;

        for required_dir in ["objects", "snapshots", "indexes"] {
            let path = self.config.path.join(required_dir);
            if !path.is_dir() {
                tracing::warn!("Missing repository directory: {}", path.display());
                valid = false;
            }
        }

        let snapshots_path = self.config.path.join("snapshots");
        if !snapshots_path.is_dir() {
            tracing::warn!("Missing snapshots directory: {}", snapshots_path.display());
            return Ok(false);
        }

        let mut snapshot_count = 0usize;
        let mut checked_chunks = 0usize;

        for entry in fs::read_dir(&snapshots_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                tracing::warn!(
                    "Skipping snapshot with invalid filename: {}",
                    path.display()
                );
                valid = false;
                continue;
            };

            if !file_name.ends_with(".snapshot.json") {
                continue;
            }

            let json = match fs::read_to_string(&path) {
                Ok(json) => json,
                Err(e) => {
                    tracing::warn!("Could not read snapshot {}: {}", path.display(), e);
                    valid = false;
                    continue;
                }
            };

            let snapshot: Snapshot = match serde_json::from_str(&json) {
                Ok(snapshot) => snapshot,
                Err(e) => {
                    tracing::warn!("Could not parse snapshot {}: {}", path.display(), e);
                    valid = false;
                    continue;
                }
            };

            snapshot_count += 1;

            for file in &snapshot.files {
                for hash_hex in &file.chunk_hashes {
                    checked_chunks += 1;

                    match self.verify_chunk_object(hash_hex, &file.compression) {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::warn!(
                                snapshot = snapshot.name,
                                file = file.path,
                                chunk = hash_hex,
                                "Chunk verification failed"
                            );
                            valid = false;
                        }
                        Err(e) => {
                            tracing::warn!(
                                snapshot = snapshot.name,
                                file = file.path,
                                chunk = hash_hex,
                                error = %e,
                                "Chunk verification errored"
                            );
                            valid = false;
                        }
                    }
                }
            }
        }

        if snapshot_count == 0 {
            tracing::warn!("No snapshots found to verify");
            valid = false;
        }

        if valid {
            info!(
                snapshots = snapshot_count,
                chunks = checked_chunks,
                "Repository verification passed"
            );
        } else {
            tracing::warn!(
                snapshots = snapshot_count,
                chunks = checked_chunks,
                "Repository verification failed"
            );
        }

        Ok(valid)
    }

    /// Verify one stored chunk object against its expected BLAKE3 hash.
    fn verify_chunk_object(&self, hash_hex: &str, compression: &str) -> Result<bool> {
        let hash_bytes = match hex::decode(hash_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!("Invalid chunk hash hex {}: {}", hash_hex, e);
                return Ok(false);
            }
        };

        if hash_bytes.len() != 32 {
            tracing::warn!(
                hash = hash_hex,
                len = hash_bytes.len(),
                "Invalid chunk hash length"
            );
            return Ok(false);
        }

        let mut expected_hash = [0u8; 32];
        expected_hash.copy_from_slice(&hash_bytes);

        let object_rel_path = crate::hasher::hash_to_path(&expected_hash);
        let object_path = self.config.path.join("objects").join(object_rel_path);

        if !object_path.is_file() {
            tracing::warn!(
                hash = hash_hex,
                path = object_path.display().to_string(),
                "Missing chunk object"
            );
            return Ok(false);
        }

        let stored = fs::read(&object_path)?;

        let data = if compression == "zstd" {
            match crate::compression::decompress(&stored) {
                Ok(data) => data,
                Err(e) => {
                    tracing::warn!(
                        hash = hash_hex,
                        path = object_path.display().to_string(),
                        error = %e,
                        "Could not decompress chunk object"
                    );
                    return Ok(false);
                }
            }
        } else {
            stored
        };

        let actual_hash = crate::hasher::hash_data(&data);
        let actual_hash_hex = hex::encode(actual_hash);

        if actual_hash_hex != hash_hex {
            tracing::warn!(
                expected = hash_hex,
                actual = actual_hash_hex,
                "Chunk hash mismatch"
            );
            return Ok(false);
        }

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
