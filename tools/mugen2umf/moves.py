import re

from .abilities import detect_throw
from .chain import chain, lands, segment, state_anim, triggers
from .fx import extract_fx
from .hitdefs import STANDARD, anim_name, hit_from, pair, projectile_from
from .motion import motion
from .parse import nums


def frame_at(ctrl, frames):
    lines = triggers(ctrl)
    elems = [int(m) for line in lines for m in re.findall(r"animelem\s*=\s*(\d+)", line)]
    if elems:
        return sorted(set(elems))
    for line in lines:
        m = re.search(r"(?<!anim)time\s*=\s*(\d+)", line)
        if m:
            t, acc = int(m.group(1)), 0
            for k, fr in enumerate(frames):
                acc += max(fr["t"], 1)
                if acc > t:
                    return [k + 1]
            return [len(frames)]
    return []


def first_clsn1(frames):
    for k, fr in enumerate(frames):
        if fr["clsn1"]:
            return k + 1
    return 1


def sound_from(ctrl, at):
    value = ctrl["body"].get("value", "").strip()
    fight = value.upper().startswith("F")
    s = nums(value.lstrip("FfSs"), (0, 0))
    out = {"at": at, "snd": [int(s[0]), int(s[1])]}
    if fight:
        out["fight"] = True
    return out


def _unflag(text, flags):
    for var, name in flags.items():
        text = re.sub(re.escape(var) + r"\s*(=\s*1\b)?", f'command = "{name}"', text)
    return text


def _fits(ctrl, when):
    groups = {}
    for key, value in ctrl["pairs"]:
        if key.startswith("trigger"):
            groups.setdefault(key[7:], []).append(re.sub(r"\s+", "", value.lower()))
    wrong = r"statetype(!=a|=[sc])" if when == "air" else r"statetype=a\b"
    excluded = lambda g: any(re.search(wrong, t) for t in groups.get(g, []))
    numbered = [g for g in groups if g != "all"]
    return not (excluded("all") or (numbered and all(excluded(g) for g in numbered)))


def _ticks(frames):
    return sum(max(f["t"], 1) for f in frames)


def build_move(state_no, states, actions, label, cmd_input, when, cost, skip=frozenset()):
    links, confirm = chain(state_no, states, actions, skip=skip)
    links = [n for n in links if state_anim(states[n]) in actions]
    if not links:
        return None, []
    anim_no = state_anim(states[links[0]])
    state, landing = {"def": states[state_no]["def"], "ctrl": []}, {}
    if len(links) > 1:
        frames, ctrl = [], []
        for k, n in enumerate(links):
            base = len(frames)
            if k and lands(states[links[k - 1]], n) and "landAt" not in landing:
                landing["landAt"] = base + 1
            seg = segment(states[n], actions[state_anim(states[n])], links[k + 1] if k + 1 < len(links) else None)
            for c in states[n]["ctrl"]:
                elems = frame_at(c, actions[state_anim(states[n])]["frames"])
                ctrl.append(dict(c, at=[e + base for e in elems], span=(base + 1, base + len(seg)), ticks=(_ticks(frames), _ticks(frames + seg))))
            frames += seg
        anim_no = 900000 + state_no
        actions[anim_no] = {"frames": frames, "loop": None}
        state["ctrl"] = ctrl
    else:
        total = _ticks(actions[anim_no]["frames"])
        state["ctrl"] = [dict(c, at=frame_at(c, actions[anim_no]["frames"]), span=(1, 0), ticks=(0, total)) for c in states[state_no]["ctrl"]]
    frames = actions[anim_no]["frames"]
    flags = {f"var({int(nums(c['body']['v'])[0])})": m.group(1) for c in state["ctrl"] if c["type"] == "varset" and c["body"].get("v")
             for m in [re.search(r'command\s*=\s*"([^"]+)"', " ".join(triggers(c)))] if m}
    branches = [{"target": int(nums(c["body"].get("value"), (-1,))[0]), "text": _unflag(" ".join(triggers(c)), flags), "at": c["at"], "t0": c["ticks"][0], "t1": c["ticks"][1],
                 "buffered": any(f in " ".join(triggers(c)) for f in flags)}
                for c in state["ctrl"] if c["type"] == "changestate" and "command" in _unflag(" ".join(triggers(c)), flags)]
    branches = [b for b in branches if b["target"] in states and b["target"] not in links and b["target"] >= 200]
    move = {"id": label, "name": label.replace("_", " ").title(), "input": cmd_input, "kind": "normal", "anim": anim_name(anim_no)}
    if when != "stand":
        move["when"] = when
    hits, projectiles, moves, sounds, spawns, extra = [], [], [], [], [], [anim_no]
    for ctrl in (c for c in state["ctrl"] if _fits(c, when)):
        at, kind = ctrl["at"], ctrl["type"]
        if kind == "hitdef":
            for a in at or [first_clsn1(frames)]:
                hits.append(hit_from(ctrl, a, states))
        elif kind == "projectile":
            for a in at or [1]:
                proj, anims = projectile_from(ctrl, a, actions)
                projectiles.append(proj)
            extra += anims
        elif kind in ("velset", "veladd") and at and ("x" in ctrl["body"] or "y" in ctrl["body"]):
            x = nums(ctrl["body"].get("x"), (0,))[0] if "x" in ctrl["body"] else None
            y = nums(ctrl["body"].get("y"), (0,))[0] if "y" in ctrl["body"] else None
            moves.append({"at": at[0], "vel": [x, y]})
        elif kind == "helper" and re.fullmatch(r"\s*\d+\s*", ctrl["body"].get("stateno", "")):
            spawns.append({"state": int(ctrl["body"]["stateno"]), "at": (at or [ctrl["span"][0]])[0], "body": ctrl["body"]})
        elif kind == "playsnd":
            sounds += [sound_from(ctrl, a) for a in at or [1]]
    fx, fx_anims = extract_fx(state["ctrl"], actions, anim_name, frames)
    extra += fx_anims
    throw, throw_anims = detect_throw(state_no, states, actions, anim_name)
    if throw:
        move["throw"] = throw
        extra += throw_anims
    elif hits:
        move["hits"] = hits
    if projectiles:
        move["projectile"] = projectiles[0] if len(projectiles) == 1 else projectiles
    if moves:
        move["moves"] = moves
    if sounds:
        move["sounds"] = sounds
    if fx["explods"]:
        move["explods"] = fx["explods"]
    if fx["afterimage"]:
        move["afterimage"] = fx["afterimage"]
    if fx["shakes"]:
        move["shake"] = fx["shakes"][0] if len(fx["shakes"]) == 1 else fx["shakes"]
    if fx["flashes"]:
        move["flash"] = fx["flashes"][0] if len(fx["flashes"]) == 1 else fx["flashes"]
    if fx["super_pause"]:
        move["superPause"] = fx["super_pause"]
    if fx["invuln"]:
        move["invuln"] = fx["invuln"]
    if fx["teleport"]:
        move["teleport"] = fx["teleport"]
    move.update(landing)
    motion_fields, move["_dropped"] = motion(state["ctrl"])
    move.update(motion_fields)
    p1_hit = next((int(nums(c["body"].get("p1stateno"), (-1,))[0]) for c in state["ctrl"] if c["type"] == "hitdef" and c["body"].get("p1stateno")), None)
    if not throw and p1_hit in states and p1_hit not in links:
        move["_followUpState"] = (p1_hit, "hitNow")
    elif confirm:
        move["_followUpState"] = confirm
    power = nums(state["def"].get("poweradd"), (0,))[0]
    if fx["super_pause"] or power <= -1000 or cost >= 1000:
        move["kind"] = "super"
        move["cost"] = max(cost, 1000)
    elif "," in cmd_input or state_no >= 1000:
        move["kind"] = "special"
        if cost:
            move["cost"] = cost
    if branches:
        move["_branches"] = branches
    if spawns:
        move["_spawns"] = spawns
    return move, extra
