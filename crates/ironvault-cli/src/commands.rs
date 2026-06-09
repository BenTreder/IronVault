//! CLI commands for IronVault

use ironvault_core::{Config, Result, SafetyConfig};
use std::path::PathBuf;
use tracing::info;

/// Initialize a repository
pub fn cmd_init(repo: &PathBuf) -> Result<()> {
    info!("Initializing repository at {}", repo.display());

    let safety = SafetyConfig::default();
    ironvault_core::init_repo(repo, &safety)?;

    println!("✓ Vault created at {}", repo.display());
    println!("  IronVault is ready to start locking files away.");
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

    println!("✓ Backup sealed: {}", snapshot.name);
    println!(
        "  Files: {} tucked safely into the vault",
        snapshot.file_count()
    );
    println!("  Original size: {} bytes", snapshot.total_size());
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

    println!("✓ Dry-run complete. No files were moved, just a careful peek.");
    println!("  Files that would be backed up: {}", files.len());
    Ok(())
}

/// List snapshots
pub fn cmd_snapshots(repo: &PathBuf, json: bool) -> Result<()> {
    info!("Listing snapshots from {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let snapshots = repository.list_snapshots()?;

    if json {
        let snapshots_json: Vec<_> = snapshots
            .iter()
            .map(|snapshot| {
                serde_json::json!({
                    "name": snapshot.name,
                    "files": snapshot.file_count(),
                    "directories": snapshot.directory_count(),
                    "symlinks": snapshot.symlink_count(),
                    "total_size": snapshot.total_size(),
                })
            })
            .collect();

        let output = serde_json::json!({
            "snapshot_count": snapshots_json.len(),
            "snapshots": snapshots_json,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Snapshots:");
        println!("  Vault shelves found:");
        for snapshot in &snapshots {
            println!(
                "  {} - {} files, {} bytes sealed",
                snapshot.name,
                snapshot.file_count(),
                snapshot.total_size()
            );
        }
    }

    Ok(())
}

/// Show repository info
pub fn cmd_info(repo: &PathBuf, json: bool) -> Result<()> {
    info!("Getting repository info for {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let info = repository.info()?;

    if json {
        let output = serde_json::json!({
            "path": info.path,
            "total_size": info.total_size,
            "total_chunks": info.total_chunks,
            "snapshot_count": info.snapshot_count,
            "free_space": info.free_space,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Vault location: {}", info.path);
        println!("Vault size: {} bytes", info.total_size);
        println!("Vault pieces: {}", info.total_chunks);
        println!("Snapshots sealed: {}", info.snapshot_count);
        println!("Free space nearby: {} bytes", info.free_space);
    }

    Ok(())
}

/// Verify repository
pub fn cmd_verify(repo: &PathBuf, json: bool) -> Result<()> {
    info!("Verifying repository {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let valid = repository.verify()?;

    let message = if valid {
        "Repository is valid. Every vault piece is accounted for."
    } else {
        "Repository has errors. The vault check found missing or damaged pieces."
    };

    if json {
        let output = serde_json::json!({
            "valid": valid,
            "message": message,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if valid {
        println!("✓ {}", message);
    } else {
        println!("✗ {}", message);
    }

    Ok(())
}

/// Generate restore plan
pub fn cmd_restore_plan(
    snapshot: &str,
    target: &PathBuf,
    repo: &PathBuf,
    json: bool,
) -> Result<()> {
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

    if json {
        let conflicts: Vec<_> = plan
            .conflicts
            .iter()
            .map(|conflict| {
                serde_json::json!({
                    "source_path": conflict.source_path,
                    "target_path": conflict.target_path.display().to_string(),
                    "kind": conflict.kind,
                })
            })
            .collect();

        let total_size: u64 = plan.files.iter().map(|item| item.size).sum();

        let output = serde_json::json!({
            "snapshot": plan.snapshot.name,
            "target": plan.target.display().to_string(),
            "files": plan.file_count(),
            "directories": plan.directory_count(),
            "symlinks": plan.symlink_count(),
            "total_size": total_size,
            "conflict_count": plan.conflict_count(),
            "safe_to_restore": !plan.has_conflicts(),
            "conflicts": conflicts,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        ironvault_core::restore::display_plan(&plan);
    }

    Ok(())
}

/// Execute restore
pub fn cmd_restore(
    snapshot: &str,
    target: &PathBuf,
    repo: &PathBuf,
    if_exists: &str,
) -> Result<()> {
    info!("Restoring {} to {}", snapshot, target.display());

    // Safety check: never restore to root
    if *target == PathBuf::from("/") {
        return Err(ironvault_core::IronVaultError::Restore(
            "Cannot restore to root filesystem. Vault door closed for safety.".to_string(),
        ));
    }

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    let snap = repository.get_snapshot(snapshot)?;

    let manager = ironvault_core::restore::RestoreManager::new(repo);
    let plan = manager.generate_plan(&snap, target)?;

    let if_exists_mode = match if_exists {
        "skip" => ironvault_core::restore::RestoreIfExists::Skip,
        _ => ironvault_core::restore::RestoreIfExists::Refuse,
    };

    let count = manager.execute_with_if_exists(&plan, if_exists_mode)?;

    println!(
        "✓ Restore unlocked {} files into {}",
        count,
        target.display()
    );
    if if_exists == "skip" {
        println!("  Skipped existing targets. IronVault only unlocked what was missing.");
    } else {
        println!("  Vault door closed behind us. Nothing extra was overwritten.");
    }
    Ok(())
}

/// Prune old snapshots
pub fn cmd_prune(config_path: &PathBuf) -> Result<()> {
    info!("Pruning with configuration from {}", config_path.display());

    let config = Config::load(config_path)?;
    let mut client = ironvault_core::IronVault::new(config)?;
    client.prune()?;

    println!("✓ Prune completed. Old vault shelves cleaned up.");
    Ok(())
}

/// Compact repository
pub fn cmd_compact(repo: &PathBuf) -> Result<()> {
    info!("Compacting repository {}", repo.display());

    let safety = SafetyConfig::default();
    let repository = ironvault_core::repository::Repository::open(repo, &safety)?;
    repository.compact()?;

    println!("✓ Compact completed. The vault is tidier now.");
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
