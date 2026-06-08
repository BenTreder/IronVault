use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_test_root() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".ironvault-cli-test-runs")
        .join(format!(
            "ironvault-cli-mvp-test-{}-{}",
            std::process::id(),
            nanos
        ))
}

fn count_files(path: &std::path::Path) -> usize {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_files(&path);
            } else if path.is_file() {
                count += 1;
            }
        }
    }

    count
}

#[test]
fn mvp_backup_snapshot_and_restore_round_trip() {
    let bin = env!("CARGO_BIN_EXE_ironvault");
    let root = unique_test_root();

    let source = root.join("source");
    let subdir = source.join("subdir");
    let repo = root.join("repo");
    let restore = root.join("restore");
    let logs = root.join("logs");
    let config_path = root.join("ironvault.test.toml");

    fs::create_dir_all(&subdir).unwrap();
    fs::create_dir_all(&logs).unwrap();

    fs::write(source.join("test.txt"), "hello ironvault\n").unwrap();
    fs::write(subdir.join("second.txt"), "second file\n").unwrap();
    fs::write(subdir.join("duplicate.txt"), "hello ironvault\n").unwrap();

    let source_test_file = source.join("test.txt");
    fs::set_permissions(&source_test_file, fs::Permissions::from_mode(0o640)).unwrap();

    let known_mtime = filetime::FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&source_test_file, known_mtime).unwrap();

    let config = format!(
        r#"[repo]
path = "{repo}"

[backup]
sources = ["{source}"]
one_file_system = true
follow_symlinks = false
parallelism = "4"
chunk_size = "8B"
compression = "zstd"
compression_level = 10

[retention]
keep_hourly = 6
keep_daily = 7
keep_weekly = 4
keep_monthly = 6
keep_yearly = 1

[safety]
require_root = false
require_repo_mount = false
repo_mount_point = "{root}"
minimum_free_space_gb = 1
prevent_if_pacman_running = false
prevent_if_pacman_lock_exists = false
prevent_if_backup_already_running = true
lock_file = "{root}/ironvault.lock"
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
log_dir = "{logs}"
log_level = "info"
"#,
        repo = repo.display(),
        source = source.display(),
        root = root.display(),
        logs = logs.display(),
    );

    fs::write(&config_path, config).unwrap();

    let init = Command::new(bin)
        .args(["init", "--repo"])
        .arg(&repo)
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let backup = Command::new(bin)
        .args(["backup", "--config"])
        .arg(&config_path)
        .output()
        .unwrap();
    assert!(
        backup.status.success(),
        "backup failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&backup.stdout),
        String::from_utf8_lossy(&backup.stderr)
    );

    let backup_stdout = String::from_utf8_lossy(&backup.stdout);
    assert!(
        backup_stdout.contains("Files: 3"),
        "backup stdout was:\n{}",
        backup_stdout
    );

    let snapshots = Command::new(bin)
        .args(["snapshots", "--repo"])
        .arg(&repo)
        .output()
        .unwrap();
    assert!(snapshots.status.success(), "snapshots failed");
    let snapshots_stdout = String::from_utf8_lossy(&snapshots.stdout);
    assert!(
        snapshots_stdout.contains("3 files"),
        "snapshots stdout was:\n{}",
        snapshots_stdout
    );

    let restore_cmd = Command::new(bin)
        .args(["restore", "--repo"])
        .arg(&repo)
        .args(["--snapshot", "latest", "--target"])
        .arg(&restore)
        .output()
        .unwrap();
    assert!(
        restore_cmd.status.success(),
        "restore failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&restore_cmd.stdout),
        String::from_utf8_lossy(&restore_cmd.stderr)
    );

    assert_eq!(
        fs::read_to_string(restore.join("source/test.txt")).unwrap(),
        "hello ironvault\n"
    );
    assert_eq!(
        fs::read_to_string(restore.join("source/subdir/second.txt")).unwrap(),
        "second file\n"
    );
    assert_eq!(
        fs::read_to_string(restore.join("source/subdir/duplicate.txt")).unwrap(),
        "hello ironvault\n"
    );

    let restored_test_file = restore.join("source/test.txt");
    let restored_metadata = fs::metadata(&restored_test_file).unwrap();

    #[cfg(unix)]
    assert_eq!(restored_metadata.permissions().mode() & 0o777, 0o640);

    let restored_mtime = filetime::FileTime::from_last_modification_time(&restored_metadata);
    assert_eq!(restored_mtime.unix_seconds(), 1_700_000_000);

    let object_count = count_files(&repo.join("objects"));
    assert_eq!(
        object_count, 4,
        "expected 4 unique chunk objects, got {}",
        object_count
    );

    let _ = fs::remove_dir_all(root);
}
