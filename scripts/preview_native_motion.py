import argparse
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

from native_motion_descriptor import prepare as describe


def preview(character, sprites, animation, destination, executable, loop):
    if destination.exists():
        raise ValueError(f"output already exists: {destination}")
    if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_]*", animation):
        raise ValueError("animation must start with a letter and contain only letters, numbers or underscores")
    if not executable.is_file():
        raise ValueError(f"sprite-gen executable not found: {executable}")
    with tempfile.TemporaryDirectory(prefix="ume-motion-descriptor-") as temp:
        descriptor_path = Path(temp) / "asset.json"
        describe(character, sprites, animation, descriptor_path)
        descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
        ticks = [round(frame["duration"] * 60) for frame in descriptor["frames"]]
        if len(set(ticks)) != 1 or 60 % ticks[0] != 0 or (ticks[0] * 100) % 60 != 0:
            raise ValueError("GIF preview requires equal durations representable exactly in fps and centiseconds")
        source = [(descriptor_path.parent / frame["file"]).resolve() for frame in descriptor["frames"]]
        destination.mkdir(parents=True)
        pngs = destination / "pngs"
        pngs.mkdir()
        for index, path in enumerate(source):
            shutil.copyfile(path, pngs / f"frame-{index:04d}.png")
    run = destination / "run"
    subprocess.run([str(executable), "unpack-atlas", "--pngs-dir", str(pngs), "--state-name", animation, "--out-dir", str(run)], check=True)
    request_path = run / "sprite-request.json"
    request = json.loads(request_path.read_text(encoding="utf-8"))
    request["states"][animation]["fps"] = 60 // ticks[0]
    request["states"][animation]["loop"] = loop
    request_path.write_text(json.dumps(request, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    subprocess.run([str(executable), "preview", "--run-dir", str(run)], check=True)
    return run / "qa" / f"{animation}.gif"


def main():
    parser = argparse.ArgumentParser(description="Create a sprite-gen motion preview of an existing UMF animation")
    parser.add_argument("--character", type=Path, required=True)
    parser.add_argument("--sprites", type=Path, required=True)
    parser.add_argument("--animation", required=True)
    parser.add_argument("--sprite-gen", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--loop", action="store_true")
    args = parser.parse_args()
    try:
        output = preview(args.character, args.sprites, args.animation, args.out, args.sprite_gen.resolve(), args.loop)
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"motion preview failed: {error}\n")
    print(f"Preview: {output}; inspect motion and collision in the game")


if __name__ == "__main__":
    main()
