# Lurk

A local-only keystroke logger for custom keyboard design analysis.

## Project Structure

```
src/
├── main.rs              - CLI entry point and daemon runner
├── models/
│   ├── event.rs         - KeystrokeEvent data structures
│   └── keycode.rs       - macOS keycode mapping
├── storage/
│   └── database.rs      - SQLite database operations
├── daemon/
│   ├── permissions.rs   - macOS Input Monitoring permission checks
│   ├── app_tracker.rs   - Active application detection
│   └── event_monitor.rs - Keyboard event capture via rdev
└── cli/
    ├── export.rs        - CSV/JSON export
    └── stats.rs         - Statistics display
```

## Building

```bash
cargo build --release
```

## Installation

```bash
./scripts/install.sh
```

## Usage

```bash
lurk daemon              # Run capture daemon
lurk stats               # Show statistics
lurk export -o data.csv  # Export to CSV
lurk check-permission    # Check Input Monitoring permission
```

## Data Location

All data stored in `~/.lurk/`:
- `events.db` - SQLite database
- `logs/` - Daemon logs

## LaunchAgent

The daemon runs as a LaunchAgent at `~/Library/LaunchAgents/com.user.lurk.plist`.

Control with:
```bash
launchctl load ~/Library/LaunchAgents/com.user.lurk.plist
launchctl unload ~/Library/LaunchAgents/com.user.lurk.plist
launchctl list | grep lurk
```
