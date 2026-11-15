#!/usr/bin/env python3
"""Regenerates assets/stages/data/example/: a data-only built-in stage (stage.def + SFF v2 with PNG32 sprites).

Run from the repository root: python3 scripts/make-example-stage.py
"""
import math
import pathlib
import struct
import zlib

OUT = pathlib.Path(__file__).resolve().parent.parent / "assets" / "stages" / "data" / "example"


def png(w, h, px):
    raw = b"".join(b"\x00" + bytes(px[y * w * 4:(y + 1) * w * 4]) for y in range(h))

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def lerp(a, b, t):
    return [a[i] + (b[i] - a[i]) * t for i in range(3)]


def paint(w, h, fn):
    px = bytearray(w * h * 4)
    for y in range(h):
        for x in range(w):
            r, g, b, a = fn(x, y)
            i = (y * w + x) * 4
            px[i:i + 4] = bytes((int(r) & 255, int(g) & 255, int(b) & 255, int(a) & 255))
    return px


def sky(x, y):
    c = lerp((14, 20, 58), (236, 120, 96), min(1.0, (y / 170.0) ** 1.6))
    return (*c, 255)


def hills(x, y):
    h = 42 + 16 * math.sin(x * 2 * math.pi / 320 * 2) + 8 * math.sin(x * 2 * math.pi / 320 * 5)
    return (40, 28, 72, 255) if y >= 90 - h else (0, 0, 0, 0)


def ridge(x, y):
    h = 18 + 10 * math.sin(x * 2 * math.pi / 320 * 3) + 5 * math.cos(x * 2 * math.pi / 320 * 7)
    return (74, 52, 110, 255) if y >= 40 - h else (0, 0, 0, 0)


def floor(x, y):
    shade = 0.8 + 0.2 * math.sin(y * 0.9)
    c = lerp((70, 60, 84), (30, 24, 40), y / 70.0)
    seam = 26 if (x + (y // 14) * 17) % 40 == 0 else 0
    return (c[0] * shade + seam, c[1] * shade + seam, c[2] * shade + seam, 255)


SPRITES = [(320, 240, sky), (320, 90, hills), (320, 40, ridge), (320, 70, floor)]


def sff(sprites):
    blocks, nodes, offset = [], [], 0
    for group, (w, h, pixels) in enumerate(sprites):
        body = b"\x00\x00\x00\x00" + png(w, h, pixels)
        nodes.append((group, w, h, offset, len(body)))
        blocks.append(body)
        offset += len(body)
    head = 68
    ldata = b"".join(blocks)
    out = bytearray(b"ElecbyteSpr\x00" + bytes([0, 0, 0, 2]) + b"\x00" * 20)
    out += struct.pack("<IIIIIIII", head, len(sprites), 0, 0, head + 28 * len(sprites), len(ldata), 0, 0)
    for group, w, h, ofs, length in nodes:
        out += struct.pack("<HHHHhhHBBIIHH", group, 0, w, h, 0, 0, 0, 12, 32, ofs, length, 0, 0)
    return bytes(out) + ldata


DEF = """[Info]
name = "Example Aurora Dock"
displayname = "Cais da Aurora (exemplo)"
author = "Ultimate MUGEN"
mugenversion = 1.1

[Camera]
startx = 0
starty = 0
boundleft = -120
boundright = 120
boundhigh = -25
boundlow = 0
verticalfollow = 0.2
tension = 50

[PlayerInfo]
p1startx = -70
p2startx = 70

[StageInfo]
zoffset = 200
localcoord = 320, 240

[BGdef]
spr = example.sff

[BG Sky]
type = normal
spriteno = 0, 0
layerno = 0
start = 0, 0
delta = 1, 1

[BG Hills]
type = normal
spriteno = 1, 0
layerno = 0
start = 0, 70
delta = 0.5, 1
tile = 1, 0

[BG Ridge]
type = parallax
spriteno = 2, 0
layerno = 0
start = 0, 140
delta = 0.8, 1
velocity = 0.1, 0

[BG Floor]
type = normal
spriteno = 3, 0
layerno = 0
start = 0, 170
delta = 1, 1
tile = 1, 0

[Sky]
type = aurora
color = 120, 255, 200
intensity = 0.5
speed = 1
"""

if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    rendered = [(w, h, paint(w, h, fn)) for w, h, fn in SPRITES]
    (OUT / "example.sff").write_bytes(sff(rendered))
    (OUT / "stage.def").write_text(DEF, encoding="utf-8")
    print("wrote", OUT)
