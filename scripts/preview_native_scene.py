import argparse
import json
import subprocess
import tempfile
from pathlib import Path

from PIL import Image

from native_motion_descriptor import prepare as describe


def preview(character, sprites, animation, destination, executable, cycles):
    if destination.exists():
        raise ValueError(f"output already exists: {destination}")
    if not executable.is_file():
        raise ValueError(f"sprite-gen executable not found: {executable}")
    if cycles < 1 or cycles > 10:
        raise ValueError("cycles must be between 1 and 10")
    with tempfile.TemporaryDirectory(prefix="ume-scene-descriptor-") as temp:
        descriptor_path = Path(temp) / "asset.json"
        describe(character, sprites, animation, descriptor_path)
        descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
        ticks = [round(frame["duration"] * 60) for frame in descriptor["frames"]]
        first = (descriptor_path.parent / descriptor["frames"][0]["file"]).resolve()
        with Image.open(first) as image:
            width, height = image.size
        if width % 2 or height % 2:
            raise ValueError("MP4 output requires an even canvas width and height")
    if sum(ticks) * cycles > 10000:
        raise ValueError("requested scene exceeds the sprite-gen 10000-frame limit")
    destination.mkdir(parents=True)
    describe(character, sprites, animation, destination / "asset.json")
    scene = {
        "kind": "sprite-gen-scene",
        "version": 1,
        "canvas": {"width": width, "height": height, "background": [36, 40, 48, 255]},
        "fps": 60,
        "duration": sum(ticks) * cycles / 60,
        "assets": {"actor": {"source": "asset.json"}},
        "layers": [{"id": "actor", "asset": "actor", "at": descriptor["anchor"], "loop": cycles > 1}],
    }
    spec = destination / "scene.json"
    spec.write_text(json.dumps(scene, indent=2) + "\n", encoding="utf-8")
    subprocess.run([str(executable), "scene-render", "--spec", str(spec), "--out-dir", str(destination / "render"), "--formats", "mp4"], check=True)
    return destination / "render" / "scene.mp4"


def main():
    parser = argparse.ArgumentParser(description="Render a 60 fps sprite-gen scene using native UMF frame timing")
    parser.add_argument("--character", type=Path, required=True)
    parser.add_argument("--sprites", type=Path, required=True)
    parser.add_argument("--animation", required=True)
    parser.add_argument("--sprite-gen", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--cycles", type=int, default=1)
    args = parser.parse_args()
    try:
        output = preview(args.character, args.sprites, args.animation, args.out, args.sprite_gen.resolve(), args.cycles)
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"motion scene failed: {error}\n")
    print(f"Preview: {output}; inspect motion and collision in the game")


if __name__ == "__main__":
    main()
