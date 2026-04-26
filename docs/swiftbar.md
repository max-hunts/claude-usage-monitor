# SwiftBar plugin

Show your Claude usage percentages in the macOS menu bar, with a TUI-style dropdown of bars and reset countdowns.

```
5h 12% · 7d 34% · £4.20/£20.00
```

## Prerequisites

1. [SwiftBar](https://github.com/swiftbar/SwiftBar) installed (`brew install --cask swiftbar`).
2. `claude-usage-monitor` built and configured (see the [main README](../README.md)). The plugin invokes it with `--json` to get one usage snapshot.

## Install

Copy or symlink the plugin into your SwiftBar plugins folder:

```sh
ln -s "$(pwd)/swiftbar/claude-usage.2s.sh" \
      "$HOME/Library/Application Support/SwiftBar/Plugins/claude-usage.2s.sh"
chmod +x swiftbar/claude-usage.2s.sh
```

The `2s` in the filename tells SwiftBar to refresh every 2 seconds.

## How it finds the binary

SwiftBar runs plugins from a `launchd` context with a minimal `PATH` — it does **not** inherit `~/.cargo/bin`, Homebrew paths, or anything from your shell profile. Setting `export` in `~/.zshrc` won't help. You have two options:

**Option A (recommended):** point the plugin directly at the absolute binary path.

```sh
launchctl setenv CLAUDE_USAGE_BIN "$HOME/.cargo/bin/claude-usage-monitor"
# Restart SwiftBar afterwards:
osascript -e 'quit app "SwiftBar"' && open -a SwiftBar
```

To make that survive reboots, add a LaunchAgent or just edit the `BIN=` line at the top of `claude-usage.2s.sh`:

```bash
BIN="/Users/yourname/.cargo/bin/claude-usage-monitor"
```

**Option B:** install the binary somewhere `launchd`'s default PATH already includes:

```sh
sudo cp ~/.cargo/bin/claude-usage-monitor /usr/local/bin/
```

## Verifying

Click the menu bar icon. If the plugin can't run the binary, the dropdown now shows the exact path it searched, the `PATH` SwiftBar saw, and the binary's stderr — paste that into an issue if it's not obvious.

## Troubleshooting

- **`claude: ⚠`** in the menu bar → the plugin couldn't run `claude-usage-monitor --json`. Check that the binary is on `PATH` (run `which claude-usage-monitor` from a terminal SwiftBar can see) or set `CLAUDE_USAGE_BIN`.
- **403 / "auth" errors** → cookies expired. Run `claude-usage-monitor` in a terminal and press `e` to update them.
