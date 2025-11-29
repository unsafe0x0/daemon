#!/bin/bash

echo "CoreDump Activity Tracker - Uninstallation"
echo "==========================================="
echo

# Stop and disable the service
echo "Stopping daemon..."
systemctl --user stop coredump-daemon.service 2>/dev/null || true
systemctl --user disable coredump-daemon.service 2>/dev/null || true
echo "✓ Service stopped and disabled"

# Remove the service file
SERVICE_FILE="$HOME/.config/systemd/user/coredump-daemon.service"
if [ -f "$SERVICE_FILE" ]; then
    rm "$SERVICE_FILE"
    echo "✓ Service file removed"
fi

# Reload systemd
systemctl --user daemon-reload 2>/dev/null || true

# Remove the binary
if command -v cargo &> /dev/null; then
    cargo uninstall coredump 2>/dev/null || true
    echo "✓ Binary uninstalled"
fi

# Ask about config removal
CONFIG_DIR="$HOME/.config/coredump"
if [ -d "$CONFIG_DIR" ]; then
    echo
    read -p "Remove configuration directory? (y/N): " REMOVE_CONFIG
    if [[ "$REMOVE_CONFIG" =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        echo "✓ Configuration removed"
    else
        echo "Configuration kept at: $CONFIG_DIR"
    fi
fi

echo
echo "==========================================="
echo "Uninstallation complete!"
echo
echo "The CoreDump daemon has been removed from your system."
echo
