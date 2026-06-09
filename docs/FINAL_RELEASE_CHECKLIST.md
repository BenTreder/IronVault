# IronVault Final Release Checklist

## Current release goal

IronVault is a local-first backup tool with a Rust core, CLI, and Tauri/Vue desktop GUI.

The current goal is a safe local backup manager that can:

- Set up a demo vault from the GUI
- Set up a real backup config from the GUI
- Block unsafe backup storage locations
- Run backups from the GUI
- Preview backup config before running
- List snapshots
- Preview restore plans
- Restore only after confirmation
- Refuse overwrite conflicts by default
- Explain where restored files go

## Safety rules

IronVault should never:

- Restore without a preview
- Restore to root
- Overwrite existing files by default
- Hide restore conflicts
- Allow the backup storage folder inside the folder being backed up
- Present a damaged repo as healthy
- Pretend a backup exists when the snapshot has zero files

## Final QA flow

Run this before tagging a release:

Run from the project root.

1. cargo fmt
2. cargo check for ironvault-core and ironvault-cli
3. cargo test for ironvault-core and ironvault-cli
4. cargo test for the MVP backup and restore test
5. frontend npm build
6. Tauri cargo check

## Manual GUI QA

Start the app from frontend/ironvault-gui with npm run tauri:dev.

Test these flows:

1. Settings, set up test vault
2. Backup, check setup
3. Backup, type SEAL, run backup
4. Dashboard, verify vault health
5. Snapshots, confirm snapshot cards show details
6. Restore, preview restore
7. Restore, type RESTORE, restore into a clean folder
8. Confirm restored file contents on disk
9. Settings, try unsafe real setup where backup storage is inside source folder
10. Confirm unsafe setup is blocked

## Known limitations

- Real packaging is not finalized yet
- Tauri bundle/installers are not the main release path yet
- File and folder picking uses typed paths, not native picker dialogs
- Restore uses typed paths, not a visual folder picker
- No scheduling yet
- No automatic pruning UI yet
- No cloud or remote backup support yet
- No encryption/password UX yet unless added later at the core level

## Good release candidate definition

A release candidate is good when:

- All automated checks pass
- Demo vault setup works
- Real vault setup works
- Backup preview works
- Backup execution works
- Snapshot details show correctly
- Restore preview works
- Restore execution restores files correctly
- Unsafe backup storage locations are blocked
- Git working tree is clean
