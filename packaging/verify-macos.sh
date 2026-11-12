#!/bin/sh
# Fails if any binary in the .app still links against Homebrew/Cellar paths, then verifies the signature.
set -eu
cd "$(dirname "$0")/.."
APP="${1:-dist/IPTV Player.app}"
bad=0
for f in "$APP"/Contents/MacOS/* "$APP"/Contents/Frameworks/*.dylib; do
  if otool -L "$f" | tail -n +2 | awk '{print $1}' | grep -E '^/(opt/homebrew|usr/local)|Cellar'; then
    echo "unrelocated reference in $f"
    bad=1
  fi
done
[ "$bad" = 0 ] || exit 1
echo "no /opt/homebrew or /usr/local references"
codesign --verify --deep --strict "$APP" && echo "signature ok"
