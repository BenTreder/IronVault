# IronVault

A premium backup program for Arch Linux - a complete, from-scratch backup engine written in Rust.

## Overview

IronVault is a deduplicating backup system that creates rolling snapshots of your system. Unlike Borg or Restic wrappers, IronVault is a complete implementation built from the ground up in Rust.

### Features

- **Deduplicated snapshots**: Files are split into chunks, hashed with BLAKE3, and stored only once
- **Rolling backups**: Incremental backups with automatic retention policies
- **Restore support**: Safe, targeted restores with restoration planning
- **Pruning**: Automatic cleanup of old snapshots while preserving referenced chunks
- **Safety checks**: Prevents dangerous operations, validates prerequisites
- **Systemd integration**: Automatic daily backups via systemd timers
- **Desktop GUI**: Tauri-based frontend for easy management

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        IronVault                              │
├─────────────────────────────────────────────────────────────┤
│  CLI (ironvault-cli)     Desktop GUI (Tauri)                │
│       │                          │                          │
│       └──────────┬───────────────┘                          │
│                  │                                          │
│  ┌─────────────▼────────────────────────────────┐         │
│  │           ironvault-core                      │         │
│  │  ┌──────────┐ ┌──────────┐ ┌─────────────┐ │         │
│  │  │ Scanner  │ │ Chunker  │ │ Repository  │ │         │
│  │  │          │ │          │ │             │ │         │
│  │  │ Hasher   │ │Metadata  │ │Prune        │ │         │
│  │  └──────────┘ └──────────┘ └─────────────┘ │         │
│  └─────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Repository     │
                    │  (Local FS)     │
                    └─────────────────┘
```

## Quick Start

### Build

```bash
cargo build --release
```

### Initialize a Repository

```bash
sudo ./target/release/ironvault init --repo /mnt/backups/ironvault
```

### Create a Backup

```bash
sudo ./target/release/ironvault backup --config /etc/ironvault/config.toml
```

### List Snapshots

```bash
sudo ./target/release/ironvault snapshots --repo /mnt/backups/ironvault
```

### Restore

```bash
sudo ./target/release/ironvault restore --snapshot latest --target /tmp/restore-here
```

## Project Structure

```
ironvault/
├── crates/
│   └── ironvault-core/     # Core backup engine
│   └── ironvault-cli/      # Command-line interface
│   └── ironvault-daemon/   # Background daemon
├── frontend/
│   └── ironvault-gui/      # Tauri desktop frontend
├── config/
│   └── ironvault.example.toml
├── systemd/
│   ├── ironvault-backup.service
│   └── ironvault-backup.timer
├── docs/
│   └── repository-format.md
└── scripts/
    └── install.sh
```

## Repository Format

IronVault stores data in a structured directory:

```
IronVaultRepo/
├── repo.toml              # Repository configuration
├── objects/               # Deduplicated chunks
│   ├── ab/cdef123.chunk   # Stored by first 2 chars of hash
│   └── ...
├── snapshots/             # Snapshot manifests
├── indexes/               # SQLite index of chunks
├── locks/                 # Lock files
├── logs/                  # Operation logs
└── metadata/              # System metadata
```

## Configuration

See `config/ironvault.example.toml` for the full configuration reference.

## Safety Model

IronVault is designed with safety as a primary concern:

1. **Root required**: Most operations require root privileges
2. **Mount validation**: Verifies repository is mounted before operations
3. **Space checks**: Ensures minimum free space before backup
4. **Lock files**: Prevents concurrent operations
5. **Dangerous path protection**: Never restores to `/`
6. **Pseudo-filesystem exclusion**: Automatically excludes `/proc`, `/sys`, etc.

## Documentation

- [Repository Format](docs/repository-format.md)
- [Restore Guide](docs/restore-guide.md)
- [Safety Model](docs/safety-model.md)
- [Roadmap](docs/roadmap.md)

## Requirements

- Rust 1.75+
- SQLite 3
- zstd
- Tauri (for frontend, optional)

## License

GPL v3 - See LICENSE for details.

## Contributing

Contributions welcome! Please read our contributing guidelines.