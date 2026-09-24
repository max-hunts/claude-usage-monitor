# SwiftBar plugin

Show your Claude and Codex usage percentages in the macOS menu bar, with a TUI-style dropdown of bars and reset countdowns.

```
5h12%·7d34%·F61%·CX13%
```

Per-model weekly windows (the Fable-only weekly cap, for example) appear automatically whenever the API reports one. The title abbreviates them to the model's initial to save menu bar space (`F61%`); the dropdown spells them out (`7d Fable   61%`).

## Prerequisites

1. [SwiftBar](https://github.com/swiftbar/SwiftBar) installed (`brew install --cask swiftbar`).
2. `claude-usage-monitor` installed with `cargo install --path . --force` and configured (see the [main README](../README.md)). The plugin invokes it with `--json` to get one usage snapshot.

## Install

Copy or symlink the plugin into your SwiftBar plugins folder:

```sh
ln -s "$(pwd)/swiftbar/claude-usage.2s.sh" \
      "$HOME/Library/Application Support/SwiftBar/Plugins/claude-usage.2s.sh"
chmod +x swiftbar/claude-usage.2s.sh
```

The `2s` in the filename tells SwiftBar to refresh every 2 seconds. `CX13%` means 13% of the Codex weekly allowance has been used; the dropdown spells out “Codex Weekly” and shows its reset countdown. Extra Credits is no longer displayed.

Codex credentials are configured in the TUI with `e` (see the main README). The plugin needs no Codex CLI. A bearer token in **Codex Authorization** and the matching account ID have been verified to work without cookies. Unavailable providers show `—` instead of 0%, with details in the dropdown; the other provider remains visible. OpenAI HTTP 429 responses temporarily delay Codex requests according to `Retry-After`.

## How it finds the binary

SwiftBar runs plugins from a `launchd` context and does not inherit your shell profile. This plugin explicitly adds `~/.cargo/bin`, `/usr/local/bin`, and `/opt/homebrew/bin` to its `PATH`, so a normal `cargo install --path . --force` is sufficient. For a different install location, use either option below:

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
- **Claude auth errors** → refresh the Claude cookies in the TUI with `e`.
- **Codex not configured** → put the bearer token in **Codex Authorization**, not the Cookie field.
- **Codex auth errors** → refresh the token and matching account ID. Cookie is optional; see [Codex troubleshooting](../README.md#codex-troubleshooting).
- **Old Extra Credits display** → update both the binary and this plugin. If the plugin was copied rather than symlinked, copy the updated script into the SwiftBar plugins folder again.
