import argparse
import json
import math
import os
import re
import struct
from pathlib import Path

from PIL import Image


def effective_ticks(ticks, speed):
    if isinstance(ticks, bool) or not isinstance(ticks, int) or not 0 < ticks <= 2147483647:
        raise ValueError("frame must have a positive i32 tick duration")
    if speed is None:
        return ticks
    if isinstance(speed, bool) or not isinstance(speed, (int, float)) or not math.isfinite(speed):
        raise ValueError("character animation speed must be finite")
    try:
        as_f32 = lambda value: struct.unpack("f", struct.pack("f", value))[0]
        speed = as_f32(speed)
        if not math.isfinite(speed):
            raise ValueError("character animation speed must fit f32")
        if speed > 1:
            value = as_f32(as_f32(ticks) / speed)
            return max(1, min(2147483647, math.floor(value + 0.5)))
    except OverflowError as error:
        raise ValueError("character animation speed must fit f32") from error
    return ticks


def load_frames(character_path, sprite_dir, animation_name):
    character = json.loads(character_path.read_text(encoding="utf-8"))
    meta_path = sprite_dir / "meta.json"
    metadata = json.loads(meta_path.read_text(encoding="utf-8")) if meta_path.exists() else {}
    if not isinstance(metadata, dict) or not isinstance(metadata.get("frames", {}), dict):
        raise ValueError("sprite metadata and frames must be objects")
    frames = character["anims"][animation_name]["frames"]
    if not isinstance(frames, list) or not frames:
        raise ValueError("animation must contain frames")
    output_frames = []
    for index, frame in enumerate(frames):
        if not isinstance(frame, dict) or frame.get("effect") is not None:
            raise ValueError(f"frame {index} is not a body sprite")
        offset = frame.get("offset", [0, 0])
        if offset is None:
            offset = [0, 0]
        if not isinstance(offset, list) or len(offset) != 2 or any(isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) for v in offset):
            raise ValueError(f"frame {index} has an invalid offset")
        name = frame.get("sprite") or frame.get("pose")
        if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9_-]+", name):
            raise ValueError(f"frame {index} needs an explicit named sprite or pose")
        ticks = effective_ticks(frame.get("ticks"), character.get("speed"))
        path = sprite_dir / f"{name}.png"
        if not path.is_file():
            raise ValueError(f"exact sprite missing: {path}; engine aliases are not inferred")
        with Image.open(path) as image:
            width, height = image.size
        geometry = metadata.get("frames", {}).get(name, {})
        if not isinstance(geometry, dict):
            raise ValueError(f"invalid geometry for {name}")
        axis_x = geometry.get("axisX", width / 2)
        axis_y = geometry.get("axisY", height)
        reference = geometry.get("refHeight", metadata.get("refHeight", height))
        values = (axis_x, axis_y, reference)
        if any(isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) for v in values):
            raise ValueError(f"non-finite geometry for {name}")
        if not 0 <= axis_x <= width or not 0 <= axis_y <= height or reference <= 0:
            raise ValueError(f"invalid geometry for {name}")
        output_frames.append({"path": path.resolve(), "ticks": ticks, "width": width, "height": height, "axisX": axis_x, "axisY": axis_y, "refHeight": reference, "offset": offset})
    return character, output_frames


def prepare(character_path, sprite_dir, animation_name, destination):
    _, frames = load_frames(character_path, sprite_dir, animation_name)
    output_frames = []
    common = None
    for frame in frames:
        if frame["offset"] != [0, 0]:
            raise ValueError("frame has an offset; a single-anchor descriptor would misrepresent runtime geometry")
        current = (frame["width"], frame["height"], frame["axisX"], frame["axisY"], frame["refHeight"])
        if common is None:
            common = current
        elif current != common:
            raise ValueError("frames use different canvas sizes, axes or reference heights; a single-anchor descriptor would misrepresent runtime geometry")
        try:
            filename = os.path.relpath(frame["path"], destination.resolve().parent)
        except ValueError:
            filename = str(frame["path"])
        output_frames.append({"file": filename, "duration": frame["ticks"] / 60})
    descriptor = {"kind": "sprite-gen-asset", "version": 1, "anchor": [common[2], common[3]], "frames": output_frames}
    with destination.open("x", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(descriptor, indent=2) + "\n")
    return len(output_frames)


def main():
    parser = argparse.ArgumentParser(description="Describe exact UMF animation frames for sprite-gen inspect-motion")
    parser.add_argument("--character", type=Path, required=True)
    parser.add_argument("--sprites", type=Path, required=True)
    parser.add_argument("--animation", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    try:
        count = prepare(args.character, args.sprites, args.animation, args.out)
    except (OSError, ValueError, TypeError, KeyError) as error:
        parser.exit(1, f"motion descriptor failed: {error}\n")
    print(f"Described {count} frames in {args.out}")


if __name__ == "__main__":
    main()
