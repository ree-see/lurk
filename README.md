# Lurk

A local-only keystroke logger for custom keyboard design analysis. Capture your typing patterns to optimize keyboard layouts, analyze finger usage, and improve typing efficiency.

**macOS only** - Uses the Input Monitoring API to capture keystrokes system-wide.

## Features

- **Background daemon** - Runs silently as a LaunchAgent, capturing keystrokes across all applications
- **Frequency analysis** - Key frequencies, bigrams (two-key sequences), and trigrams
- **Timing analysis** - Inter-key intervals, hold durations, percentile breakdowns
- **Interactive dashboard** - TUI for visualizing typing patterns
- **Data export** - CSV and JSON export for external analysis
- **Privacy-first** - All data stored locally in SQLite, never transmitted

## Installation

### Prerequisites

- macOS 10.15+
- Rust toolchain (`cargo`)

### Build

```bash
git clone https://github.com/ree-see/lurk.git
cd lurk
cargo build --release
```

### Install

```bash
./scripts/install.sh
```

This copies the binary to `~/.lurk/` and installs a LaunchAgent.

### Grant Permission

Lurk requires Input Monitoring permission:

1. Open **System Settings**
2. Go to **Privacy & Security > Input Monitoring**
3. Enable **lurk**

Verify with:
```bash
lurk check-permission
```

## Usage

```bash
lurk daemon              # Run capture daemon (default)
lurk analyze             # Analyze typing patterns
lurk stats               # Show basic statistics
lurk dashboard           # Open interactive TUI
lurk export -o data.csv  # Export to CSV
lurk export -f json -o data.json  # Export to JSON
```

### Example Output

```
=== Lurk Analysis ===

Total events:     1781
Typing segments:  31 (gaps > 5000ms filtered)
Analyzed events:  1781

Total key presses: 978

--- Top 10 Keys ---
 1. LeftArrow            130 (13.29%)
 2. Backspace             86 (8.79%)
 3. Space                 85 (8.69%)
 4. Return                46 (4.70%)
 5. LeftCommand           41 (4.19%)

--- Top 10 Bigrams ---
 1. LeftArrow -> LeftArrow      127 (13.41%)
 2. Backspace -> Backspace       64 (6.76%)
 3. L -> U                       13 (1.37%)
 4. R -> K                       12 (1.27%)

--- Inter-Key Timing ---
Samples:    797
Mean:       367.1ms
Median:     179ms
P90:        811ms
P95:        1327ms

--- Top 10 Hold Durations ---
 1. Space           mean=126.0ms median=127ms p95=155ms (n=85)
 2. Return          mean=132.1ms median=134ms p95=158ms (n=45)
```

## Data Storage

All data stored in `~/.lurk/`:
- `events.db` - SQLite database with keystroke events
- `logs/` - Daemon stdout/stderr

## LaunchAgent Control

```bash
# Start
launchctl load ~/Library/LaunchAgents/com.user.lurk.plist

# Stop
launchctl unload ~/Library/LaunchAgents/com.user.lurk.plist

# Check status
launchctl list | grep lurk
```

## Uninstall

```bash
./scripts/uninstall.sh
```

## Use Cases

- **Keyboard layout optimization** - Identify high-frequency keys and sequences to inform custom layouts (Colemak, Dvorak, custom)
- **Ergonomic analysis** - Find awkward key combinations causing strain
- **Typing speed improvement** - Identify slow transitions between keys
- **Split keyboard configuration** - Determine optimal key placement across halves
