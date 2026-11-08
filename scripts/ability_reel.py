"""Build one animated GIF per fighter from the frames of `UME_TOUR=clips` (files `C-<fighter>-<nn>-<move>-<frame>.png`).

usage: python scripts/ability_reel.py <frames_dir> <out_dir> [width] [colors] [fps] [--no-normals]
Every clip gets a caption with its number and move name and its last frame is held so the move can be read.
Captions are drawn with Pillow, the GIF itself is encoded by ffmpeg (global palette, frame differences only).
"""
import collections
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

from PIL import Image, ImageDraw, ImageFont

NAME = re.compile(r"C-(?P<hero>[a-z0-9]+)-(?P<index>\d+)-(?P<move>.+)-(?P<frame>\d+)\.png$")
HOLD_FRAMES = 4
CAPTION_HEIGHT = 22
FILTER = "split[a][b];[a]palettegen=max_colors={colors}:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle"


def captioned(file, label, width, font):
    image = Image.open(file).convert("RGB")
    image = image.resize((width, round(image.height * width / image.width)), Image.LANCZOS)
    canvas = Image.new("RGB", (width, image.height + CAPTION_HEIGHT))
    canvas.paste(image, (0, 0))
    draw = ImageDraw.Draw(canvas)
    draw.rectangle((0, image.height, width, canvas.height), fill=(8, 8, 20))
    draw.text((6, image.height + 4), label, fill=(255, 214, 77), font=font)
    return canvas


def build(hero, clips, out_dir, width, colors, fps):
    font = ImageFont.load_default(size=13)
    target = out_dir / f"{hero}-abilities.gif"
    with tempfile.TemporaryDirectory() as scratch:
        count = 0
        for number, index in enumerate(sorted(clips), 1):
            files = sorted(clips[index]["files"])
            label = f"{hero}  {number:02d}/{len(clips)}  {clips[index]['move'].replace('-', ' ')}"
            for position, file in enumerate(files):
                frame = captioned(file, label, width, font)
                for _ in range(HOLD_FRAMES if position == len(files) - 1 else 1):
                    frame.save(pathlib.Path(scratch) / f"{count:05d}.png")
                    count += 1
        command = ["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(fps), "-i", str(pathlib.Path(scratch) / "%05d.png"), "-vf", FILTER.format(colors=colors), "-loop", "0", str(target)]
        subprocess.run(command, check=True)
    return target, count


def main():
    skip_normals = "--no-normals" in sys.argv
    sys.argv = [argument for argument in sys.argv if argument != "--no-normals"]
    source, out_dir = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    width = int(sys.argv[3]) if len(sys.argv) > 3 else 448
    colors = int(sys.argv[4]) if len(sys.argv) > 4 else 64
    fps = int(sys.argv[5]) if len(sys.argv) > 5 else 14
    if not shutil.which("ffmpeg"):
        sys.exit("ffmpeg is required to encode the GIFs")
    out_dir.mkdir(parents=True, exist_ok=True)
    heroes = collections.defaultdict(lambda: collections.defaultdict(lambda: {"files": [], "move": ""}))
    for file in source.glob("C-*.png"):
        match = NAME.match(file.name)
        if match and not (skip_normals and match["move"].startswith("normal-")):
            clip = heroes[match["hero"]][int(match["index"])]
            clip["files"].append(file)
            clip["move"] = match["move"]
    for hero in sorted(heroes):
        target, count = build(hero, heroes[hero], out_dir, width, colors, fps)
        print(f"{target} {count} frames {target.stat().st_size / 1e6:.2f} MB")


main()
