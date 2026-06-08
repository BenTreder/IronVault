# IronVault Safety Model

IronVault is designed with safety as a primary concern. This document outlines the safety mechanisms in place.

## Root Privileges

Most backup operations require root privileges because:

- Reading system files requires elevated permissions
- Setting file metadata (ownership, permissions) requires root
- Accessing block devices requires root

**Configuration**: `safety.require_root = true`

## Repository Mount Validation

IronVault verifies that the repository is mounted before performing operations.

**Configuration**: `safety.require_repo_mount = true`
**Mount point**: `safety.repo_mount_point = "/mnt/BorgBackup"`

## Free Space Checks

Before starting a backup, IronVault checks available disk space.

**Configuration**: `safety.minimum_free_space_gb = 25`

If free space is below this threshold, the backup will not run.

## Lock Files

IronVault uses lock files to prevent concurrent operations.

**Lock file**: `safety.lock_file = "/run/ironvault.lock"`

If a lock file exists and the process is still running, IronVault will refuse to proceed.

## Pacman Protection

IronVault can detect and wait for pacman operations:

**Configuration**:
- `safety.prevent_if_pacman_running = true`
- `safety.prevent_if_pacman_lock_exists = true`

## Repository Path Validation

IronVault ensures the repository is not inside source directories and vice versa.

This prevents:
- Backing up the repository into itself
- Accidentally including repository data in backups

## Restore Safety

### Target Validation

- **Never restores to `/`**
- Target directory must exist
- Creates parent directories as needed

### Confirmation

Before restoring, IronVault:
1. Generates a restore plan
2. Displays what will be restored
3. Requires explicit confirmation

## Dangerous Path Protection

IronVault automatically excludes these paths:

| Path | Reason |
|------|--------|
| `/proc` | Virtual filesystem |
| `/sys` | Virtual filesystem |
| `/dev` | Device files |
| `/run` | Runtime data |
| `/tmp` | Temporary files |
| `/var/tmp` | Temporary files |
| `/mnt` | Other mounts |
| `/media` | Removable media |

## Pseudo-Filesystem Protection

IronVault detects and skips:

- `proc` filesystem
- `sysfs` filesystem
- `devtmpfs`
- `tmpfs`
- `cgroup`
- `mqueue`

## Chunk Deletion Protection

During pruning, IronVault:

1. Checks which snapshots reference each chunk
2. Only deletes chunks that are unreferenced by ANY snapshot
3. Never deletes chunks still needed by newer snapshots

## Logging

All safety-relevant actions are logged:

- Start/end times
- Source paths
- Exclusions
- Files scanned
- Errors and warnings

## Error Handling

IronVault uses a layered error system:

- **Recoverable errors**: May be retried (e.g., lock contention)
- **Fatal errors**: Stop the operation (e.g., permission denied)
- **Safety errors**: Prevent dangerous operations

## Best Practices

1. **Test restores**: Periodically test restoring to a temporary directory
2. **Monitor logs**: Check `/var/log/ironvault/ironvault.log` regularly
3. **Verify backups**: Use `ironvault verify` to check repository integrity
4. **Update config**: Review `config/ironvault.example.toml` for new options