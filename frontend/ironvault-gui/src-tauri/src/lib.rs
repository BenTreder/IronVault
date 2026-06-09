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

#[derive(Debug, Serialize, Deserialize)]
struct SetupTestVaultResult {
    repo_path: String,
    config_path: String,
    source_path: String,
    initialized_repo: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetupCustomVaultResult {
    repo_path: String,
    config_path: String,
    source_path: String,
    initialized_repo: bool,
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
async fn setup_test_vault() -> Result<SetupTestVaultResult, String> {
    let home = std::env::var("HOME").map_err(|error| {
        format!("Could not find your home folder for the GUI test source: {error}")
    })?;

    let test_root = std::path::PathBuf::from("/tmp/ironvault-gui-live-test");
    let repo_path = test_root.join("repo");
    let config_path = test_root.join("ironvault.toml");
    let source_path = std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("ironvault")
        .join("gui-live-source");

    std::fs::create_dir_all(source_path.join("docs")).map_err(|error| {
        format!("Could not create GUI test docs folder: {error}")
    })?;

    std::fs::create_dir_all(source_path.join("photos")).map_err(|error| {
        format!("Could not create GUI test photos folder: {error}")
    })?;

    std::fs::create_dir_all(&test_root).map_err(|error| {
        format!("Could not create GUI test vault folder: {error}")
    })?;

    std::fs::write(
        source_path.join("docs").join("readme.txt"),
        "IronVault GUI live test file.\nVault door closed. Everything looks safe.\n",
    )
    .map_err(|error| format!("Could not write GUI test readme file: {error}"))?;

    std::fs::write(
        source_path.join("photos").join("list.txt"),
        "This is a small placeholder file for the GUI test vault.\n",
    )
    .map_err(|error| format!("Could not write GUI test photo list file: {error}"))?;

    let config = format!(
        r#"[repo]
path = "{repo}"

[backup]
sources = ["{source}"]
one_file_system = true
follow_symlinks = false
parallelism = "auto"
chunk_size = "8B"
compression = "zstd"
compression_level = 3

[retention]
keep_hourly = 24
keep_daily = 7
keep_weekly = 4
keep_monthly = 6
keep_yearly = 2

[safety]
require_root = false
require_repo_mount = false
repo_mount_point = "{repo}"
minimum_free_space_gb = 0
prevent_if_pacman_running = false
prevent_if_pacman_lock_exists = false
prevent_if_backup_already_running = false
lock_file = "{lock_file}"
never_restore_to_root = true

[excludes]
paths = []

[metadata]
save_package_list = false
save_enabled_services = false
save_block_devices = false
save_kernel_info = false

[notifications]
enabled = false
desktop_notifications = false

[logging]
log_dir = "{log_dir}"
log_level = "info"
"#,
        repo = repo_path.display(),
        source = source_path.display(),
        lock_file = test_root.join("ironvault.lock").display(),
        log_dir = test_root.join("logs").display(),
    );

    std::fs::write(&config_path, config).map_err(|error| {
        format!("Could not write GUI test vault config: {error}")
    })?;

    let initialized_repo = if repo_path.join("snapshots").exists() {
        false
    } else {
        run_ironvault(&["init", "--repo", &repo_path.display().to_string()])?;
        true
    };

    Ok(SetupTestVaultResult {
        repo_path: repo_path.display().to_string(),
        config_path: config_path.display().to_string(),
        source_path: source_path.display().to_string(),
        initialized_repo,
        message: if initialized_repo {
            "GUI test vault created and initialized.".to_string()
        } else {
            "GUI test vault config refreshed. Existing repo kept.".to_string()
        },
    })
}

#[tauri::command]
async fn setup_custom_vault(
    source_path: String,
    repo_path: String,
    config_path: String,
) -> Result<SetupCustomVaultResult, String> {
    let source = std::path::PathBuf::from(source_path.trim());
    let repo = std::path::PathBuf::from(repo_path.trim());
    let config = std::path::PathBuf::from(config_path.trim());

    if source.as_os_str().is_empty() {
        return Err("Choose a source folder before creating a real vault config.".to_string());
    }

    if repo.as_os_str().is_empty() {
        return Err("Choose a vault repo path before creating a real vault config.".to_string());
    }

    if config.as_os_str().is_empty() {
        return Err("Choose a config file path before creating a real vault config.".to_string());
    }

    if !source.exists() {
        return Err(format!("Source folder does not exist: {}", source.display()));
    }

    if !source.is_dir() {
        return Err(format!("Source path is not a folder: {}", source.display()));
    }

    if let Some(parent) = config.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!("Could not create config parent folder {}: {error}", parent.display())
        })?;
    }

    if let Some(parent) = repo.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!("Could not create repo parent folder {}: {error}", parent.display())
        })?;
    }

    let lock_file = config
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("ironvault.lock");

    let log_dir = config
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("logs");

    let config_body = format!(
        r#"[repo]
path = "{repo}"

[backup]
sources = ["{source}"]
one_file_system = true
follow_symlinks = false
parallelism = "auto"
chunk_size = "4MiB"
compression = "zstd"
compression_level = 3

[retention]
keep_hourly = 24
keep_daily = 7
keep_weekly = 4
keep_monthly = 6
keep_yearly = 2

[safety]
require_root = false
require_repo_mount = false
repo_mount_point = "{repo}"
minimum_free_space_gb = 0
prevent_if_pacman_running = false
prevent_if_pacman_lock_exists = false
prevent_if_backup_already_running = false
lock_file = "{lock_file}"
never_restore_to_root = true

[excludes]
paths = []

[metadata]
save_package_list = false
save_enabled_services = false
save_block_devices = false
save_kernel_info = false

[notifications]
enabled = false
desktop_notifications = false

[logging]
log_dir = "{log_dir}"
log_level = "info"
"#,
        repo = repo.display(),
        source = source.display(),
        lock_file = lock_file.display(),
        log_dir = log_dir.display(),
    );

    std::fs::write(&config, config_body).map_err(|error| {
        format!("Could not write real vault config {}: {error}", config.display())
    })?;

    let initialized_repo = if repo.join("snapshots").exists() {
        false
    } else {
        run_ironvault(&["init", "--repo", &repo.display().to_string()])?;
        true
    };

    Ok(SetupCustomVaultResult {
        repo_path: repo.display().to_string(),
        config_path: config.display().to_string(),
        source_path: source.display().to_string(),
        initialized_repo,
        message: if initialized_repo {
            "Real vault config created and repository initialized.".to_string()
        } else {
            "Real vault config refreshed. Existing repository kept.".to_string()
        },
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
            setup_test_vault,
            setup_custom_vault,
            restore_plan,
            restore_snapshot,
            get_info,
            verify_repository,
        ])
        .run(tauri::generate_context!())
        .expect("error running IronVault Tauri application");
}
