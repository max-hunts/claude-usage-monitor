#!/usr/bin/env bash
# Build the macOS .app bundle locally.
# Run from the repo root: ./scripts/build-app.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/ClaudeUsageMonitor.app/Contents/MacOS"

if ! command -v alacritty >/dev/null 2>&1; then
    echo "alacritty not found. Install with: brew install --cask alacritty" >&2
    exit 1
fi

echo "==> cargo build --release"
cargo build --release --manifest-path "$ROOT/Cargo.toml"

echo "==> copying binaries into bundle"
cp "$ROOT/target/release/claude-usage-monitor" "$APP/claude-usage-monitor"
cp "$(command -v alacritty)" "$APP/alacritty"
chmod +x "$APP/claude-usage-monitor" "$APP/alacritty" "$APP/launch"

echo "==> done. Run: open ClaudeUsageMonitor.app"
