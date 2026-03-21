#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEXT_FILE="$SCRIPT_DIR/hud_text.txt"
ORIGINAL_TEXT="Hello world, Hello Shiroe!"
LOREM_WIDE="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore."

# Restore original text on exit/interrupt
cleanup() {
    echo "$ORIGINAL_TEXT" > "$TEXT_FILE"
}
trap cleanup EXIT INT TERM

# Launch the app in the background
echo "$ORIGINAL_TEXT" > "$TEXT_FILE"
cd "$SCRIPT_DIR"
pnpm run:tauri &
APP_PID=$!

# Give the app time to start and render
sleep 3

echo "Get ready..." > "$TEXT_FILE"
sleep 1

echo "3" > "$TEXT_FILE"
sleep 1

echo "2" > "$TEXT_FILE"
sleep 1

echo "1" > "$TEXT_FILE"
sleep 1

echo "$LOREM_WIDE" > "$TEXT_FILE"
sleep 3

echo "$ORIGINAL_TEXT" > "$TEXT_FILE"
sleep 3

kill "$APP_PID" 2>/dev/null
wait "$APP_PID" 2>/dev/null

echo "Demo complete."
