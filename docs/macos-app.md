# macOS `.app` bundle

The repo includes a thin `.app` wrapper that runs the TUI inside Alacritty as a proper macOS window — draggable, resizable, ⌘Q / ⌘W to quit, dock icon, no terminal needed.

> **You must build this yourself.** Do not redistribute the resulting `.app` — it is unsigned, and any `.app` downloaded over the internet (vs. cloned with git) gets the macOS quarantine attribute and will be blocked by Gatekeeper. Building locally avoids this entirely.

## Prerequisites

- The Rust binary built (`cargo build --release` from the repo root).
- [Alacritty](https://alacritty.org) installed:
  ```sh
  brew install --cask alacritty
  ```

## Build

From the repo root:

```sh
./scripts/build-app.sh
```

This copies the `claude-usage-monitor` release binary and the `alacritty` binary into `ClaudeUsageMonitor.app/Contents/MacOS/`. The `.app` is now self-contained and runnable.

```sh
open ClaudeUsageMonitor.app
```

Drag it into `/Applications` if you want it permanently installed.

## What's in the bundle

```
ClaudeUsageMonitor.app/
└── Contents/
    ├── Info.plist             # bundle identifier, icon, window decorations
    ├── MacOS/
    │   ├── launch             # entry point — runs alacritty -e claude-usage-monitor
    │   ├── alacritty          # copied from your installed alacritty (gitignored)
    │   ├── alacritty.toml     # background color, font, padding to match the TUI
    │   └── claude-usage-monitor  # the Rust binary (gitignored)
    └── Resources/
        └── AppIcon.icns
```

The `alacritty` and `claude-usage-monitor` binaries are listed in `.gitignore` — only the bundle template lives in source control.

## Why building locally works (and downloading doesn't)

Files copied through `git clone` don't get the `com.apple.quarantine` extended attribute. Files downloaded by Safari/Chrome/`curl` do. Quarantined unsigned apps are blocked by Gatekeeper on modern macOS with no easy workaround for end users.

