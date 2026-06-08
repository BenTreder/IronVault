//! CLI commands for IronVault

use ironvault_core::{Config, Result, SafetyConfig};
use std::path::PathBuf;
use tracing::info;

/// Initialize a repository
pub fn cmd_init(repo: &PathBuf) -> Result<()> {
    info!("Initializing repository at {}", repo.display());

    let safety = SafetyConfig::default();
    ironvault_core::init_repo(repo, &safety)?;

    println!("✓ Repository initialized at {}", repo.display());
    Ok(())
}

/// Run a backup
pub fn cmd_backup(config_path: &PathBuf) -> Result<()> {
    info!("Loading configuration from {}", config_path.display());

    let config = Config::load(config_path)?;

    // Run safety checks
    let safety = SafetyChecker::new(&config.safety);
    let sources: Vec<&std::path::Path> =
        config.backup.sources.iter().map(|p| p.as_path()).collect();
    safety.check_all(&config.repo.path, &sources)?;

    // Run backup
    let mut client = ironvault_core::IronVault::new(config)?;
    let snapshot = client.backup()?;

    println!("✓ Backup completed: {}", snapshot.name);
    println!("  Files: {}", snapshot.file_count());
    println!("  Size: {} bytes", snapshot.total_size());
    Ok(())
}

/// Run a dry-run
pub fn cmd_dry_run(config_path: &PathBuf) -> Result<()> {
    info!(
        "Running dry-run with configuration from {}",
        config_path.display()
    );

    let config = Config::load(config_path)?;

    let client = ironvault_core::IronVault::new(config)?;
    let files = client.dry_run()?;

    println!("✓ Dry-run completed");
    println!("  Files that would be backed up: {}", files.len());
    Ok(())
}

/// List snapshots
pub fn cmd_snapshots(repo: &PathBuf) -> Result<()> {
    info!("Listing snapshots from {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let snapshots = repository.list_snapshots()?;

    println!("Snapshots:");
    for snapshot in &snapshots {
        println!(
            "  {} - {} files, {} bytes",
            snapshot.name,
            snapshot.file_count(),
            snapshot.total_size()
        );
    }
    Ok(())
}

/// Show repository info
pub fn cmd_info(repo: &PathBuf) -> Result<()> {
    info!("Getting repository info for {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let info = repository.info()?;

    println!("Repository: {}", info.path);
    println!("Total size: {} bytes", info.total_size);
    println!("Total chunks: {}", info.total_chunks);
    println!("Snapshots: {}", info.snapshot_count);
    println!("Free space: {} bytes", info.free_space);
    Ok(())
}

/// Verify repository
pub fn cmd_verify(repo: &PathBuf) -> Result<()> {
    info!("Verifying repository {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let valid = repository.verify()?;

    if valid {
        println!("✓ Repository is valid");
    } else {
        println!("✗ Repository has errors");
    }
    Ok(())
}

/// Generate restore plan
pub fn cmd_restore_plan(snapshot: &str, target: &PathBuf, repo: &PathBuf) -> Result<()> {
    info!(
        "Generating restore plan for {} to {}",
        snapshot,
        target.display()
    );

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let snap = repository.get_snapshot(snapshot)?;

    let manager = ironvault_core::restore::RestoreManager::new(repo);
    let plan = manager.generate_plan(&snap, target)?;

    ironvault_core::restore::display_plan(&plan);
    Ok(())
}

/// Execute restore
pub fn cmd_restore(snapshot: &str, target: &PathBuf, repo: &PathBuf) -> Result<()> {
    info!("Restoring {} to {}", snapshot, target.display());

    // Safety check: never restore to root
    if *target == PathBuf::from("/") {
        return Err(ironvault_core::IronVaultError::Restore(
            "Cannot restore to root filesystem".to_string(),
        ));
    }

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let snap = repository.get_snapshot(snapshot)?;

    let manager = ironvault_core::restore::RestoreManager::new(repo);
    let plan = manager.generate_plan(&snap, target)?;
    let count = manager.execute(&plan)?;

    println!("✓ Restored {} files to {}", count, target.display());
    Ok(())
}

/// Prune old snapshots
pub fn cmd_prune(config_path: &PathBuf) -> Result<()> {
    info!("Pruning with configuration from {}", config_path.display());

    let config = Config::load(config_path)?;
    let mut client = ironvault_core::IronVault::new(config)?;
    client.prune()?;

    println!("✓ Prune completed");
    Ok(())
}

/// Compact repository
pub fn cmd_compact(repo: &PathBuf) -> Result<()> {
    info!("Compacting repository {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    repository.compact()?;

    println!("✓ Compact completed");
    Ok(())
}

/// Safety checker wrapper
struct SafetyChecker {
    config: ironvault_core::SafetyConfig,
}

impl SafetyChecker {
    fn new(config: &ironvault_core::SafetyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    fn check_all(&self, repo: &PathBuf, sources: &[&std::path::Path]) -> Result<()> {
        ironvault_core::safety::SafetyChecker::new(&self.config).check_all(repo, sources)
    }
}
