//! IronVault CLI - Command-line interface for the backup system

use clap::Parser;
use ironvault_core::Result;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::fmt;

mod commands;

use commands::*;

/// IronVault - A premium backup program for Arch Linux
#[derive(Parser, Debug)]
#[command(name = "ironvault")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Initialize a new repository
    Init {
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
    },
    /// Create a backup
    Backup {
        /// Configuration file path
        #[arg(short, long)]
        config: PathBuf,
    },
    /// Dry run - scan files without backing up
    DryRun {
        /// Configuration file path
        #[arg(short, long)]
        config: PathBuf,
    },
    /// List snapshots
    Snapshots {
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
    },
    /// Show repository info
    Info {
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
    },
    /// Verify repository
    Verify {
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
    },
    /// Generate a restore plan
    RestorePlan {
        /// Snapshot name
        #[arg(short, long, default_value = "latest")]
        snapshot: String,
        /// Target directory
        #[arg(short, long)]
        target: PathBuf,
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
        /// Output restore plan as JSON for GUI and automation use
        #[arg(long)]
        json: bool,
    },
    /// Execute a restore
    Restore {
        /// Snapshot name
        #[arg(short, long, default_value = "latest")]
        snapshot: String,
        /// Target directory
        #[arg(short, long)]
        target: PathBuf,
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
        /// What to do if a restore target already exists: refuse or skip
        #[arg(long, default_value = "refuse", value_parser = ["refuse", "skip"])]
        if_exists: String,
    },
    /// Prune old snapshots
    Prune {
        /// Configuration file path
        #[arg(short, long)]
        config: PathBuf,
    },
    /// Compact the repository
    Compact {
        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,
    },
}

fn init_logging(level: Level) {
    fmt()
        .with_env_filter(format!("ironvault={}", level.as_str()))
        .with_writer(std::io::stderr)
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(Level::INFO);

    match &cli.command {
        Commands::Init { repo } => cmd_init(repo)?,
        Commands::Backup { config } => cmd_backup(config)?,
        Commands::DryRun { config } => cmd_dry_run(config)?,
        Commands::Snapshots { repo } => cmd_snapshots(repo)?,
        Commands::Info { repo } => cmd_info(repo)?,
        Commands::Verify { repo } => cmd_verify(repo)?,
        Commands::RestorePlan {
            snapshot,
            target,
            repo,
            json,
        } => cmd_restore_plan(snapshot, target, repo, *json)?,
        Commands::Restore {
            snapshot,
            target,
            repo,
            if_exists,
        } => cmd_restore(snapshot, target, repo, if_exists)?,
        Commands::Prune { config } => cmd_prune(config)?,
        Commands::Compact { repo } => cmd_compact(repo)?,
    }

    Ok(())
}
