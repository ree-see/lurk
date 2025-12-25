#!/bin/bash
set -e

echo "Uninstalling lurk..."
echo

PLIST="$HOME/Library/LaunchAgents/com.user.lurk.plist"

if [ -f "$PLIST" ]; then
    echo "Stopping daemon..."
    launchctl unload "$PLIST" 2>/dev/null || true
    
    echo "Removing LaunchAgent..."
    rm -f "$PLIST"
fi

if [ -f "/usr/local/bin/lurk" ]; then
    echo "Removing binary..."
    sudo rm -f /usr/local/bin/lurk
fi

echo
read -p "Remove data directory (~/.lurk)? This will delete all recorded data. [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Removing data directory..."
    rm -rf ~/.lurk
    echo "Data removed."
else
    echo "Data preserved at ~/.lurk"
fi

echo
echo "Uninstall complete!"
