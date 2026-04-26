# claude-usage-monitor

Live terminal dashboard for your [claude.ai](https://claude.ai) usage — 5-hour window, 7-day window, and Extra Credits — refreshed every 2 seconds.

```
                       Claude Usage Monitor   ●  live   claude.ai   ·   e: edit creds  q: quit

                                  5h Window   12%   ·   resets in 3h 14m
                       ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

                                  7d Window   34%   ·   resets in 2d 8h
                       ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

                              Extra Credits   £4.20 / £20.00   (21.0%)
                       ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## How it works

There is no public API for the per-account usage shown on `claude.ai/settings/usage`. This tool calls the same internal endpoint your browser does (`/api/organizations/{org_id}/usage`), authenticated with the cookies from your logged-in browser session. It uses Chrome's TLS/HTTP2 fingerprint (via [`rquest`](https://crates.io/crates/rquest)) so Cloudflare lets the request through.

This is a personal-use tool. You provide your own session cookies; nothing is shared.

## Install

Requires Rust 1.85+ and macOS or Linux.

```sh
git clone https://github.com/maxhuntdesign/claude-usage-monitor
cd claude-usage-monitor
cargo build --release
./target/release/claude-usage-monitor
```

On first run the TUI shows a setup screen. Paste in four values from your browser (see below), press Enter, and the dashboard appears.

## Getting your cookies

1. Open <https://claude.ai/settings/usage> in **Chrome** while signed in.
2. Open DevTools (`⌥⌘I` on macOS, `F12` on Linux) → **Application** tab → **Cookies** → `https://claude.ai`.
3. Copy these three cookie values:
   - `sessionKey` (long string starting with `sk-ant-sid…`)
   - `cf_clearance`
   - `__cf_bm` (optional — short-lived, helps but isn't required)
4. Your **Org ID** is the UUID in the URL on most claude.ai pages, or in the network request to `/api/organizations/<this>/usage` in the **Network** tab.

Paste each into its field on the setup screen. Tab cycles fields, Enter saves.

### Where credentials are stored

Saved to `~/.config/claude-usage-monitor/config.toml` with file mode `600` (owner read/write only). This is plaintext — it's "out of source and not world-readable" but not encrypted at rest. macOS Keychain integration is on the roadmap (see [docs/future/keychain.md](docs/future/keychain.md)).

You can also provide credentials via env vars (these win over the file):

```sh
CLAUDE_ORG_ID=...
CLAUDE_SESSION_KEY=...
CLAUDE_CF_CLEARANCE=...
CLAUDE_CF_BM=...           # optional
```

### When cookies expire

`__cf_bm` cycles every ~30 minutes; `cf_clearance` lasts hours. When fetches start returning 403, the footer turns red and prompts you to press **`e`** — that re-opens the setup screen pre-filled with your current values, so you only have to update the one cookie that changed.

## Keybindings

| Key       | Action                            |
|-----------|-----------------------------------|
| `q`       | Quit                              |
| `Ctrl+C`  | Quit                              |
| `e`       | Edit credentials (opens setup)    |
| `Tab`     | Next field (in setup)             |
| `Shift+Tab` | Previous field (in setup)       |
| `Enter`   | Save (in setup)                   |
| `Esc`     | Cancel setup / quit               |

## Optional add-ons

- **macOS `.app` bundle** — run the TUI as a draggable, resizable window. See [docs/macos-app.md](docs/macos-app.md).
- **SwiftBar menu-bar plugin** — show usage percentages in the macOS menu bar. See [docs/swiftbar.md](docs/swiftbar.md).

## License

MIT — see [LICENSE](LICENSE).
