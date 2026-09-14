#!/bin/bash
# <bitbar.title>Claude + Codex Usage</bitbar.title>
# <bitbar.version>v3.0</bitbar.version>
# <bitbar.author>max</bitbar.author>
# <bitbar.desc>Live Claude and Codex usage from claude-usage-monitor --json</bitbar.desc>
# <swiftbar.hideAbout>true</swiftbar.hideAbout>
# <swiftbar.hideRunInTerminal>true</swiftbar.hideRunInTerminal>
# <swiftbar.hideDisablePlugin>true</swiftbar.hideDisablePlugin>

export PATH="/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:$HOME/.cargo/bin"

# Locate the binary. Override via CLAUDE_USAGE_BIN if installed elsewhere.
BIN="${CLAUDE_USAGE_BIN:-claude-usage-monitor}"
RESOLVED="$(command -v "$BIN" 2>/dev/null || true)"

fail() {
    echo "claude: ⚠ | color=#F85149"
    echo "---"
    while [ "$#" -gt 0 ]; do
        echo "$1 | color=#F85149 font='Menlo' size=12"
        shift
    done
    echo "---"
    echo "Refresh | refresh=true"
    exit 0
}

if [ -z "$RESOLVED" ]; then
    fail \
        "claude-usage-monitor not found on PATH" \
        "looked for: $BIN" \
        "PATH=$PATH" \
        "" \
        "Fix: set CLAUDE_USAGE_BIN to the absolute path," \
        "e.g. /Users/$USER/.cargo/bin/claude-usage-monitor," \
        "either in this script or via launchctl setenv."
fi

ERR_FILE="$(mktemp -t claude-usage.XXXXXX)"
JSON=$("$RESOLVED" --json 2>"$ERR_FILE")
RC=$?
ERR=$(cat "$ERR_FILE")
rm -f "$ERR_FILE"

if [ "$RC" -ne 0 ] || [ -z "$JSON" ]; then
    if [ -n "$ERR" ]; then
        # Strip the leading "Error: " for cleaner display
        ERR=${ERR#Error: }
        fail \
            "claude-usage-monitor failed (exit $RC)" \
            "$ERR" \
            "" \
            "binary: $RESOLVED"
    else
        fail \
            "claude-usage-monitor returned no output (exit $RC)" \
            "binary: $RESOLVED" \
            "" \
            "Try running it in a terminal to set up cookies."
    fi
fi

# Catch a non-JSON success (e.g. accidental text) so we don't crash python below.
case "$JSON" in
    \{*) ;;
    *) fail \
        "claude-usage-monitor returned non-JSON output" \
        "first 80 chars: ${JSON:0:80}" \
        "" \
        "binary: $RESOLVED" ;;
esac

OUT=$(JSON="$JSON" python3 <<'PYEOF'
import os, json
from datetime import datetime, timezone

d = json.loads(os.environ["JSON"])
five  = d.get("five_hour")  or {}
seven = d.get("seven_day")  or {}
codex = d.get("codex") or {}
weekly = codex.get("weekly")
claude_error = d.get("claude_error")

# Model-scoped weekly limits (e.g. the Fable-only weekly cap) are only reported
# in the "limits" array, not as a top-level window.
scoped = []
for lim in d.get("limits") or []:
    if lim.get("kind") != "weekly_scoped":
        continue
    model = ((lim.get("scope") or {}).get("model") or {})
    scoped.append({
        "label":  model.get("display_name") or "Scoped",
        "pct":    lim.get("percent") or 0,
        "resets": lim.get("resets_at"),
    })

# colors (matching TUI)
ORANGE = "#FF8800"
DANGER = "#F85149"
MUTED  = "#8B949E"
FG     = "#E6EDF3"

# 256-color codes (SwiftBar ANSI doesn't render truecolor in title reliably)
ANSI_ORANGE = 208
ANSI_DANGER = 196
ANSI_MUTED  = 240

def ansi(text, code):
    return f"\x1b[38;5;{code}m{text}\x1b[0m"

def bar_color(pct):
    if pct >= 100: return ANSI_DANGER
    return ANSI_ORANGE

def make_bar(pct, width=32):
    pct = max(0.0, min(100.0, pct))
    filled = round(width * pct / 100)
    empty  = width - filled
    fill_seg  = ansi("━" * filled, bar_color(pct)) if filled else ""
    empty_seg = ansi("━" * empty, ANSI_MUTED) if empty else ""
    return fill_seg + empty_seg

def fmt_reset(iso):
    if not iso: return ""
    try:
        t = datetime.fromisoformat(iso.replace("Z", "+00:00"))
    except Exception:
        return ""
    delta = t - datetime.now(timezone.utc)
    secs = int(delta.total_seconds())
    if secs <= 0: return "now"
    d_, secs = divmod(secs, 86400)
    h_, secs = divmod(secs, 3600)
    m_, _    = divmod(secs, 60)
    if d_:  return f"{d_}d {h_}h"
    if h_:  return f"{h_}h {m_}m"
    return f"{m_}m"

# ------ menu bar title ------
five_pct = five.get("utilization")
seven_pct = seven.get("utilization")
def colorize(text, pct):
    if pct >= 100:
        return f"\x1b[38;2;215;0;95m{text}\x1b[0m"  # truecolor pink
    if pct >= 70:
        return f"\x1b[38;5;208m{text}\x1b[0m"       # 256-color orange
    return text

# Explicitly distinguish unavailable values from zero usage.
def title_window(label, pct):
    return f"{label}—" if pct is None else colorize(f"{label}{pct:.0f}%", pct)

parts = [title_window("5h", five_pct), title_window("7d", seven_pct)]
for s in scoped:
    parts.append(colorize(f"{s['label'][:1].upper()}{s['pct']:.0f}%", s["pct"]))
parts.append(title_window("CX", weekly.get("utilization") if weekly else None))
if claude_error or codex.get("error"):
    parts.append("⚠")
print(f"{' · '.join(parts)} | font='Menlo' size=12 ansi=true trim=false")

# ------ dropdown ------
print("---")

def section(label, pct, reset_iso, value_str=None):
    if pct is None:
        print(f"{label}  unavailable | font='Menlo' size=13 color={MUTED}")
        print(" | size=4")
        return
    pct_str = f"{pct:.0f}%" if value_str is None else value_str
    rt = fmt_reset(reset_iso)
    suffix = f"  ·  resets in {rt}" if rt else ""
    print(f"{label}  {pct_str}{suffix} | font='Menlo' size=13 color={FG}")
    print(f"{make_bar(pct)} | font='Menlo' size=13 ansi=true trim=false")
    print(" | size=4")

section("5h Window  ", five_pct,  five.get("resets_at"))
section("7d Window  ", seven_pct, seven.get("resets_at"))

for s in scoped:
    section(f"7d {s['label']}  ", s["pct"], s["resets"])

section("Codex Weekly", weekly.get("utilization") if weekly else None,
        weekly.get("resets_at") if weekly else None)
for provider, error in [("Claude", claude_error), ("Codex", codex.get("error"))]:
    if error:
        safe_error = " ".join(str(error).replace("|", "/").split())
        print(f"{provider}: {safe_error} | color={DANGER} font='Menlo' size=12")
PYEOF
)

echo "$OUT"

echo "---"
echo "Open Claude usage page | href=https://claude.ai/settings/usage"
echo "Open Codex usage page | href=https://chatgpt.com/codex/cloud/settings/analytics#usage"
echo "Refresh | refresh=true"
