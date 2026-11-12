#!/bin/sh
# Launches the bundled app with a scratch HOME for a few seconds; fails if it exits early.
set -eu
cd "$(dirname "$0")/.."
APP="${1:-dist/IPTV Player.app}"
SCRATCH=$(mktemp -d)
HOME="$SCRATCH" "$APP/Contents/MacOS/iptv-player" >"$SCRATCH/out.log" 2>&1 &
PID=$!
sleep 6
if kill -0 "$PID" 2>/dev/null; then
  kill "$PID"
  echo "app started and stayed up 6s"
else
  echo "app exited early:"; head -20 "$SCRATCH/out.log"; exit 1
fi
