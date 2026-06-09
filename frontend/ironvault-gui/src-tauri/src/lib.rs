//! IronVault Tauri Backend
//!
//! The frontend calls these commands through the Tauri invoke bridge.
//! The commands are intentionally small and honest. If the IronVault CLI
//! is not available yet, the frontend gets a clear error instead of fake data.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotInfo {
    name: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    file_count: usize,
    #[serde(default)]
    files: usize,
    #[serde(default)]
    directories: usize,
    #[serde(default)]
    symlinks: usize,
    #[serde(default)]
    total_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BackupResult {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoInfo {
    path: String,
    snapshot_count: usize,
    total_size: u64,
    #[serde(default)]
    total_chunks: Option<u64>,
    free_space: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyResult {
    valid: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotsResponse {
    snapshots: Vec<SnapshotInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RestoreConflict {
    source_path: String,
    target_path: String,
    kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RestorePlanInfo {
    snapshot: String,
    target: String,
    files: usize,
    directories: usize,
    symlinks: usize,
    total_size: u64,
    conflict_count: usize,
    safe_to_restore: bool,
    conflicts: Vec<RestoreConflict>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RestoreResult {
    success: bool,
    message: String,
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
async fn init_repository(path: String) -> Result<String, String> {
    let output = run_ironvault(&["init", "--repo", &path])?;
    Ok(output)
}

#[tauri::command]
async fn get_info(repo_path: String) -> Result<RepoInfo, String> {
    let output = run_ironvault(&["info", "--repo", &repo_path, "--json"])?;
    serde_json::from_str(&output).map_err(|error| {
        format!("IronVault returned repository info that the GUI could not read: {error}")
    })
}

#[tauri::command]
async fn verify_repository(repo_path: String) -> Result<VerifyResult, String> {
    let output = run_ironvault(&["verify", "--repo", &repo_path, "--json"])?;
    serde_json::from_str(&output).map_err(|error| {
        format!("IronVault returned verify data that the GUI could not read: {error}")
    })
}

#[tauri::command]
async fn list_snapshots(repo_path: String) -> Result<Vec<SnapshotInfo>, String> {
    let output = run_ironvault(&["snapshots", "--repo", &repo_path, "--json"])?;
    let parsed: SnapshotsResponse = serde_json::from_str(&output).map_err(|error| {
        format!("IronVault returned snapshot data that the GUI could not read: {error}")
    })?;

    Ok(parsed.snapshots)
}

#[tauri::command]
async fn create_backup(config_path: String) -> Result<BackupResult, String> {
    let output = run_ironvault(&["backup", "--config", &config_path])?;

    Ok(BackupResult {
        success: true,
        message: output,
    })
}

#[tauri::command]
async fn restore_plan(
    repo_path: String,
    snapshot: String,
    target_path: String,
) -> Result<RestorePlanInfo, String> {
    let output = run_ironvault(&[
        "restore-plan",
        "--repo",
        &repo_path,
        "--snapshot",
        &snapshot,
        "--target",
        &target_path,
        "--json",
    ])?;

    serde_json::from_str(&output).map_err(|error| {
        format!("IronVault returned restore plan data that the GUI could not read: {error}")
    })
}

#[tauri::command]
async fn restore_snapshot(
    repo_path: String,
    snapshot: String,
    target_path: String,
) -> Result<RestoreResult, String> {
    let output = run_ironvault(&[
        "restore",
        "--repo",
        &repo_path,
        "--snapshot",
        &snapshot,
        "--target",
        &target_path,
        "--if-exists",
        "refuse",
    ])?;

    Ok(RestoreResult {
        success: true,
        message: output,
    })
}

fn run_ironvault(args: &[&str]) -> Result<String, String> {
    let mut last_error = None;

    for candidate in ironvault_candidates() {
        let output = Command::new(&candidate).args(args).output();

        match output {
            Ok(output) if output.status.success() => {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let detail = if stderr.is_empty() { stdout } else { stderr };

                return Err(format!(
                    "IronVault command failed: {} {}. {}",
                    candidate.display(),
                    args.join(" "),
                    detail
                ));
            }
            Err(error) => {
                last_error = Some(format!("{}: {}", candidate.display(), error));
            }
        }
    }

    Err(format!(
        "IronVault CLI was not found. Build it with `cargo build -p ironvault-cli`, or set IRONVAULT_CLI to the ironvault binary path. Last error: {}",
        last_error.unwrap_or_else(|| "no command candidates were tried".to_string())
    ))
}

fn ironvault_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("IRONVAULT_CLI") {
        candidates.push(PathBuf::from(path));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    candidates.push(manifest_dir.join("../../../target/debug/ironvault"));
    candidates.push(manifest_dir.join("../../../target/release/ironvault"));
    candidates.push(PathBuf::from("ironvault"));

    candidates
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_version,
            init_repository,
            list_snapshots,
            create_backup,
            restore_plan,
            restore_snapshot,
            get_info,
            verify_repository,
        ])
        .run(tauri::generate_context!())
        .expect("error running IronVault Tauri application");
}
