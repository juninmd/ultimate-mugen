"""Mirror of the native sprite resolution rules in ``crates/engine/src/native/pixel.rs``.

Native sprite art is resolved at runtime by ``load_pose_with_source``: a pose name is looked up as
its own ``.png``, then through the legacy numbered stems, then the idle canvas. The strict Rust audit
(``native_sprite_audit --require-complete``) treats the ``idle-reuse``, ``fallback-idle`` and
``missing`` resolutions as gaps. This script reproduces those classifications in Python so the roster
can be checked without building the engine. It cannot verify distinct pixel images; run the Rust audit
for that.

Usage: ``python scripts/native_sprite_coverage.py [--out report.json] [--targets]``
``--targets`` prints the unique frame names a regeneration pass should create.
Exit code 2 when any fighter still has gaps (matching the Rust example).
"""

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHARS = ROOT / "assets/chars"

BASE_STEM = "01_idle"
STAND_KEYS = ("idle", "walk", "run", "stand", "turn", "crouch", "land", "hop", "guard", "getup", "recover")
PUNCH_KEYS = ("jab", "punch")
KICK_KEYS = ("kick", "lk", "hk", "sweep")
SP_KEYS = ("special", "super", "arc", "frost", "cut", "rise")
PROJ_KEYS = ("proj", "blast", "fireball", "wave", "gale")
VICTORY_KEYS = ("win", "vic", "taunt", "intro")
HIT_STEMS = {
    "idle1": "07_idle1", "idle3": "07_idle1", "idle2": "08_idle2",
    "hith1": "09_hit1", "hith2": "10_hit2", "hith3": "11_hit3",
    "hitair1": "12_fall1", "fall1": "12_fall1", "trip": "12_fall1",
    "hitair2": "13_fall2", "fall2": "13_fall2", "bounce": "13_fall2",
    "lie": "14_lie", "dead": "14_lie", "lose": "14_lie",
    "powerup0": "15_power0", "powerup1": "16_power1", "powerup2": "17_power2",
}
LEVEL_MAX_UNIQUE = {"idle": 3, "walk": 6, "walkBack": 6, "run": 4, "powerUp": 3}


def contains_any(lower, keys):
    return any(key in lower for key in keys)


def basic_stem(lower):
    if contains_any(lower, STAND_KEYS):
        return BASE_STEM
    if contains_any(lower, PUNCH_KEYS) or lower.startswith("str"):
        return "02_attack1"
    if contains_any(lower, KICK_KEYS):
        return "03_attack2"
    if contains_any(lower, SP_KEYS) or lower.startswith("sp"):
        return "04_special"
    if contains_any(lower, PROJ_KEYS):
        return "05_projectile"
    if contains_any(lower, VICTORY_KEYS):
        return "06_victory"
    return BASE_STEM


def pose_stem(lower):
    if lower.startswith("df"):
        return "14_lie"
    return HIT_STEMS.get(lower, basic_stem(lower))


def resolution_for(pose, files):
    lower = pose.lower()
    chain = [pose, pose_stem(lower), BASE_STEM, "idle"]
    seen = set()
    for name in chain:
        key = name.lower()
        if key in seen:
            continue
        seen.add(key)
        if name in files:
            if name == pose:
                return "exact", name
            if name == BASE_STEM and not lower.startswith("idle"):
                return "idle-reuse", name
            if name == pose_stem(lower):
                return "shared-pose", name
            return "fallback-idle", name
    return "missing", None


def targets_for(char, defaults):
    folder = CHARS / "sprites" / char["id"]
    files = {path.stem for path in folder.glob("*.png") if path.suffix == ".png"}
    poses = dict(defaults.get("poses") or {})
    poses.update((char.get("art") or {}).get("poses") or {})
    targets = {}
    anims = dict(defaults.get("anims") or {})
    anims.update(char.get("anims") or {})
    for anim in anims.values():
        for frame in anim.get("frames", []):
            if frame.get("effect") or frame.get("sprite"):
                continue
            pose = frame.get("pose")
            if isinstance(pose, str):
                lower = pose.lower()
                if pose not in poses:
                    continue
                if lower.startswith("df"):
                    if pose not in files:
                        targets[pose] = True
                    continue
                if lower.startswith("idle"):
                    continue
                if pose in files:
                    continue
                stem = pose_stem(lower)
                if stem != BASE_STEM and stem in files:
                    continue
                targets[pose] = True
            elif isinstance(pose, dict):
                blob = json.dumps(pose, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
                digest = 0xcbf29ce484222325
                for byte in blob:
                    digest = ((digest ^ byte) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
                key = f"inline_{digest:016x}"
                if key not in files:
                    targets[key] = True
    return sorted(targets)


def audit_character(char, defaults):
    cid = char["id"]
    folder = CHARS / "sprites" / cid
    files = {path.stem for path in folder.glob("*.png") if path.suffix == ".png"}
    poses = dict(defaults.get("poses") or {})
    poses.update((char.get("art") or {}).get("poses") or {})
    anims = dict(defaults.get("anims") or {})
    anims.update(char.get("anims") or {})
    gap_names = {"idle-reuse", "fallback-idle", "missing"}
    animations = []
    for name, anim in anims.items():
        frames = []
        seen_keys = set()
        for index, frame in enumerate(anim.get("frames", [])):
            if frame.get("effect"):
                frames.append({"frame": index, "resolution": "procedural-effect"})
                continue
            key = frame.get("sprite") or frame.get("pose")
            if not isinstance(key, str):
                if isinstance(key, dict):
                    blob = json.dumps(key, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
                    digest = 0xcbf29ce484222325
                    for byte in blob:
                        digest = ((digest ^ byte) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
                    key = f"inline_{digest:016x}"
                    resolution, source = resolution_for(key, files)
                    frames.append({"frame": index, "pose": key, "resolution": resolution, "file": source})
                    continue
                renderable = (char.get("art") or {}).get("type") == "rig"
                frames.append({"frame": index, "resolution": "inline-rig" if renderable else "idle-reuse"})
                continue
            resolution, source = resolution_for(key, files)
            frames.append({"frame": index, "pose": key, "resolution": resolution, "file": source})
        unique = {frame.get("file") or frame.get("resolution") for frame in frames}
        unique.discard(None)
        entry = {"animation": name, "uniqueBodyImages": len(unique), "frames": frames}
        gaps = []
        minimum = LEVEL_MAX_UNIQUE.get(name, 0)
        if unique and len(unique) < minimum:
            gaps.append(f"{name}: {len(unique)} distinct body images, needs {minimum}")
        for frame in frames:
            if frame["resolution"] in gap_names:
                gaps.append(f"{name}[{frame['frame']}]: {frame['resolution']}")
        entry["uniqueGaps"] = [gap for gap in gaps if "distinct" in gap]
        entry["resolutionGaps"] = [gap for gap in gaps if "distinct" not in gap]
        animations.append(entry)
    avatar = (folder / "portrait.png").is_file()
    gaps = [gap for entry in animations for gap in entry["uniqueGaps"] + entry["resolutionGaps"]]
    if not avatar:
        gaps.insert(0, "dedicated avatar missing")
    return animations, gaps, avatar


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=None)
    parser.add_argument("--targets", action="store_true", help="print regeneration target frame names per fighter")
    args = parser.parse_args()
    defaults = json.loads((ROOT / "crates/engine/src/native/defaults.json").read_text(encoding="utf-8"))
    fighters = []
    incomplete = []
    for path in sorted(CHARS.glob("*.json")):
        if path.stem == "index":
            continue
        char = json.loads(path.read_text(encoding="utf-8"))
        if "id" not in char:
            continue
        animations, gaps, avatar = audit_character(char, defaults)
        fighters.append({"id": char["id"], "avatar": avatar, "animations": animations, "gaps": gaps, "character": char})
        if gaps:
            incomplete.append(char["id"])
    report = {"scope": str(CHARS), "fighters": [{key: value for key, value in fighter.items() if key != "character"} for fighter in fighters]}
    print(f"{len(fighters)} fighters, {len(incomplete)} with gaps: {', '.join(incomplete)}")
    for fighter in fighters:
        counts = {}
        for entry in fighter["animations"]:
            for frame in entry["frames"]:
                counts[frame["resolution"]] = counts.get(frame["resolution"], 0) + 1
        print(f"{fighter['id']:10} {json.dumps(counts, sort_keys=True)}")
        if args.targets:
            targets = targets_for(fighter["character"], defaults)
            print(f"{fighter['id']:10} targets ({len(targets)}): {', '.join(targets)}")

    if args.out:
        args.out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    if incomplete:
        sys.exit(2)


if __name__ == "__main__":
    main()
