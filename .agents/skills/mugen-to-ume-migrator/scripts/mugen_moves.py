import re

from mugen_fx import MAX_VFX, quake_from, vfx_from
from mugen_parse import nums

STANDARD = {
    0: "idle", 5: "turn", 6: "crouchTurn", 10: "crouchStart", 11: "crouch", 12: "crouchEnd", 20: "walk", 21: "walkBack",
    40: "jumpStart", 41: "jumpUp", 42: "jumpFwd", 43: "jumpBack", 44: "jumpDown", 47: "land", 100: "run", 105: "hop",
    120: "guardStart", 121: "crouchGuardStart", 122: "airGuardStart", 130: "guard", 131: "crouchGuard", 132: "airGuard",
    140: "guardEnd", 150: "guardHit", 151: "airGuardHit", 152: "crouchGuardHit", 170: "lose", 175: "draw", 181: "win",
    190: "intro", 195: "taunt", 5000: "hitLight", 5001: "hitMedium", 5002: "hitHeavy", 5010: "hitLowLight",
    5011: "hitLowMedium", 5012: "hitLowHeavy", 5020: "hitCrouch", 5030: "hitAir", 5040: "airRecover", 5050: "fall",
    5070: "trip", 5100: "bounce", 5101: "bounceAir", 5110: "lieDown", 5120: "getUp", 5150: "dead",
}
LEVEL = {"light": "light", "medium": "medium", "med": "medium", "hard": "heavy", "heavy": "heavy", "back": "heavy", "up": "heavy", "diagup": "heavy"}


def anim_name(number):
    return f"seq{number - 900000}" if number >= 900000 else STANDARD.get(number, f"a{number}")


def triggers(ctrl):
    return [v.lower() for k, v in ctrl["pairs"] if k.startswith("trigger")]


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


def pair(text, default=(0.0, 0.0)):
    values = nums(text, default)
    return [round(values[0], 2), round(values[1], 2)]


def hit_from(ctrl, at):
    b = ctrl["body"]
    damage = nums(b.get("damage"), (0, 0))
    hit = {"at": at, "damage": max(round(damage[0]), 1)}
    if len(damage) > 1 and damage[1]:
        hit["guardDamage"] = round(damage[1])
    level = LEVEL.get(b.get("animtype", "light").strip().lower())
    if level:
        hit["level"] = level
    flag = b.get("guardflag", "MA").upper()
    if "L" in flag and "H" not in flag:
        hit["guard"] = "low"
    elif "H" in flag and "L" not in flag:
        hit["guard"] = "high"
    elif not flag.strip():
        hit["guard"] = "unblockable"
    if b.get("pausetime"):
        p = nums(b["pausetime"], (0, 0))
        hit["pausetime"] = [int(p[0]), int(p[1])]
    if b.get("ground.hittime"):
        hit["hitstun"] = int(nums(b["ground.hittime"], (12,))[0])
    if b.get("ground.velocity"):
        hit["knockback"] = pair(b["ground.velocity"])
    if b.get("air.velocity"):
        hit["airKnockback"] = pair(b["air.velocity"])
    if b.get("fall", "0").strip().startswith("1") or b.get("ground.type", "").strip().lower() == "trip":
        hit["launch"] = True
    sound = b.get("hitsound", "").strip()
    if sound and not sound.upper().startswith("S"):
        s = nums(sound, (5, 0))
        hit["hitSound"] = [int(s[0]), int(s[1])]
    if b.get("hitflag"):
        hit["hitflag"] = b["hitflag"].strip().upper()
    return hit


def projectile_from(ctrl, at, actions):
    b = ctrl["body"]
    proj = {"at": at, "anim": anim_name(int(nums(b.get("projanim"), (0,))[0])), "speed": pair(b.get("velocity"), (4, 0)),
            "offset": pair(b.get("offset")), "damage": max(round(nums(b.get("damage"), (30,))[0]), 1)}
    hit_anim = int(nums(b.get("projhitanim"), (-1,))[0])
    if hit_anim in actions:
        proj["hitAnim"] = anim_name(hit_anim)
    for key, field in (("projhits", "hits"), ("projpriority", "priority"), ("projremovetime", "removeTime")):
        if b.get(key):
            proj[field] = int(nums(b[key], (1,))[0])
    level = LEVEL.get(b.get("animtype", "").strip().lower())
    if level:
        proj["level"] = level
    return proj, [int(nums(b.get("projanim"), (0,))[0])] + ([hit_anim] if hit_anim in actions else [])


def sound_from(ctrl, at):
    value = ctrl["body"].get("value", "").strip()
    fight = value.upper().startswith("F")
    s = nums(value.lstrip("FfSs"), (0, 0))
    out = {"at": at, "snd": [int(s[0]), int(s[1])]}
    if fight:
        out["fight"] = True
    return out


def state_anim(state):
    anim_no = int(nums(state["def"].get("anim"), (-1,))[0]) if state["def"].get("anim") else None
    for ctrl in state["ctrl"]:
        if anim_no is None and ctrl["type"] == "changeanim":
            anim_no = int(nums(ctrl["body"].get("value"), (0,))[0])
    return anim_no


def chain(state_no, states, actions, limit=4):
    out = [state_no]
    while len(out) < limit:
        nxt = [int(nums(c["body"].get("value"), (0,))[0]) for c in states[out[-1]]["ctrl"] if c["type"] == "changestate" and "command" not in " ".join(triggers(c))]
        nxt = [n for n in nxt if n >= 200 and n not in out and n in states and state_anim(states[n]) in actions]
        if not nxt:
            break
        out.append(nxt[0])
    return out


def build_move(state_no, states, actions, label, cmd_input, when, cost):
    links = [n for n in chain(state_no, states, actions) if state_anim(states[n]) in actions]
    if not links:
        return None, []
    anim_no = state_anim(states[links[0]])
    state = {"def": states[state_no]["def"], "ctrl": []}
    if len(links) > 1:
        frames, ctrl = [], []
        for n in links:
            base = len(frames)
            for c in states[n]["ctrl"]:
                elems = frame_at(c, actions[state_anim(states[n])]["frames"])
                ctrl.append(dict(c, at=[e + base for e in elems]))
            frames += actions[state_anim(states[n])]["frames"]
        anim_no = 900000 + state_no
        actions[anim_no] = {"frames": frames, "loop": None}
        state["ctrl"] = ctrl
    else:
        state["ctrl"] = [dict(c, at=frame_at(c, actions[anim_no]["frames"])) for c in states[state_no]["ctrl"]]
    frames = actions[anim_no]["frames"]
    move = {"id": label, "name": label.replace("_", " ").title(), "input": cmd_input, "kind": "normal", "anim": anim_name(anim_no)}
    if when != "stand":
        move["when"] = when
    hits, projectiles, moves, sounds, vfx, quake, extra = [], [], [], [], [], [], [anim_no]
    super_pause = False
    for ctrl in state["ctrl"]:
        at = ctrl["at"]
        kind = ctrl["type"]
        if kind == "hitdef":
            for a in at or [first_clsn1(frames)]:
                hits.append(hit_from(ctrl, a))
        elif kind == "projectile":
            proj, anims = projectile_from(ctrl, (at or [1])[0], actions)
            projectiles.append(proj)
            extra += anims
        elif kind in ("velset", "veladd") and at and ("x" in ctrl["body"] or "y" in ctrl["body"]):
            x = nums(ctrl["body"].get("x"), (0,))[0] if "x" in ctrl["body"] else None
            y = nums(ctrl["body"].get("y"), (0,))[0] if "y" in ctrl["body"] else None
            moves.append({"at": at[0], "vel": [x, y]})
        elif kind in ("posadd", "posset") and "p2dist" in " ".join(ctrl["body"].values()).lower() and at:
            move["teleport"] = at[0]
        elif kind == "explod" and at and len(vfx) < MAX_VFX:
            effect, anims = vfx_from(ctrl, at[0], actions, anim_name)
            if effect:
                vfx.append(effect)
                extra += anims
        elif kind == "envshake" and at and len(quake) < 2:
            quake.append(quake_from(ctrl, at[0]))
        elif kind == "playsnd":
            sounds.append(sound_from(ctrl, (at or [1])[0]))
        elif kind == "superpause":
            super_pause = True
    if hits:
        move["hits"] = hits
    if projectiles:
        move["projectile"] = projectiles[0] if len(projectiles) == 1 else projectiles
    if moves:
        move["moves"] = moves
    if "fly" in label:
        move["flight"] = True
    if vfx:
        move["vfx"] = vfx
    if quake:
        move["quake"] = quake
    if sounds:
        move["sounds"] = sounds
    power = nums(state["def"].get("poweradd"), (0,))[0]
    if super_pause or power <= -1000 or cost >= 1000:
        move["kind"] = "super"
        move["cost"] = max(cost, 1000)
    elif "," in cmd_input or state_no >= 1000:
        move["kind"] = "special"
        if cost:
            move["cost"] = cost
    return move, extra


def convert_input(text):
    parts = [p.strip() for p in text.split(",")]
    if len(parts) == 2 and parts[0].startswith("/"):
        direction = parts[0].lstrip("/$~")
        return f"{direction}+{parts[1].lstrip('/$')}"
    return ", ".join(p.replace("/", "").replace("$", "") if not re.match(r"~\d+\$", p) else p for p in parts)


def trigger_groups(ctrl):
    shared, groups = [], {}
    for key, value in ctrl["pairs"]:
        if key == "triggerall":
            shared.append(value.lower())
        elif key.startswith("trigger") and key[7:].isdigit():
            groups.setdefault(int(key[7:]), []).append(value.lower())
    return [" & ".join(shared + lines) for _, lines in sorted(groups.items())] or [" & ".join(shared)]


def routes(cmd_states, commands):
    commands = {k.lower(): v for k, v in commands.items()}
    out = []
    for ctrl in cmd_states:
        if ctrl["type"] != "changestate":
            continue
        value = int(nums(ctrl["body"].get("value"), (-1,))[0])
        usable = [g for g in trigger_groups(ctrl) if not re.search(r"ctrl\s*=\s*0|stateno\s*=\s*3001", g)]
        if not usable:
            continue
        text = usable[0]
        names = re.findall(r'command\s*=\s*"([^"]+)"', text)
        hold_only = all(re.fullmatch(r"/\$?\w+", c.strip()) for c in commands.get(names[0], ["x"])) if names else False
        if not names or hold_only or names[0] not in commands:
            continue
        state_type = re.search(r"statetype\s*=\s*([sca])", text)
        when = {"a": "air", "c": "crouch"}.get(state_type.group(1) if state_type else "s", "stand")
        cost = int(max([float(x) for x in re.findall(r"power\s*>=?\s*(\d+)", text)] or [0]))
        out.append((value, names[0], commands[names[0]][0], when, cost))
    return out
