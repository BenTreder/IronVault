#!/usr/bin/env bash
set +e

cd "$(dirname "$0")/.." || exit 1

VERSION="${1:-0.1.0-rc1}"
RELEASE_DIR="$PWD/release-candidates/ironvault-$VERSION"

rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

echo
echo "=== IronVault release candidate build ==="
echo "VERSION=$VERSION"
echo "RELEASE_DIR=$RELEASE_DIR"

git rev-parse --short HEAD > "$RELEASE_DIR/git-commit.txt"
git branch --show-current > "$RELEASE_DIR/git-branch.txt"

echo
echo "=== core QA ==="
cargo fmt
cargo check -p ironvault-core -p ironvault-cli
cargo test -p ironvault-core -p ironvault-cli
cargo test -p ironvault-cli --test mvp_backup_restore

echo
echo "=== build release CLI ==="
cargo build -p ironvault-cli --release
cp target/release/ironvault "$RELEASE_DIR/ironvault-cli"

echo
echo "=== frontend build ==="
cd frontend/ironvault-gui
npm run build

echo
echo "=== Tauri backend check ==="
cd src-tauri
cargo check

echo
echo "=== Tauri release build ==="
cd ..
npm run tauri:build

cd ../..

echo
echo "=== collect GUI artifacts ==="
find frontend/ironvault-gui/src-tauri/target/release \
  -maxdepth 8 \
  -type f \
  \( -name "*.AppImage" -o -name "*.deb" -o -name "*.rpm" -o -name "*.tar.gz" -o -name "ironvault-gui" \) \
  -print \
  -exec cp {} "$RELEASE_DIR/" \;

cat > "$RELEASE_DIR/RELEASE_NOTES.txt" <<NOTES
IronVault $VERSION

This release candidate includes:

- Rust backup core
- CLI backup and restore workflow
- Tauri/Vue desktop GUI
- Demo vault setup
- Real vault setup
- Unsafe backup storage guard
- Backup setup preview
- Snapshot detail cards
- Restore preview
- Guarded restore confirmation
- Restore overwrite refusal by default
- Final release checklist docs

Known limitations:

- File and folder paths are typed manually
- No scheduling yet
- No cloud backup yet
- No pruning UI yet
- No final installer polish yet
NOTES

echo
echo "=== release candidate contents ==="
find "$RELEASE_DIR" -maxdepth 2 -type f -print | sort
