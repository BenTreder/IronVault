//! System metadata collection for IronVault

use crate::snapshot::SnapshotMetadata;
use crate::Result;
use std::process::Command;
use tracing::{info, warn};

/// Collect system metadata for backup
pub fn collect_metadata(config: &crate::config::MetadataConfig) -> Result<SnapshotMetadata> {
    let mut metadata = SnapshotMetadata::default();

    if config.save_package_list {
        metadata.package_list = Some(collect_package_list()?);
    }

    if config.save_enabled_services {
        metadata.enabled_services = Some(collect_enabled_services()?);
    }

    if config.save_block_devices {
        metadata.block_devices = Some(collect_block_devices()?);
    }

    if config.save_kernel_info {
        metadata.kernel_info = Some(collect_kernel_info()?);
        metadata.arch_info = Some(collect_arch_info()?);
    }

    Ok(metadata)
}

/// Collect pacman package list
fn collect_package_list() -> Result<Vec<String>> {
    let mut packages = Vec::new();

    // Get explicitly installed packages
    if let Ok(output) = Command::new("pacman").arg("-Qqe").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            packages.extend(stdout.lines().map(|s| s.to_string()));
        }
    } else {
        warn!("Failed to get pacman -Qqe");
    }

    // Get manually installed packages
    if let Ok(output) = Command::new("pacman").arg("-Qqm").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            packages.extend(stdout.lines().map(|s| s.to_string()));
        }
    }

    info!(count = packages.len(), "Packages collected");
    Ok(packages)
}

/// Collect enabled systemd services
fn collect_enabled_services() -> Result<Vec<String>> {
    let mut services = Vec::new();

    if let Ok(output) = Command::new("systemctl")
        .arg("list-unit-files")
        .arg("--state=enabled")
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if let Some(service) = line.split_whitespace().next() {
                    if service.ends_with(".service") || service.ends_with(".timer") {
                        services.push(service.to_string());
                    }
                }
            }
        }
    } else {
        warn!("Failed to get enabled services");
    }

    info!(count = services.len(), "Services collected");
    Ok(services)
}

/// Collect block device information
fn collect_block_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();

    if let Ok(output) = Command::new("lsblk").arg("-f").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            devices = stdout.lines().skip(1).map(|s| s.to_string()).collect();
        }
    } else {
        warn!("Failed to get block devices");
    }

    Ok(devices)
}

/// Collect kernel information
fn collect_kernel_info() -> Result<String> {
    let output = Command::new("uname").arg("-a").output()?;
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| crate::IronVaultError::Metadata(e.to_string()))
}

/// Collect architecture information
fn collect_arch_info() -> Result<String> {
    let output = Command::new("arch").output()?;
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| crate::IronVaultError::Metadata(e.to_string()))
}
