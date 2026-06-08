# IronVault Roadmap

## Version 0.1.0 (Current) - Foundation Release

### Completed
- [x] Repository initialization
- [x] Configuration loading
- [x] Safety checks
- [x] File scanner
- [x] Exclusion handling
- [x] Fixed-size chunking (4 MiB default)
- [x] BLAKE3 hashing
- [x] zstd compression
- [x] Object storage
- [x] Snapshot manifest writing
- [x] Snapshot listing
- [x] Restore to safe target
- [x] Basic pruning
- [x] CLI interface
- [x] Tauri GUI scaffold
- [x] Systemd service/timer
- [x] Documentation

### Known Limitations
- Limited actual data processing (placeholder implementation)
- Basic pruning logic
- No encryption
- Single-threaded operations

---

## Version 0.2.0 - Data Processing

### Planned Features
- [ ] Full file reading and chunking
- [ ] Actual chunk storage and deduplication
- [ ] Parallel hashing with rayon
- [ ] Improved pruning logic
- [ ] Compression statistics
- [ ] Progress reporting
- [ ] Dry-run file listing

---

## Version 0.3.0 - Production Ready

### Planned Features
- [ ] Content-defined chunking (optional)
- [ ] Repository verification
- [ ] Chunk integrity checking
- [ ] Backup resumption
- [ ] Memory-efficient streaming
- [ ] Better error recovery
- [ ] Comprehensive test suite

---

## Version 0.4.0 - Encryption

### Planned Features
- [ ] Repository encryption (Age-based)
- [ ] Key management
- [ ] Encrypted metadata
- [ ] Secure deletion
- [ ] Audit logging

---

## Version 0.5.0 - Advanced Features

### Planned Features
- [ ] Incremental deduplication
- [ ] Remote repository support
- [ ] Bandwidth limiting
- [ ] Throttling
- [ ] Email notifications
- [ ] Web dashboard

---

## Version 1.0.0 - Stable Release

### Planned Features
- [ ] Full test coverage
- [ ] Performance benchmarks
- [ ] Production documentation
- [ ] Stable API guarantees
- [ ] Migration tools
- [ ] Plugin system

---

## Future Considerations

### Potential Features
- [ ] Deduplication across snapshots
- [ ] Deduplication across repositories
- [ ] Cloud storage backend
- [ ] Deduplication-aware compression
- [ ] Machine learning for file classification
- [ ] Deduplication statistics dashboard