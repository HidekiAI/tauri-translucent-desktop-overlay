#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UDP_PORT=$(jq -r '.udp_port // 7331' "${SCRIPT_DIR}/hud_config.json" 2>/dev/null || echo 7331)
ORIGINAL_TEXT="Hello world, Hello Shiroe!"
LOREM_WIDE="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."

# Ensure netcat (with UDP support) is installed
if [ ! -e "$(which nc 2>/dev/null)" ]; then
    echo "netcat not found, installing netcat-openbsd..."
    sudo apt install -y netcat-openbsd
fi

# Send a string to the HUD over UDP loopback
send_hud() {
    printf '%s' "$1" | nc -u -w1 127.0.0.1 "$UDP_PORT"
}

# Warn if picom is not running — do NOT start or stop compositors here.
# Manipulating compositors while the app is starting causes a mid-flight
# compositor switch that corrupts the X surface and creates ghost pixels.
# Set up picom once manually before running this script (see README).
if ! pgrep -x picom >/dev/null; then
    echo "WARNING: picom is not running. Start it first:"
    echo "  xfconf-query -c xfwm4 -p /general/use_compositing -s false"
    echo "  picom --backend glx --no-use-damage &"
    echo "Continuing anyway — transparency may not work correctly."
fi

# Kill any existing HUD instances before starting a fresh one.
# WebKit2GTK spawns separate WebKitWebProcess and WebKitGPUProcess child
# processes that outlive the main app and retain stale rendering state.
# They must be explicitly killed or the next run inherits ghost pixels.
pkill -f "tauri-desktop-HUD" 2>/dev/null || true
pkill -f "cargo tauri" 2>/dev/null || true   # note: space not hyphen
pkill -f "WebKitWebProcess" 2>/dev/null || true
pkill -f "WebKitGPUProcess" 2>/dev/null || true
sleep 2

# Launch the app in the background with full backtraces on panic
cd "$SCRIPT_DIR"
RUST_BACKTRACE=1 cargo tauri dev &
APP_PID=$!

# Wait until the UDP port is ready (app finished compiling and started)
echo "Waiting for app to start..."
until printf '' | nc -u -w1 127.0.0.1 "$UDP_PORT" 2>/dev/null && ss -ulnp | grep -q ":${UDP_PORT}"; do
    sleep 1
done
# Wait for the WebView to fully load and for init() to register the event
# listener.  The UDP socket binds before the JS listener is ready; messages
# sent too early are silently dropped and never rendered.
# More importantly: the window must complete its FIRST paint before any text
# is sent.  If a message arrives before the first composite, the X11 surface
# is uninitialised and WebKit's dirty-rect logic never backfills those pixels,
# causing every subsequent message to ghost over the initial garbage state.
# 10 s gives the compositor time to paint the empty window cleanly.
echo "App ready. Waiting for WebView to initialise..."
sleep 3

DELAY=3 # seconds between messages — reduce to 10 then 5 once confirmed working

send_hud "Get ready..."
sleep $DELAY

send_hud "3"
sleep $DELAY

send_hud "2"
sleep $DELAY

send_hud "1"
sleep $DELAY

# Long line — triggers min_font_size shrink if min_font_size_pt > 0 in hud_config.json
send_hud "$LOREM_WIDE"
sleep $DELAY

send_hud "$ORIGINAL_TEXT"
sleep $DELAY

kill "$APP_PID" 2>/dev/null
pkill -f "tauri-desktop-HUD" 2>/dev/null || true
wait "$APP_PID" 2>/dev/null

echo "Demo complete."
