"""Convert a M.U.G.E.N character folder into a umf/2 bundle: <id>.json plus sprites/ (poses, meta.json, sounds)."""
import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mugen_moves import STANDARD, anim_name, build_move, convert_input, routes
from mugen_parse import contain_axis, nums, png_bytes, read_air, read_commands, read_def, read_snd, read_states, sections, read_text, sprite_image
from sff_decode import SFF, read_act

ACTIVE_PART = {"kick": "footF"}


def frame_json(fr, has_hit):
    pose = f"s{fr['g']}_{fr['i']}"
    out = {"pose": pose, "ticks": fr["t"] if fr["t"] != 0 else 1}
    if fr["x"] or fr["y"]:
        out["offset"] = [fr["x"], fr["y"]]
    if fr["clsn1"] and has_hit:
        x1 = min(b[0] for b in fr["clsn1"]); y1 = min(b[1] for b in fr["clsn1"])
        x2 = max(b[2] for b in fr["clsn1"]); y2 = max(b[3] for b in fr["clsn1"])
        out["hit"] = {"part": "handF", "box": [x1, y1, x2, y2]}
    if fr["clsn2"]:
        out["hurt"] = fr["clsn2"]
    if set(out) == {"pose", "ticks"}:
        return f"{pose}*{out['ticks']}"
    return out


def constants(paths):
    data = {}
    for path in paths:
        for name, body, _ in sections(read_text(path)):
            if name.lower() in ("data", "size", "velocity", "movement"):
                for k, v in body.items():
                    data.setdefault(k, v)
    return data


def stats_from(c, height):
    n = lambda key, d: nums(c.get(key), d)
    return {
        "life": int(n("life", (1000,))[0]), "power": 3000, "attack": int(n("attack", (100,))[0]), "defense": int(n("defence", (100,))[0]),
        "walk": [abs(n("walk.fwd", (2.4,))[0]), abs(n("walk.back", (2.2,))[0])],
        "run": [abs(n("run.fwd", (4.6,))[0]), abs(n("run.back", (4.5,))[0])],
        "jump": [n("jump.neu", (0, -8.4))[1], n("jump.back", (-2.55,))[0], n("jump.neu", (0, -8.4))[0], n("jump.fwd", (2.5,))[0]],
        "height": height, "width": [int(n("ground.front", (16,))[0]), int(n("ground.back", (15,))[0])],
        "airJumps": int(n("airjump.num", (1,))[0]),
    }


def convert(src, out, fighter_id, name, extra):
    d = read_def(src)
    sff = SFF(open(d["sprite"], "rb").read())
    pal = read_act(open(d["pal"], "rb").read()) if d["pal"] and sff.version == 1 else None
    actions = read_air(d["anim"])
    commands = read_commands(d["cmd"])
    states = read_states([d["cmd"]] + d["cns"])
    moves, wanted, seen = [], [n for n in STANDARD if n in actions], set()
    for value, label, text, when, cost in routes(states.get(-1, {"ctrl": []})["ctrl"], commands):
        if value < 200 or value not in states:
            continue
        cmd_input = convert_input(text)
        move_id = re.sub(r"\W+", "_", label if when == "stand" else f"{when}_{label}").strip("_").lower()
        if (cmd_input, when) in seen or any(m["id"] == move_id for m in moves):
            continue
        move, anims = build_move(value, states, actions, move_id, cmd_input, when, cost)
        if move:
            seen.add((cmd_input, when))
            moves.append(move)
            wanted += [a for a in anims if a in actions]
    hit_anims = {m["anim"] for m in moves} | {p["anim"] for m in moves for p in ([m["projectile"]] if isinstance(m.get("projectile"), dict) else m.get("projectile", []))}
    anims = {}
    for number in dict.fromkeys(wanted):
        name_ = anim_name(number)
        frames = [frame_json(fr, name_ in hit_anims) for fr in actions[number]["frames"]]
        anims[name_] = {"frames": frames}
        if actions[number]["loop"] is not None:
            anims[name_]["loop"] = actions[number]["loop"]
    os.makedirs(os.path.join(out, "sprites"), exist_ok=True)
    meta, used = {"frames": {}}, set()
    for number in dict.fromkeys(wanted):
        for fr in actions[number]["frames"]:
            used.add((fr["g"], fr["i"]))
    idle, axis = sprite_image(sff, pal, *(lambda f: (f["g"], f["i"]))(actions[0]["frames"][0]))
    box = idle.getbbox()
    ref = float(box[3] - box[1])
    for g, i in sorted(used):
        image, axis = sprite_image(sff, pal, g, i)
        if image is None:
            continue
        image, axis = contain_axis(image, axis)
        pose = f"s{g}_{i}"
        open(os.path.join(out, "sprites", pose + ".png"), "wb").write(png_bytes(image))
        meta["frames"][pose] = {"axisX": float(axis[0]), "axisY": float(axis[1]), "refHeight": ref}
    first = actions[0]["frames"][0]
    open(os.path.join(out, "sprites", "01_idle.png"), "wb").write(png_bytes(idle))
    meta["frames"]["01_idle"] = meta["frames"][f"s{first['g']}_{first['i']}"]
    meta["refHeight"] = ref
    portrait, _ = sprite_image(sff, None, 9000, 1)
    if portrait is not None:
        open(os.path.join(out, "sprites", "portrait.png"), "wb").write(png_bytes(portrait))
    bank = read_snd(d["sound"]) if d["sound"] else {}
    wanted_snd = sorted({tuple(s["snd"]) for m in moves for s in m.get("sounds", []) if not s.get("fight")} & set(bank))
    for g, i in wanted_snd:
        open(os.path.join(out, "sprites", f"snd_{g}_{i}.wav"), "wb").write(bank[(g, i)])
    meta["sounds"] = [list(k) for k in wanted_snd]
    for m in moves:
        m["sounds"] = [s for s in m.get("sounds", []) if s.get("fight") or tuple(s["snd"]) in bank]
        if not m["sounds"]:
            m.pop("sounds")
    json.dump(meta, open(os.path.join(out, "sprites", "meta.json"), "w"), indent=1)
    c = constants(d["cns"])
    fighter = {"format": "umf/2", "id": fighter_id, "name": name, "series": "M.U.G.E.N", "author": d["info"].get("author", "").strip('"'),
               "color": "#c8321e", "stats": stats_from(c, ref), "art": {"type": "sprites", "style": {"boxes": "exact"}, "poses": {}},
               "anims": anims, "moves": moves}
    if extra:
        add = json.load(open(extra))
        fighter["moves"] += add.pop("moves", [])
        fighter["anims"].update(add.pop("anims", {}))
        fighter.update(add)
    json.dump(fighter, open(os.path.join(out, f"{fighter_id}.json"), "w"), ensure_ascii=False)
    return fighter


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("src")
    ap.add_argument("out")
    ap.add_argument("--id", required=True)
    ap.add_argument("--name", required=True)
    ap.add_argument("--extra", help="json merged on top: extra moves, anims, abilities, helpers")
    a = ap.parse_args()
    f = convert(a.src, a.out, a.id, a.name, a.extra)
    print(f"{a.id}: {len(f['moves'])} moves, {len(f['anims'])} anims, height {f['stats']['height']}")
