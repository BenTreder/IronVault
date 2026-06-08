//! IronVault Daemon - Background backup service

use clap::Parser;
use ironvault_core::Result;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::fmt;

#[derive(Parser, Debug)]
#[command(name = "ironvault-daemon")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: PathBuf,
    #[arg(short, long)]
    daemon: bool,
}

fn init_logging(level: Level) {
    fmt()
        .with_env_filter(format!("ironvault={}", level.as_str()))
        .with_writer(Box::new(std::io::stderr()))
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(Level::INFO);

    info!("IronVault daemon starting");

    if cli.daemon {
        // Run as daemon (would fork and detach in production)
        info!("Running as daemon");
        run_daemon(&cli.config)?;
    } else {
        // Run once
        info!("Running single backup");
        run_once(&cli.config)?;
    }

    Ok(())
}

fn run_daemon(config: &PathBuf) -> Result<()> {
    // In production, this would:
    // 1. Fork and detach from terminal
    // 2. Set up signal handlers
    // 3. Run backup at scheduled intervals
    // 4. Handle log rotation

    info!("Daemon mode not yet fully implemented");
    Ok(())
}

fn run_once(config: &PathBuf) -> Result<()> {
    let ironvault_config = ironvault_core::Config::load(config)?;
    let client = ironvault_core::IronVault::new(ironvault_config)?;
    let snapshot = client.backup()?;
    info!(name = snapshot.name, "Backup completed");
    Ok(())
}
