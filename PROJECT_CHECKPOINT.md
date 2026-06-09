# IronVault Project Checkpoint

Date: 2026-06-08
Status: Core backup engine and CLI are working through Phase 9.

IronVault is a custom Rust backup tool focused on safe local backups, restore safety, repository verification, and a future premium desktop GUI.

## Current features

- Repository initialization
- File backup
- Snapshot creation
- Snapshot listing
- Chunk-based storage
- Chunk deduplication
- zstd compression
- File restore
- Directory restore
- Symlink restore
- File permission preservation on Linux
- File modified time preservation
- Directory permission preservation on Linux
- Directory modified time preservation
- Restore path traversal protection
- Symlink target safety checks
- Restore overwrite refusal by default
- Restore-plan conflict reporting
- Repository verification
- Missing chunk detection
- CLI wording polish with IronVault personality
- Restore-plan JSON output for GUI and automation use
- Safe restore skip mode with --if-exists skip
- GUI-ready JSON outputs for info, snapshots, verify, and restore-plan

## Current safety guarantees

- IronVault refuses to restore to `/`.
- IronVault rejects absolute snapshot paths.
- IronVault rejects parent traversal snapshot paths.
- IronVault rejects unsafe symlink targets.
- IronVault refuses to overwrite existing restore targets by default.
- IronVault checks chunk objects during repository verification.
- IronVault reports restore conflicts before restore execution.

## Current test proof

The main smoke test proves:

- init works
- backup works
- snapshot listing works
- restore-plan detects conflicts
- restore refuses overwrite
- clean restore succeeds
- file contents restore correctly
- duplicate chunks dedupe correctly
- file metadata restores correctly
- empty directories restore correctly
- symlinks restore correctly
- verify passes before corruption
- verify reports errors after removing a chunk

## Product voice

IronVault should sound serious, trustworthy, and safe, but not boring.

Good phrases:

- Backup sealed.
- Vault check passed.
- Vault door closed.
- Every vault piece is accounted for.
- No guessing, no stomping, no chaos.

Avoid:

- probably fine
- oopsie
- maybe safe
- your files might be safe

## Known limits

- Encryption is not implemented yet.
- Restore overwrite modes are not implemented yet.
- GUI is not production-ready yet.
- Scheduling is not implemented yet.
- Rich JSON output for GUI use is not implemented yet.
- Verification does not yet fully report orphaned chunks.
- Cross-platform behavior is focused mostly on Linux.

## Recommended next phase

Phase 15 should start the Tauri GUI bridge.

The CLI now has enough JSON output for the desktop app to read real project state without scraping human text.

Available GUI-friendly commands:

- ironvault info --repo ./repo --json
- ironvault snapshots --repo ./repo --json
- ironvault verify --repo ./repo --json
- ironvault restore-plan --repo ./repo --snapshot latest --target ./restore --json

Restore safety command now available:

- ironvault restore --repo ./repo --snapshot latest --target ./restore --if-exists skip

## Quality gate

Before merging a phase, run:

cargo fmt
cargo check -p ironvault-core -p ironvault-cli
cargo test -p ironvault-core -p ironvault-cli
cargo test -p ironvault-cli --test mvp_backup_restore

## Phase 30 GUI checkpoint

Current GUI status after Phase 30:

- Dashboard reads live vault info and verify status
- Settings can save vault path
- Settings can create a safe demo vault
- Settings can create a real backup setup
- Settings blocks unsafe backup storage locations inside the source folder
- Backup page previews the selected IronVault settings file
- Backup page shows folder to back up, storage folder, file counts, and readiness
- Backup page requires typing SEAL before running backup
- Snapshots page shows snapshot cards directly with counts and restore guidance
- Restore page previews restore plans
- Restore page explains where restored files go
- Restore page requires typing RESTORE before running restore
- Restore runs with overwrite refusal
- CLI/core tests pass
- Frontend build passes
- Tauri backend check passes

Next finalization work:

- Final release checklist
- Packaging/build decision
- Native file picker decision
- Final icon replacement
- Release candidate tag
