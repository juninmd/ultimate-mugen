"""Pack every character's sprite folder into one binary file for the web build.

Layout (little endian):
  "UMEP" | u8 version | u16 id length | id utf-8 | u32 file count |
  count x (u16 name length, name utf-8, u32 data length, data)

Images are downscaled and quantised to 256 colours: the browser fetches one pack per fighter
instead of dozens of multi-megabyte PNGs, and the engine reads the same names it would read from
assets/chars/sprites/<id>/ on disk. Frame anchors in meta.json are scaled with each image so the
floor and body scale match the desktop build.
"""

import io
import json
import math
import os
import sys

from PIL import Image

MAGIC = b"UMEP"
VERSION = 2
SPRITES = os.path.join("assets", "chars", "sprites")
MAX_EDGE = 768
PORTRAIT_EDGE = 256
COLORS = 256
ANCHOR_KEYS = ("axisX", "axisY", "refHeight")


def encode(name, data):
    image = Image.open(io.BytesIO(data)).convert("RGBA")
    limit = PORTRAIT_EDGE if name == "portrait.png" else MAX_EDGE
    scale = min(1.0, limit / max(image.size))
    if scale < 1.0:
        image = image.resize((max(1, round(image.width * scale)), max(1, round(image.height * scale))), Image.LANCZOS)
    quantized = image.quantize(colors=COLORS, method=Image.FASTOCTREE, dither=Image.NONE)
    out = io.BytesIO()
    quantized.save(out, "PNG", optimize=True)
    return out.getvalue(), scale


def rescale_meta(meta_bytes, scales):
    meta = json.loads(meta_bytes)
    frames = meta.get("frames") or {}
    reference = meta.get("refHeight")
    for name, scale in scales.items():
        if scale == 1.0:
            continue
        stem = name[:-4]
        frame = frames.get(stem)
        if frame is None:
            if reference:
                frames[stem] = {"refHeight": reference * scale}
            continue
        for key in ANCHOR_KEYS:
            if key in frame:
                frame[key] = frame[key] * scale
        if isinstance(frame.get("awakeningHair"), list) and len(frame["awakeningHair"]) == 4:
            frame["awakeningHair"] = [math.floor(value * scale) if index < 2 else math.ceil(value * scale) for index, value in enumerate(frame["awakeningHair"])]
    meta["frames"] = frames
    return json.dumps(meta, ensure_ascii=False).encode("utf-8")


def pack_character(character, character_dir, target):
    entries = []
    scales = {}
    for name in sorted(os.listdir(character_dir)):
        path = os.path.join(character_dir, name)
        if name.lower().endswith(".png"):
            with open(path, "rb") as handle:
                entries.append((name, None, handle.read()))
        elif name == "meta.json":
            with open(path, "rb") as handle:
                entries.append((name, "meta", handle.read()))
    if not entries:
        return 0
    payload = bytearray(MAGIC)
    payload.append(VERSION)
    encoded_id = character.encode("utf-8")
    payload.extend(len(encoded_id).to_bytes(2, "little"))
    payload.extend(encoded_id)
    payload.extend(len(entries).to_bytes(4, "little"))
    meta_pending = None
    for name, kind, data in entries:
        if kind == "meta":
            meta_pending = data
            continue
        encoded, scales[name] = encode(name, data)
        payload.extend(len(name.encode("utf-8")).to_bytes(2, "little"))
        payload.extend(name.encode("utf-8"))
        payload.extend(len(encoded).to_bytes(4, "little"))
        payload.extend(encoded)
    if meta_pending is not None:
        encoded = rescale_meta(meta_pending, scales)
        payload.extend(len(b"meta.json").to_bytes(2, "little"))
        payload.extend(b"meta.json")
        payload.extend(len(encoded).to_bytes(4, "little"))
        payload.extend(encoded)
    with open(target, "wb") as handle:
        handle.write(payload)
    return len(payload)


def main():
    out_root = sys.argv[1] if len(sys.argv) > 1 else os.path.join("web", "dist")
    target_dir = os.path.join(out_root, "assets", "chars", "sprites")
    os.makedirs(target_dir, exist_ok=True)
    total = 0
    for character in sorted(os.listdir(SPRITES)):
        source = os.path.join(SPRITES, character)
        if not os.path.isdir(source):
            continue
        size = pack_character(character, source, os.path.join(target_dir, f"{character}.pack"))
        if size:
            total += size
            print(f"  {character}.pack {size / 1024 / 1024:.2f} MB")
    print(f"OK: {target_dir} ({total / 1024 / 1024:.1f} MB)")


if __name__ == "__main__":
    main()
