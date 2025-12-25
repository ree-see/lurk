#!/bin/bash
set -e

echo "Installing lurk..."
echo

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building release binary..."
cd "$PROJECT_DIR"
cargo build --release

echo "Copying binary to /usr/local/bin/lurk..."
sudo cp target/release/lurk /usr/local/bin/
sudo chmod +x /usr/local/bin/lurk

echo "Creating data directory..."
mkdir -p ~/.lurk/logs

echo "Installing LaunchAgent..."
PLIST_SRC="$PROJECT_DIR/launchd/com.user.lurk.plist"
PLIST_DST="$HOME/Library/LaunchAgents/com.user.lurk.plist"

if [ -f "$PLIST_DST" ]; then
    echo "Unloading existing LaunchAgent..."
    launchctl unload "$PLIST_DST" 2>/dev/null || true
fi

cp "$PLIST_SRC" "$PLIST_DST"

echo "Loading LaunchAgent..."
launchctl load "$PLIST_DST"

echo
echo "Installation complete!"
echo
echo "Next steps:"
echo "1. Run: lurk daemon"
echo "   (This will prompt for Input Monitoring permission)"
echo "2. Grant permission in System Settings -> Privacy & Security -> Input Monitoring"
echo "3. Restart: launchctl unload ~/Library/LaunchAgents/com.user.lurk.plist && launchctl load ~/Library/LaunchAgents/com.user.lurk.plist"
echo
echo "Commands:"
echo "  lurk stats          - Show statistics"
echo "  lurk export -o x.csv - Export to CSV"
echo "  lurk check-permission - Check permission status"
