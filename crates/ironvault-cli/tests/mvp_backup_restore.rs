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

fn first_file(path: &std::path::Path) -> Option<std::path::PathBuf> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                return Some(path);
            }

            if path.is_dir() {
                if let Some(found) = first_file(&path) {
                    return Some(found);
                }
            }
        }
    }

    None
}

#[test]
fn mvp_backup_snapshot_and_restore_round_trip() {
    let bin = env!("CARGO_BIN_EXE_ironvault");
    let root = unique_test_root();

    let source = root.join("source");
    let subdir = source.join("subdir");
    let empty_dir = source.join("empty-dir");
    let repo = root.join("repo");
    let restore = root.join("restore");
    let logs = root.join("logs");
    let config_path = root.join("ironvault.test.toml");

    fs::create_dir_all(&subdir).unwrap();
    fs::create_dir_all(&empty_dir).unwrap();
    fs::create_dir_all(&logs).unwrap();

    fs::write(source.join("test.txt"), "hello ironvault\n").unwrap();
    fs::write(subdir.join("second.txt"), "second file\n").unwrap();
    fs::write(subdir.join("duplicate.txt"), "hello ironvault\n").unwrap();

    let source_test_file = source.join("test.txt");
    fs::set_permissions(&source_test_file, fs::Permissions::from_mode(0o640)).unwrap();

    let known_mtime = filetime::FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&source_test_file, known_mtime).unwrap();

    #[cfg(unix)]
    {
        fs::set_permissions(&empty_dir, fs::Permissions::from_mode(0o750)).unwrap();
        filetime::set_file_mtime(&empty_dir, known_mtime).unwrap();
        std::os::unix::fs::symlink("test.txt", source.join("link-to-test")).unwrap();
    }

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

    fs::create_dir_all(restore.join("source")).unwrap();
    fs::write(
        restore.join("source/test.txt"),
        "existing file should stay safe\n",
    )
    .unwrap();

    let restore_conflict_cmd = Command::new(bin)
        .args(["restore", "--repo"])
        .arg(&repo)
        .args(["--snapshot", "latest", "--target"])
        .arg(&restore)
        .output()
        .unwrap();
    assert!(
        !restore_conflict_cmd.status.success(),
        "restore should refuse to overwrite existing files\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&restore_conflict_cmd.stdout),
        String::from_utf8_lossy(&restore_conflict_cmd.stderr)
    );
    assert_eq!(
        fs::read_to_string(restore.join("source/test.txt")).unwrap(),
        "existing file should stay safe\n"
    );

    fs::remove_dir_all(&restore).unwrap();

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

    let restored_empty_dir = restore.join("source/empty-dir");
    assert!(
        restored_empty_dir.is_dir(),
        "empty directory was not restored"
    );

    #[cfg(unix)]
    {
        let restored_empty_dir_metadata = fs::metadata(&restored_empty_dir).unwrap();
        assert_eq!(
            restored_empty_dir_metadata.permissions().mode() & 0o777,
            0o750
        );

        let restored_empty_dir_mtime =
            filetime::FileTime::from_last_modification_time(&restored_empty_dir_metadata);
        assert_eq!(restored_empty_dir_mtime.unix_seconds(), 1_700_000_000);

        let restored_link = restore.join("source/link-to-test");
        let link_metadata = fs::symlink_metadata(&restored_link).unwrap();
        assert!(
            link_metadata.file_type().is_symlink(),
            "symlink was not restored"
        );
        assert_eq!(
            fs::read_link(&restored_link).unwrap(),
            std::path::PathBuf::from("test.txt")
        );
        assert_eq!(
            fs::read_to_string(&restored_link).unwrap(),
            "hello ironvault\n"
        );
    }

    let object_count = count_files(&repo.join("objects"));
    assert_eq!(
        object_count, 4,
        "expected 4 unique chunk objects, got {}",
        object_count
    );

    let verify_good = Command::new(bin)
        .args(["verify", "--repo"])
        .arg(&repo)
        .output()
        .unwrap();
    assert!(
        verify_good.status.success(),
        "verify failed before corruption\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&verify_good.stdout),
        String::from_utf8_lossy(&verify_good.stderr)
    );
    let verify_good_stdout = String::from_utf8_lossy(&verify_good.stdout);
    assert!(
        verify_good_stdout.contains("Repository is valid"),
        "verify stdout before corruption was:\n{}",
        verify_good_stdout
    );

    let corrupt_chunk =
        first_file(&repo.join("objects")).expect("expected at least one chunk object");
    fs::remove_file(&corrupt_chunk).unwrap();

    let verify_bad = Command::new(bin)
        .args(["verify", "--repo"])
        .arg(&repo)
        .output()
        .unwrap();
    assert!(
        verify_bad.status.success(),
        "verify command should report errors without crashing\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&verify_bad.stdout),
        String::from_utf8_lossy(&verify_bad.stderr)
    );
    let verify_bad_stdout = String::from_utf8_lossy(&verify_bad.stdout);
    assert!(
        verify_bad_stdout.contains("Repository has errors"),
        "verify stdout after corruption was:\n{}",
        verify_bad_stdout
    );

    let _ = fs::remove_dir_all(root);
}
