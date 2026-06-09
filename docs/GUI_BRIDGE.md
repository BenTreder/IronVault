# IronVault GUI Bridge

Status: Phase 14

This document tracks the CLI commands that the future Tauri desktop GUI can safely call.

The goal is simple:

- The GUI should not scrape human CLI text.
- The GUI should call commands with JSON output.
- Human wording can stay friendly.
- GUI data should stay structured and predictable.

## Current GUI-ready commands

### Repository info

Command:

    ironvault info --repo ./repo --json

Purpose:

Dashboard cards can use this for vault location, vault size, chunk count, snapshot count, and nearby free space.

Expected shape:

    {
      "path": "./repo",
      "total_size": 12345,
      "total_chunks": 4,
      "snapshot_count": 1,
      "free_space": 999999
    }

GUI usage:

- Vault location card
- Vault size card
- Snapshot count card
- Storage health area

## Snapshot list

Command:

    ironvault snapshots --repo ./repo --json

Purpose:

The GUI can show a list of snapshots without parsing human text.

Expected shape:

    {
      "snapshot_count": 1,
      "snapshots": [
        {
          "name": "auto-2026-06-08_120000",
          "files": 3,
          "directories": 3,
          "symlinks": 1,
          "total_size": 12345
        }
      ]
    }

GUI usage:

- Snapshot browser
- Restore source picker
- Recent backup list

## Repository verify

Command:

    ironvault verify --repo ./repo --json

Purpose:

The GUI can show vault health clearly.

Expected healthy shape:

    {
      "valid": true,
      "message": "Repository is valid. Every vault piece is accounted for."
    }

Expected damaged shape:

    {
      "valid": false,
      "message": "Repository has errors. The vault check found missing or damaged pieces."
    }

GUI usage:

- Vault health badge
- Dashboard warning card
- Restore safety precheck

## Restore plan

Command:

    ironvault restore-plan --repo ./repo --snapshot latest --target ./restore --json

Purpose:

The GUI can preview restore safety before running restore.

Expected shape:

    {
      "snapshot": "auto-2026-06-08_120000",
      "target": "./restore",
      "files": 3,
      "directories": 3,
      "symlinks": 1,
      "total_size": 12345,
      "conflict_count": 0,
      "safe_to_restore": true,
      "conflicts": []
    }

Conflict shape:

    {
      "source_path": "source/test.txt",
      "target_path": "./restore/source/test.txt",
      "kind": "file target already exists"
    }

GUI usage:

- Restore preview screen
- Conflict warning screen
- Safe to restore button state

## Restore execution

Default safe restore:

    ironvault restore --repo ./repo --snapshot latest --target ./restore

Default behavior:

- Refuses to overwrite existing targets.
- Stops if restore-plan has conflicts.
- This should remain the safest default.

Skip existing restore:

    ironvault restore --repo ./repo --snapshot latest --target ./restore --if-exists skip

Skip behavior:

- Existing targets stay untouched.
- Missing files are restored.
- No overwriting.
- Useful for partial restore repair.

Suggested GUI wording:

- Vault door closed. Existing targets were found.
- Restore map ready. No conflicts found.
- Skipped existing targets. IronVault only unlocked what was missing.
- No guessing, no stomping, no chaos.

## Next GUI bridge needs

Useful next additions:

- backup --json
- dry-run --json
- restore --json
- verify details with missing chunk list
- structured error output
- progress events for long-running jobs

## Safety notes

The GUI should treat these rules as product law:

- Never hide conflicts.
- Never overwrite by default.
- Never present damaged repo status as safe.
- Never run restore without a restore-plan preview first.
- Never make the user guess what happened.

IronVault should stay calm, clear, and protective.
