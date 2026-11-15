"""Scrubs every file under the given dir of the credentials read from stdin (one value per line), then verifies.
Text files are rewritten with *** ; binaries (PNGs) are only scanned. Any leftover match deletes the dir and fails."""
import shutil, sys, urllib.parse
from pathlib import Path

out = Path(sys.argv[1])
vals = [v for v in sys.stdin.read().split("\n") if v]
host = vals[2].rsplit(":", 1)[0] if len(vals) > 2 else ""
pats = {v for v in vals + [host] if v}
pats |= {urllib.parse.quote(v, safe="") for v in list(pats)}
pats = sorted((p.encode() for p in pats), key=len, reverse=True)
leaks = 0
for f in out.rglob("*"):
    if not f.is_file():
        continue
    data = f.read_bytes()
    try:
        data.decode("utf-8")
        text = True
    except UnicodeDecodeError:
        text = False
    if text:
        new = data
        for p in pats:
            new = new.replace(p, b"***")
        if new != data:
            f.write_bytes(new)
        data = new
    # short values (<6 bytes) would match random PNG bytes: scan binaries for the longer ones only
    leaks += any(p in data for p in pats if text or len(p) >= 6)
if leaks:
    shutil.rmtree(out)
    out.mkdir()
    print("FAIL redaction: credential leak in output (output deleted)")
    sys.exit(1)
print("redaction: clean (%d patterns, all files in %s scanned)" % (len(pats), out.name))
