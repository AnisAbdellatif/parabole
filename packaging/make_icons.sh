#!/usr/bin/env bash
# Regenerates every icon from packaging/icon.svg (needs inkscape + python3). Outputs are committed.
set -euo pipefail
cd "$(dirname "$0")"
render() { inkscape "$1" -w "$2" -h "$2" -o "$3" >/dev/null 2>&1; }

# macOS-style: art inset to 824/1024 with a transparent margin.
PAD=$(mktemp -d)/pad.svg
{ echo '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024"><g transform="translate(100 100) scale(8.5833)">'
  sed '1d;$d' icon.svg; echo '</g></svg>'; } > "$PAD"

rm -rf icons && mkdir -p icons/macos.iconset icons/linux icons/windows
for s in 16 32 128 256 512; do
  render "$PAD" $s "icons/macos.iconset/icon_${s}x${s}.png"
  render "$PAD" $((s * 2)) "icons/macos.iconset/icon_${s}x${s}@2x.png"
done
render "$PAD" 1024 icon.png
for s in 16 22 24 32 48 64 128 256 512; do render icon.svg $s "icons/linux/${s}x${s}.png"; done
for s in 16 20 24 32 40 48 64 128 256; do render icon.svg $s "icons/windows/$s.png"; done
render icon.svg 256 ../assets/icon.png

python3 -I - <<'PY'
import glob, struct
files = sorted(glob.glob("icons/windows/*.png"), key=lambda f: int(f.split("/")[-1][:-4]))
data = [open(f, "rb").read() for f in files]
off = 6 + 16 * len(files)
head, ents, body = struct.pack("<HHH", 0, 1, len(files)), b"", b""
for f, d in zip(files, data):
    s = int(f.split("/")[-1][:-4])
    ents += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(d), off + len(body))
    body += d
open("icon.ico", "wb").write(head + ents + body)
PY
echo "icons regenerated"
