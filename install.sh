#!/bin/bash

set -e

echo "CoreDump Activity Tracker - Installation"
echo "========================================"
echo

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: This daemon only supports Linux"
    exit 1
fi

# Check for required tools
echo "Checking dependencies..."
if ! command -v xdotool &> /dev/null; then
    echo "Error: xdotool is not installed"
    echo "Install it with: sudo apt-get install xdotool"
    exit 1
fi
echo "✓ xdotool found"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed"
    echo "Install from: https://rustup.rs"
    exit 1
fi
echo "✓ Cargo found"

# Build the daemon
echo
echo "Building daemon..."
cargo build --release

# Create config directory
CONFIG_DIR="$HOME/.config/coredump"
mkdir -p "$CONFIG_DIR"

# Create config file if it doesn't exist
CONFIG_FILE="$CONFIG_DIR/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo
    read -p "Enter your CoreDump private key: " PRIVATE_KEY
    echo "private_key = \"$PRIVATE_KEY\"" > "$CONFIG_FILE"
    echo "✓ Config file created at $CONFIG_FILE"
else
    echo "✓ Config file already exists at $CONFIG_FILE"
fi

# Install binary
echo
echo "Installing binary..."
cargo install --path .
echo "✓ Binary installed to ~/.cargo/bin/coredump-daemon"

# Setup systemd service
echo
echo "Setting up systemd service..."
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"
cp coredump-daemon.service "$SERVICE_DIR/"
echo "✓ Service file copied"

# Reload systemd and enable service
systemctl --user daemon-reload
systemctl --user enable coredump-daemon.service
systemctl --user start coredump-daemon.service

echo
echo "========================================"
echo "Installation complete!"
echo
echo "The daemon is now running in the background."
echo
echo "Useful commands:"
echo "  - Check status: systemctl --user status coredump-daemon"
echo "  - View logs:    journalctl --user -u coredump-daemon -f"
echo "  - Stop:         systemctl --user stop coredump-daemon"
echo "  - Restart:      systemctl --user restart coredump-daemon"
echo
echo "Config file location: $CONFIG_FILE"
echo
