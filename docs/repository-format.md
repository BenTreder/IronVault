# IronVault Repository Format

This document describes the on-disk layout of an IronVault repository.

## Directory Structure

```
IronVaultRepo/
├── repo.yaml           # Repository metadata
├── objects/            # Deduplicated chunks
│   ├── ab/
│   │   └── cdef1234567890abcdef1234567890abcdef12.chunk
│   └── ...
├── snapshots/          # Snapshot manifests
│   ├── auto-2026-06-08_10-30-00.snapshot.json
│   └── latest.snapshot.json -> auto-2026-06-08_10-30-00.snapshot.json
├── indexes/            # SQLite indexes
│   └── chunks.sqlite   # Chunk lookup database
├── locks/              # Lock files
├── logs/               # Operation logs
│   └── ironvault.log
└── metadata/           # System metadata
    ├── packages.txt
    ├── services.txt
    └── system.yaml
```

## Object Storage

Chunks are stored using a two-level directory structure based on the BLAKE3 hash:

- First 2 hex characters of the hash form the subdirectory
- Remaining 30 characters form the filename
- Files are compressed with zstd before storage

Example: A chunk with hash `abcdef1234567890...` is stored at:
```
objects/ab/cdef1234567890abcdef1234567890abcdef12.chunk
```

## Snapshot Format

Snapshots are stored as JSON files with the following structure:

```json
{
  "id": "uuid-v4",
  "name": "auto-2026-06-08_10-30-00",
  "hostname": "hostname",
  "created_at": "2026-06-08T10:30:00Z",
  "source_paths": ["/"],
  "files": [
    {
      "path": "/home/user/file.txt",
      "size": 1024,
      "permissions": 644,
      "uid": 1000,
      "gid": 1000,
      "mtime": "2026-06-08T10:00:00Z",
      "chunk_hashes": ["abc123...", "def456..."],
      "compression": "zstd"
    }
  ],
  "directories": [...],
  "symlinks": [...],
  "metadata": {...}
}
```

## Chunk Index

The SQLite database tracks all stored chunks:

```sql
CREATE TABLE chunks (
    hash TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    compressed_size INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    source_paths TEXT NOT NULL,
    files_count INTEGER NOT NULL,
    size INTEGER NOT NULL
);

CREATE TABLE snapshot_chunks (
    snapshot_id TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    PRIMARY KEY (snapshot_id, chunk_hash)
);
```

## Version History

| Version | Date       | Changes                          |
|---------|------------|----------------------------------|
| 1.0.0   | 2026-06-08 | Initial repository format        |