"""Generates the placeholder app icon (packaging/icon.png): white play triangle on a blue rounded square. stdlib only."""
import struct, zlib, sys

N = 1024
R = 180  # corner radius
tri = [(380, 300), (380, 724), (760, 512)]


def inside_tri(x, y):
    def s(a, b):
        return (x - b[0]) * (a[1] - b[1]) - (a[0] - b[0]) * (y - b[1])
    d = [s(tri[0], tri[1]), s(tri[1], tri[2]), s(tri[2], tri[0])]
    return not (any(v < 0 for v in d) and any(v > 0 for v in d))


def inside_round(x, y):
    cx = min(max(x, R), N - 1 - R)
    cy = min(max(y, R), N - 1 - R)
    return (x - cx) ** 2 + (y - cy) ** 2 <= R * R


rows = []
for y in range(N):
    row = bytearray([0])
    for x in range(N):
        if not inside_round(x, y):
            row += b"\0\0\0\0"
        elif inside_tri(x, y):
            row += b"\xff\xff\xff\xff"
        else:
            row += b"\x2b\x6c\xe0\xff"
    rows.append(bytes(row))


def chunk(t, d):
    c = struct.pack(">I", len(d)) + t + d
    return c + struct.pack(">I", zlib.crc32(t + d))


png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", N, N, 8, 6, 0, 0, 0))
png += chunk(b"IDAT", zlib.compress(b"".join(rows), 9)) + chunk(b"IEND", b"")
open(sys.argv[1] if len(sys.argv) > 1 else "packaging/icon.png", "wb").write(png)
