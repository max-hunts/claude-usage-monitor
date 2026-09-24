# claude-usage-monitor

Claude and Codex usage fetching have been verified with live accounts. Both integrations use internal endpoints, so upstream changes may require updates.

Live terminal dashboard for your [claude.ai](https://claude.ai) usage — 5-hour window, 7-day window, per-model weekly windows (e.g. Fable), and Codex weekly usage — refreshed every 2 seconds.

```
Claude + Codex Usage   ●  live   Claude / OpenAI   ·   e: edit creds  q: quit

   5h Window   12%   ·   resets in 3h 14m
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   7d Window   34%   ·   resets in 2d 8h
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   7d Fable   61%   ·   resets in 2d 8h
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   Codex Weekly   13%   ·   resets in 2d 8h
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## How it works

There is no public API for the per-account usage shown on `claude.ai/settings/usage`. This tool calls the same internal endpoint your browser does (`/api/organizations/{org_id}/usage`), authenticated with the cookies from your logged-in browser session. It uses Chrome's TLS/HTTP2 fingerprint (via [`rquest`](https://crates.io/crates/rquest)) so Cloudflare lets the request through.

Codex is fetched directly from `https://chatgpt.com/backend-api/wham/usage`, using an access token and `ChatGPT-Account-Id` header, plus an optional Cookie header. There is no Codex CLI/app-server dependency. Both providers poll independently every **2 seconds**, with an 8-second request timeout. Codex selects the 604,800-second window from either primary or secondary; a missing weekly window displays as unavailable, never 0%.

This is a personal-use tool. You provide your own credentials, which are sent only to their respective provider.

## Install

Requires Rust 1.85+, CMake, and macOS or Linux.

```sh
# macOS — install cmake if you don't have it
brew install cmake

git clone https://github.com/max-hunts/claude-usage-monitor
cd claude-usage-monitor
cargo build --release
./target/release/claude-usage-monitor
```

To install or update the command in `~/.cargo/bin` from this checkout:

```sh
cargo install --path . --force
claude-usage-monitor
```

Ensure `~/.cargo/bin` is on your `PATH`. Quit any running monitor with `q` and relaunch it after installing; an already-running process continues using the previous version. Saved credentials are preserved. The macOS `.app` contains its own copy of the binary and must be rebuilt separately.

On first run the TUI shows a setup screen. Paste in four values from your browser (see below), press Enter, and the dashboard appears. Codex credentials are optional; existing Claude-only configuration still works.

## Getting your cookies

1. Open <https://claude.ai/settings/usage> in **Chrome** while signed in.
2. Open DevTools (`⌥⌘I` on macOS, `F12` on Linux) → **Application** tab → **Cookies** → `https://claude.ai`.
3. Copy these three cookie values:
   - `sessionKey` (long string starting with `sk-ant-sid…`)
   - `cf_clearance`
   - `__cf_bm` (optional — short-lived, helps but isn't required)
4. Your **Org ID** is the UUID in the URL on most claude.ai pages, or in the network request to `/api/organizations/<this>/usage` in the **Network** tab.

Paste each into its field on the setup screen. Tab cycles fields, Enter saves.

## Codex credentials

1. Open [Codex usage](https://chatgpt.com/codex/cloud/settings/analytics#usage) in Chrome or another browser with DevTools, sign in, and select the intended workspace.
2. Open **DevTools → Network**, filter for **`wham/usage`**, and reload the page.
3. Select the request and expand **Headers → Request Headers**.
4. Run `claude-usage-monitor`, press **`e`**, and Tab to the Codex fields. Copy the values as follows:

| Monitor field | Request header | What to paste |
|---|---|---|
| **Codex Authorization** | `authorization` | Copy the entire value, including the `Bearer ` prefix: `Bearer eyJ…`. |
| **Codex account ID** | `chatgpt-account-id` | The account/workspace ID used by this request. |
| **Codex Cookie header (optional)** | `cookie` | Leave blank initially. Only add the complete Cookie header if required. |

**Paste the complete Authorization value (`Bearer …`) into Codex Authorization, not into the Cookie field.** The parser also accepts a token without the prefix for compatibility. Press **Enter** to save. The token and account ID together have been verified against the live endpoint without a Cookie header; cookie-only authentication has not been verified.

These are session credentials, not an OpenAI Platform API key. Paste the full values directly into the local setup form. Token and cookie fields are masked; **F2** or **Ctrl+U** instantly clears the focused field; the shortcut is shown on the selected field and in the footer. The form scrolls to keep the focused field visible.

Within a couple of seconds, **Codex Weekly** should show the percentage **used** and the next reset time. For example, a browser showing 81% remaining should correspond to 19% in the monitor. On the tested account, the API returns a weekly primary window and a null secondary window. The monitor selects the weekly window by duration, so accounts reporting it in the secondary slot also work.

Environment overrides (independent of the Claude environment variables):

```sh
CODEX_ACCESS_TOKEN=...
CODEX_ACCOUNT_ID=...
CODEX_COOKIE=...          # optional
```

A supplied Codex environment variable overrides its saved field; an empty value clears it for that run. Leave Codex credentials blank to use Claude alone.

### Where credentials are stored

Saved to `~/.config/claude-usage-monitor/config.toml` with file mode `600` (owner read/write only). This is plaintext — it's "out of source and not world-readable" but not encrypted at rest. macOS Keychain integration is on the roadmap (see [docs/future/keychain.md](docs/future/keychain.md)).

You can also provide credentials via env vars (these win over the file):

```sh
CLAUDE_ORG_ID=...
CLAUDE_SESSION_KEY=...
CLAUDE_CF_CLEARANCE=...
CLAUDE_CF_BM=...           # optional
```

### Refresh and partial failures

The TUI keeps the last successful reading when one provider fails, marks it stale, and reports the provider's error. Network calls run off the input/render loop. The other provider continues refreshing.

`--json` fetches both providers concurrently once. It preserves Claude's top-level fields except `extra_usage`, and adds `claude_error`, `claude_updated_at` and:

```json
{
  "codex": {
    "weekly": { "utilization": 13.0, "resets_at": "2033-05-18T03:33:20+00:00" },
    "error": null,
    "updated_at": "2026-09-14T12:00:00+00:00"
  }
}
```

Provider failures are represented in JSON with a successful process exit, allowing SwiftBar to show the other provider. A failed one-shot fetch has no cached reading and shows unavailable. Missing configuration or startup failures can still produce a nonzero exit.

If OpenAI returns HTTP 429, the monitor respects `Retry-After` (seconds or HTTP date; 30 seconds if absent). The account-specific deadline is saved in `~/.config/claude-usage-monitor/codex-backoff.json` so SwiftBar's separate invocations honor it too. Normal polling resumes every 2 seconds afterward. This file contains no tokens or cookies.

### Codex troubleshooting

- **Not configured:** the Codex Authorization field is empty. Press `e` and check that the bearer token was not pasted into the optional Cookie field.
- **Account ID missing:** fill in the account ID from the same browser request as the token.
- **Auth 401/403:** capture a fresh token and matching account ID. If those still fail, try the browser request's complete Cookie header in the optional field.
- **Weekly limit unavailable:** the response did not contain a 7-day window. The monitor shows unavailable rather than a misleading 0%.
- **Credentials changed outside the TUI:** restart the monitor; running workers keep the configuration loaded at startup or when the setup form was saved.

### When Claude cookies expire

`__cf_bm` cycles every ~30 minutes; `cf_clearance` lasts hours. When fetches start returning 403, the footer turns red and prompts you to press **`e`** — that re-opens the setup screen pre-filled with your current values, so you only have to update the one cookie that changed.

## Keybindings

| Key       | Action                            |
|-----------|-----------------------------------|
| `q`       | Quit                              |
| `Ctrl+C`  | Quit                              |
| `e`       | Edit credentials (opens setup)    |
| `Tab`     | Next field (in setup)             |
| `Shift+Tab` | Previous field (in setup)       |
| `F2` / `Ctrl+U` | Clear current field instantly (in setup) |
| `Enter`   | Save (in setup)                   |
| `Esc`     | Cancel setup / quit               |

## Optional add-ons

- **macOS `.app` bundle** — run the TUI as a draggable, resizable window. See [docs/macos-app.md](docs/macos-app.md).
- **SwiftBar menu-bar plugin** — show usage percentages in the macOS menu bar. See [docs/swiftbar.md](docs/swiftbar.md).

## License

MIT — see [LICENSE](LICENSE).

## Development checks

```sh
cargo test
cargo clippy --all-targets -- -D warnings
python3 -m unittest discover -s tests
bash -n swiftbar/claude-usage.2s.sh
```

Tests use fixture credentials and a local mock HTTP server; they do not contact Claude or OpenAI.
