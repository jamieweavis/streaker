#!/usr/bin/env bash
# Takes the README screenshot (macOS only): the preferences window beside the open tray menu,
# over the desktop, cropped just after the Control Center icon.
#
#   scripts/screenshot.sh [output.png]    (default: docs/screenshot.png)
#
# The app poses itself with fixed preferences and stats (the `screenshot` feature: see
# src/app/screenshot.rs); this hides every other app, takes the screenshot and puts them back.
#
# The terminal running this needs the Screen Recording and Accessibility permissions, and
# permission to control System Events. What the screenshot can't set up itself: the wallpaper,
# dark mode, a 2x display, and the tray icon sitting directly left of the system's menu bar
# icons (quit other menu bar apps, or Cmd-drag it there once it appears).
set -euo pipefail
cd "$(dirname "$0")/.."

out="${1:-docs/screenshot.png}"
# Points of desktop left around the window, and after the Control Center icon.
margin_left=22
margin_bottom=27
margin_right=13

cargo build --features screenshot

system_events() {
    osascript -e 'on run arguments' -e 'tell application "System Events"' -e "$1" -e 'end tell' -e 'end run' "${@:2}"
}

# The installed Streaker would put a second icon in the menu bar.
installed=$(system_events 'get POSIX path of application file of (first process whose name is "Streaker")' 2>/dev/null || true)
if [[ -n "$installed" ]]; then
    osascript -e 'tell application id "com.jamieweavis.streaker" to quit'
fi

frontmost=$(system_events 'get name of first process whose frontmost is true')
visible=$(system_events 'set AppleScript'"'"'s text item delimiters to linefeed
    get (name of every process whose visible is true) as text')
log=$(mktemp)
app_pid=

restore() {
    if [[ -n "$app_pid" ]]; then
        kill "$app_pid" 2>/dev/null
        wait "$app_pid" 2>/dev/null || true
    fi
    rm -f "$log"
    while IFS= read -r name; do
        [[ -n "$name" ]] && system_events 'set visible of process (item 1 of arguments) to true' "$name" || true
    done <<<"$visible"
    system_events 'set frontmost of process (item 1 of arguments) to true' "$frontmost" || true
    if [[ -n "$installed" ]]; then
        open -g "$installed"
    fi
}
trap restore EXIT

system_events 'set visible of (every process whose visible is true) to false'

target/debug/Streaker >"$log" 2>&1 &
app_pid=$!

# The app prints where it put the preferences window just before opening the tray menu.
for _ in $(seq 100); do
    posed=$(grep '^posed ' "$log" || true)
    [[ -n "$posed" ]] && break
    kill -0 "$app_pid" 2>/dev/null || break
    sleep 0.1
done
if [[ -z "$posed" ]]; then
    cat "$log" >&2
    echo "Streaker didn't pose for the screenshot" >&2
    exit 1
fi
read -r _ window_left window_bottom <<<"$posed"
# Time for the menu to open.
sleep 1

# The system's menu bar icons end with Control Center, then the clock.
control_center_right=$(system_events 'tell process "MenuBarAgent"
        set edges to {}
        repeat with extra in (get UI elements of menu bar 1)
            set end of edges to (item 1 of (get position of extra)) + (item 1 of (get size of extra))
        end repeat
        set AppleScript'"'"'s text item delimiters to linefeed
        return edges as text
    end tell' | sort -n | tail -2 | head -1)

left=$((window_left - margin_left))
width=$((control_center_right + margin_right - left))
height=$((window_bottom + margin_bottom))
screencapture -x -R"$left,0,$width,$height" "$out"
echo "Saved $out"
