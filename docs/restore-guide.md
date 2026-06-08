# IronVault Restore Guide

This guide explains how to restore files from IronVault backups.

## Safety First

**IronVault will NEVER allow restoring directly to `/`**. All restores must specify a target directory.

## Basic Restore

To restore the latest snapshot:

```bash
# Create a restore target
mkdir /tmp/restore-target

# Run the restore
sudo ironvault restore --snapshot latest --target /tmp/restore-target --repo /mnt/backups/ironvault
```

## Restore to a Different Location

To restore to a different location:

```bash
sudo ironvault restore --snapshot latest --target /mnt/recovery/ironvault-restore --repo /mnt/backups/ironvault
```

## Preview Before Restoring

Generate a restore plan first to see what will be restored:

```bash
sudo ironvault restore-plan --snapshot latest --target /tmp/restore-preview --repo /mnt/backups/ironvault
```

## Restoring Specific Snapshots

List available snapshots:

```bash
sudo ironvault snapshots --repo /mnt/backups/ironvault
```

Then restore a specific one:

```bash
sudo ironvault restore --snapshot auto-2026-06-07_02-30-00 --target /tmp/restore --repo /mnt/backups/ironvault
```

## Restoring Individual Files

For selective file restoration, you can extract specific files from a snapshot:

```bash
# This feature is planned for v0.2
# Currently, full snapshot restore is the only option
```

## Restoring System State

To restore a system from scratch:

1. Boot from an Arch install media
2. Mount your root filesystem
3. Mount your backup repository
4. Run the restore to a temporary location
5. Copy restored files to the mounted system

```bash
# From the install media
sudo mount /dev/sda1 /mnt
sudo mount /dev/sdb1 /mnt/backups

sudo ironvault restore --snapshot latest --target /mnt --repo /mnt/backups/ironvault

# Then chroot and reinstall bootloader, etc.
```

## Troubleshooting

### "Cannot restore to root filesystem"

This error means you tried to restore to `/`. Always specify a target directory:

```bash
# Wrong
sudo ironvault restore --snapshot latest --repo /mnt/backups/ironvault

# Correct
sudo ironvault restore --snapshot latest --target /mnt/restore --repo /mnt/backups/ironvault
```

### "Repository not found"

Make sure the repository path is correct and mounted:

```bash
ls -la /mnt/backups/ironvault/
```

### Permission denied

IronVault requires root privileges for most operations. Use `sudo`:

```bash
sudo ironvault restore --snapshot latest --target /tmp/restore --repo /mnt/backups/ironvault
```