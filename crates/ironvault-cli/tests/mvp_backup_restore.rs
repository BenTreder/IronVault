use std::fs;
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

    let config = format!(
        r#"[repo]
path = "{repo}"

[backup]
sources = ["{source}"]
one_file_system = true
follow_symlinks = false
parallelism = "4"
chunk_size = "4MiB"
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
        backup_stdout.contains("Files: 2"),
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
        snapshots_stdout.contains("2 files"),
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

    let _ = fs::remove_dir_all(root);
}
