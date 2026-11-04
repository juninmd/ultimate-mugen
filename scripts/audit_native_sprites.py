import argparse
import json
from pathlib import Path

from PIL import Image


def inspect(root):
    fighters = []
    for source in sorted((root / "assets/chars").glob("*.json")):
        character = json.loads(source.read_text(encoding="utf-8"))
        if "id" not in character:
            continue
        folder = root / "assets/chars/sprites" / character["id"]
        meta = json.loads((folder / "meta.json").read_text(encoding="utf-8")) if (folder / "meta.json").exists() else {}
        files = sorted(folder.glob("*.png"))
        images = {}
        for path in files:
            with Image.open(path) as image:
                alpha = image.convert("RGBA").getchannel("A")
                images[path.name] = {"size": list(image.size), "alphaBounds": alpha.getbbox(), "transparentPixels": alpha.getextrema()[0] == 0}
        frames = []
        for name, animation in character.get("anims", {}).items():
            for index, frame in enumerate(animation.get("frames", [])):
                pose = frame.get("sprite") or frame.get("pose")
                effect = frame.get("effect") is not None
                exact = isinstance(pose, str) and (folder / (pose + ".png")).is_file()
                frames.append({"animation": name, "frame": index, "pose": pose, "effect": effect, "ticks": frame.get("ticks"), "exactFile": exact})
        fighters.append({"id": character["id"], "declaredHeight": character.get("stats", {}).get("height"), "referenceHeight": meta.get("refHeight"), "avatar": (folder / "portrait.png").is_file(), "images": images, "frames": frames})
    return {"scope": "Exact-file inventory; semantic aliases and procedural effects must be reviewed separately. A missing exact file alone does not prove missing runtime art.", "fighters": fighters}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    report = inspect(args.root)
    args.out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for fighter in report["fighters"]:
        body = [frame for frame in fighter["frames"] if not frame["effect"]]
        print(fighter["id"], "height", fighter["declaredHeight"], "avatar", fighter["avatar"], "body frames", len(body), "exact files", sum(frame["exactFile"] for frame in body))


if __name__ == "__main__":
    main()
