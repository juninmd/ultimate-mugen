import json
import os
import re
from collections import Counter

from .branches import expand
from .abilities import is_fly_state, detect_abilities, detect_cancels
from .moves import STANDARD, anim_name, build_move
from .routes import convert_input, routes
from .coverage import coverage
from .scripts import helper_closure, passthrough
from .spawns import attach, seeds as spawn_seeds
from .parse import contain_axis, nums, png_bytes, read_air, read_commands, read_def, read_snd, read_states, sections, read_text, sprite_image
from .sff import SFF, read_act

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
    sff.legacy_shared = not d["info"].get("mugenversion", "").strip().startswith("1")
    pal = read_act(open(d["pal"], "rb").read()) if d["pal"] and sff.version == 1 else None
    actions = read_air(d["anim"])
    commands = read_commands(d["cmd"])
    states = read_states([d["cmd"]] + d["cns"])
    cmd_ctrls = states.get(-1, {"ctrl": []})["ctrl"]
    fly_states = [sid for sid, st in states.items() if is_fly_state(st)]
    add = json.load(open(extra, encoding="utf-8")) if extra else {}
    unlocked = [(r["state"], r["id"], r["input"], r.get("when", "stand"), r.get("cost", 0)) for r in add.pop("mugenRoutes", [])]
    raw_routes = unlocked + [(v, lb, convert_input(tx), wh, co) for v, lb, tx, wh, co in routes(cmd_ctrls, commands, fly_states)]
    abilities, consumed, ab_anims = detect_abilities(states, raw_routes, actions, anim_name)
    shadowed = set()
    moves, wanted, seen, state_to_id = [], [n for n in STANDARD if n in actions] + ab_anims, set(), {}
    for value, label, cmd_input, when, cost in raw_routes:
        if value < 200 or value not in states or value in consumed:
            continue
        move_id = re.sub(r"\W+", "_", label if when == "stand" else f"{when}_{label}").strip("_").lower()
        if (cmd_input, when) in seen or any(m["id"] == move_id for m in moves):
            shadowed.add(value)
            continue
        move, anims = build_move(value, states, actions, move_id, cmd_input, when, cost, skip=consumed)
        if move:
            seen.add((cmd_input, when))
            moves.append(move)
            state_to_id.setdefault(value, move_id)
            wanted += [a for a in anims if a in actions]
    branch_anims, branch_commands = expand(moves, states, actions, commands, consumed, state_to_id)
    wanted += [a for a in branch_anims if a in actions]
    cancels = detect_cancels(cmd_ctrls, state_to_id)
    dropped = Counter(k for move in moves for k in move.pop("_dropped", []))
    for move in moves:
        if move["id"] in cancels:
            move["cancels"] = cancels[move["id"]]
    remap = helper_closure(spawn_seeds(moves), states)
    dropped.update(attach(moves, remap))
    scripts, script_anims = passthrough(states, commands, actions, anim_name, set(state_to_id) | consumed, remap, {s: m for s, m in state_to_id.items() if m in {mv['id'] for mv in moves}})
    wanted += script_anims
    if branch_commands:
        scripts = scripts or {}
        scripts["commands"] = scripts.get("commands", []) + branch_commands
    hit_anims = {m["anim"] for m in moves} | {anim_name(a) for a in script_anims} | {p["anim"] for m in moves for p in ([m["projectile"]] if isinstance(m.get("projectile"), dict) else m.get("projectile", []))}
    no_loop = {"intro", "taunt"} | {m["anim"] for m in moves}
    anims = {}
    for number in dict.fromkeys(wanted):
        name_ = anim_name(number)
        frames = [frame_json(fr, name_ in hit_anims) for fr in actions[number]["frames"]]
        if name_ in no_loop:
            frames = [f.replace("*-1", "*6") if isinstance(f, str) else ({**f, "ticks": 6} if f.get("ticks", 1) < 0 else f) for f in frames]
        anims[name_] = {"frames": frames}
        if actions[number]["loop"] is not None and name_ not in no_loop:
            anims[name_]["loop"] = actions[number]["loop"]
    os.makedirs(os.path.join(out, "sprites"), exist_ok=True)
    meta = {"frames": {}}
    used = set(sff.index)
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
    wanted_snd = sorted(bank)
    for g, i in wanted_snd:
        open(os.path.join(out, "sprites", f"snd_{g}_{i}.wav"), "wb").write(bank[(g, i)])
    meta["sounds"] = [list(k) for k in wanted_snd]
    for m in moves:
        m["sounds"] = [s for s in m.get("sounds", []) if s.get("fight") or tuple(s["snd"]) in bank]
        if not m["sounds"]:
            m.pop("sounds")
    json.dump(meta, open(os.path.join(out, "sprites", "meta.json"), "w"), indent=1)
    c = constants(d["cns"])
    author = d["info"].get("author", "").strip('"') or "M.U.G.E.N"
    fighter = {
        "format": "umf/2", "id": fighter_id, "name": name, "series": "M.U.G.E.N", "author": author,
        "description": f"{name} converted from M.U.G.E.N to UMF 2.0.", "color": "#c8321e",
        "intro": "", "taunt": "", "winQuote": "",
        "stats": stats_from(c, ref), "art": {"type": "sprites", "style": {"boxes": "exact"}, "poses": {}},
        "anims": anims, "moves": moves,
    }
    if abilities:
        fighter["abilities"] = abilities
    if scripts:
        fighter["scripts"] = scripts
    if add:
        known = {m["id"]: m for m in fighter["moves"]}
        for extra in add.pop("moves", []):
            known[extra["id"]].update(extra) if extra["id"] in known else fighter["moves"].append(extra)
        fighter["anims"].update(add.pop("anims", {}))
        if isinstance(add.get("scripts"), dict) and fighter.get("scripts"):
            mine, theirs = fighter["scripts"], add.pop("scripts")
            cns = [c for c in (mine.get("cns"), theirs.get("cns")) if c]
            fighter["scripts"] = {**mine, **theirs, "cns": cns, "commands": mine.get("commands", []) + theirs.get("commands", [])}
        for key in ("profile", "abilities", "helpers", "passives", "scripts"):
            if isinstance(add.get(key), dict) and isinstance(fighter.get(key), dict):
                fighter[key] = {**fighter[key], **add.pop(key)}
        fighter.update(add)
    json.dump(fighter, open(os.path.join(out, f"{fighter_id}.json"), "w", encoding="utf-8"), ensure_ascii=False)
    by_id = {m["id"]: m for m in moves}
    pairs = [{"id": mid, "state": sid, "when": by_id[mid].get("when", "stand"), "listed": bool(by_id[mid].get("input"))} for sid, mid in state_to_id.items() if mid in by_id]
    json.dump(pairs, open(os.path.join(out, "routes.json"), "w", encoding="utf-8"), indent=1)
    sprite_files = sum(1 for g, i in sff.index if os.path.exists(os.path.join(out, "sprites", f"s{g}_{i}.png")))
    report = coverage(states, commands, actions, sff, bank, state_to_id, consumed, set(remap) | {int(x) for x in re.findall(r"\[Statedef (\d+)\]", (scripts or {}).get("cns") or "")}, fighter, sprite_files, shadowed, dropped)
    open(os.path.join(out, "coverage.md"), "w", encoding="utf-8").write(report)
    return fighter


def feature_counts(f):
    ms = f["moves"]
    return {
        "moves": len(ms),
        "anims": len(f["anims"]),
        "abilities": sorted((f.get("abilities") or {}).keys()),
        "explods": sum(len(m.get("explods", [])) for m in ms),
        "afterimages": sum(1 for m in ms if "afterimage" in m),
        "shakes": sum(1 for m in ms if "shake" in m),
        "flashes": sum(1 for m in ms if "flash" in m),
        "teleports": sum(len(m["teleport"]) if isinstance(m.get("teleport"), list) else (1 if "teleport" in m else 0) for m in ms),
        "throws": sum(1 for m in ms if "throw" in m),
        "followUps": sum(1 for m in ms if "followUp" in m),
        "cancels": sum(1 for m in ms if "cancels" in m),
        "wallBounces": sum(1 for m in ms for h in m.get("hits", []) if h.get("wallBounce")),
        "projectiles": sum(1 for m in ms if "projectile" in m),
        "motion": sum(1 for m in ms for k in ("velMul", "velAdd", "posFreeze", "posSet", "palFx") if k in m),
    }
