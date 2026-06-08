#!/bin/bash
# IronVault Installation Script
# Run as root

set -e

echo "=== IronVault Installation Script ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Install dependencies
echo "Installing dependencies..."
pacman -Sy --noconfirm rustup

# Add rustup to PATH
source /etc/profile.d/rustup.sh 2>/dev/null || true

# Install Tauri prerequisites
pacman -Sy --noconfirm webkit2gtk gtk3 libappindicator-gtk3 openssl curl
    - webkit2gtk
    - gtk3
    - libappindicator-gtk3
    - openssl
    - curl

# Build IronVault
echo "Building IronVault..."
cd /opt/IronVault
cargo build --release

# Create directories
echo "Creating directories..."
mkdir -p /etc/ironvault
mkdir -p /var/log/ironvault
mkdir -p /run/ironvault

# Copy config
echo "Installing configuration..."
cp config/ironvault.example.toml /etc/ironvault/config.toml

# Install binary
echo "Installing binary..."
install -m 755 target/release/ironvault /usr/bin/ironvault

# Install systemd files
echo "Installing systemd service..."
cp systemd/ironvault-backup.service /etc/systemd/system/
cp systemd/ironvault-backup.timer /etc/systemd/system/
systemctl daemon-reload

# Set permissions
chown -R root:root /etc/ironvault
chown -R root:root /var/log/ironvault
chmod 755 /var/log/ironvault

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "1. Edit /etc/ironvault/config.toml"
echo "2. Create a repository: sudo ironvault init --repo /mnt/backups/ironvault"
echo "3. Enable the timer: sudo systemctl enable --now ironvault-backup.timer"
echo ""
echo "For more information, see: man ironvault(1)"