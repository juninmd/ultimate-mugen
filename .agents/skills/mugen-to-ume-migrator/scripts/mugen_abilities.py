import re

from mugen_parse import nums


def _trigs(ctrl):
    return [v.lower() for k, v in ctrl["pairs"] if k.startswith("trigger")]


def _state_anim(state):
    anim = int(nums(state["def"].get("anim"), (-1,))[0]) if state["def"].get("anim") else None
    for c in state["ctrl"]:
        if anim is None and c["type"] == "changeanim":
            anim = int(nums(c["body"].get("value"), (-1,))[0])
    return anim if anim is not None and anim >= 0 else None


def _is_fly_state(state):
    if not state:
        return False
    dirs = set()
    for c in state["ctrl"]:
        if c["type"] in ("velset", "veladd"):
            for t in _trigs(c):
                for d in ("holdfwd", "holdback", "holdup", "holddown"):
                    if d in t:
                        dirs.add(d)
    return len(dirs) >= 3


def _is_charge_state(state):
    if not state or state["def"].get("movetype", "i").lower() == "a":
        return False
    gains = any(c["type"] == "poweradd" and nums(c["body"].get("value"), (0,))[0] > 0 for c in state["ctrl"])
    releases = any(c["type"] == "changestate" and any("command !=" in t or "power >=" in t for t in _trigs(c)) for c in state["ctrl"])
    return gains and releases


def _is_pure_teleport(state):
    if not state or state["def"].get("movetype", "i").lower() == "a":
        return False
    for c in state["ctrl"]:
        if c["type"] in ("posadd", "posset"):
            vals = " ".join(c["body"].values()).lower()
            if "p2dist" in vals or abs(nums(c["body"].get("x"), (0,))[0]) >= 80:
                return True
    return False


def _dir_anims(state, actions):
    out = {}
    for c in state["ctrl"]:
        if c["type"] != "changeanim":
            continue
        val = int(nums(c["body"].get("value"), (-1,))[0])
        if val not in actions:
            continue
        t = " ".join(_trigs(c))
        for cmd, key in (("holdup", "upAnim"), ("holddown", "downAnim"), ("holdfwd", "fwdAnim"), ("holdback", "backAnim")):
            if re.search(rf'command\s*=\s*"{cmd}"', t) and key not in out:
                out[key] = val
    return out


def detect_abilities(states, cmd_routes, actions, anim_name_fn):
    abilities, consumed, extra_anims = {}, set(), []
    for sid, st in states.items():
        if _is_fly_state(st):
            consumed.add(sid)
    for value, label, cmd_input, when, cost in cmd_routes:
        st = states.get(value)
        if not st:
            continue
        if _is_fly_state(st) or "fly" in label.lower():
            consumed.add(value)
            if "fly" not in abilities:
                fly_st = states[next((s for s in consumed if _is_fly_state(states.get(s))), value)]
                fly = {"speed": 4.0, "input": cmd_input}
                if cost:
                    fly["cost"] = cost
                an = _state_anim(fly_st)
                if an in actions:
                    fly["anim"] = anim_name_fn(an)
                    extra_anims.append(an)
                for k, v in _dir_anims(fly_st, actions).items():
                    fly[k] = anim_name_fn(v)
                    extra_anims.append(v)
                abilities["fly"] = fly
        elif "teleport" not in abilities and _is_pure_teleport(st) and "teleport" in label.lower():
            tp = {"input": cmd_input}
            if cost:
                tp["cost"] = cost
            an = _state_anim(st)
            if an in actions:
                tp["anim"] = anim_name_fn(an)
                extra_anims.append(an)
            for c in st["ctrl"]:
                if c["type"] == "posadd":
                    elems = [int(m) for t in _trigs(c) for m in re.findall(r"animelem\s*=\s*(\d+)", t)]
                    if elems:
                        tp["at"] = elems[0]
                elif c["type"] == "playsnd" and "sound" not in tp:
                    s = nums(c["body"].get("value", "").lstrip("FfSs"), (0, 0))
                    tp["sound"] = [int(s[0]), int(s[1])]
            abilities["teleport"] = tp
            consumed.add(value)
    for sid, st in states.items():
        if _is_charge_state(st):
            ch = {"auraColor": [90, 70, 15]}
            an = _state_anim(st)
            if an in actions:
                ch["anim"] = anim_name_fn(an)
                extra_anims.append(an)
            if any(c["type"] == "envshake" for c in st["ctrl"]):
                ch["shake"] = True
            for c in st["ctrl"]:
                if c["type"] == "playsnd":
                    s = nums(c["body"].get("value", "").lstrip("FfSs"), (0, 0))
                    ch["sound"] = [int(s[0]), int(s[1])]
                    break
            abilities["charge"] = ch
            consumed.add(sid)
            break
    return abilities, consumed, extra_anims


def detect_throw(state_no, states, actions, anim_name_fn):
    st = states.get(state_no)
    if not st:
        return None, []
    for c in st["ctrl"]:
        if c["type"] != "hitdef":
            continue
        p1 = int(nums(c["body"].get("p1stateno"), (-1,))[0])
        p2 = int(nums(c["body"].get("p2stateno"), (-1,))[0])
        if p1 not in states:
            continue
        p1_st = states[p1]
        binds = [x for x in p1_st["ctrl"] if x["type"] == "targetbind"]
        if not binds:
            continue
        pos = nums(binds[0]["body"].get("pos"), (30, 0))
        dmg = 80
        for x in p1_st["ctrl"]:
            if x["type"] == "targetlifeadd":
                dmg = max(1, abs(round(nums(x["body"].get("value"), (-80,))[0])))
        throw = {"damage": dmg, "bind": [round(pos[0], 1), round(pos[1], 1)], "launch": [-4.5, -5.5]}
        extra = []
        if p2 in states:
            for x in states[p2]["ctrl"]:
                if x["type"] == "changeanim2":
                    an = int(nums(x["body"].get("value"), (-1,))[0])
                    if an in actions:
                        throw["p2Anim"] = anim_name_fn(an)
                        extra.append(an)
                        break
        return throw, extra
    return None, []


def detect_wall_bounce(ctrl, states):
    p2 = int(nums(ctrl["body"].get("p2stateno"), (-1,))[0])
    seen, queue = set(), [p2] if p2 in states else []
    while queue and len(seen) < 6:
        cur = queue.pop(0)
        if cur in seen or cur not in states:
            continue
        seen.add(cur)
        for c in states[cur]["ctrl"]:
            if any("backedgebodydist" in t or "frontedgebodydist" in t for t in _trigs(c)):
                return True
            if c["type"] == "changestate":
                nxt = int(nums(c["body"].get("value"), (-1,))[0])
                if nxt in states:
                    queue.append(nxt)
    return False


def extract_fx(ctrls, actions, anim_name_fn, frames):
    explods, shakes, flashes, teleports, extra_anims, seen_tp = [], [], [], [], [], set()
    afterimage, super_pause, invuln = None, None, None
    turn_frames = {(c["at"] or [1])[0] for c in ctrls if c["type"] == "turn"}
    for c in ctrls:
        kind, b, at = c["type"], c["body"], c["at"]
        frame = (at or [1])[0]
        if kind == "explod":
            an = int(nums(b.get("anim"), (-1,))[0])
            if an in actions:
                ex = {"at": frame, "anim": anim_name_fn(an)}
                if b.get("pos"):
                    p = nums(b["pos"], (0, 0))
                    ex["pos"] = [round(p[0], 1), round(p[1], 1)]
                if b.get("postype"):
                    ex["postype"] = b["postype"].strip().lower()
                if b.get("bindtime"):
                    ex["bind"] = int(nums(b["bindtime"], (1,))[0])
                if b.get("removetime"):
                    ex["removeTime"] = int(nums(b["removetime"], (-2,))[0])
                if b.get("sprpriority"):
                    ex["priority"] = int(nums(b["sprpriority"], (3,))[0])
                if b.get("scale"):
                    sc = nums(b["scale"], (1, 1))
                    ex["scale"] = [round(sc[0], 2), round(sc[1], 2)]
                if b.get("supermove", "").strip().startswith("1"):
                    ex["superMove"] = True
                explods.append(ex)
                extra_anims.append(an)
        elif kind == "afterimage" and afterimage is None:
            afterimage = {"at": frame, "time": int(nums(b.get("time"), (25,))[0]), "length": int(nums(b.get("length"), (8,))[0])}
            if b.get("framegap"):
                afterimage["framegap"] = int(nums(b["framegap"], (2,))[0])
            if b.get("trans"):
                afterimage["trans"] = b["trans"].strip().lower()
            for mk, uk in (("palbright", "palBright"), ("palcontrast", "palContrast"), ("paladd", "palAdd")):
                if b.get(mk):
                    v = nums(b[mk], (0, 0, 0))
                    afterimage[uk] = [int(v[0]), int(v[1]), int(v[2])]
        elif kind == "envshake":
            sh = {"at": frame, "time": max(1, int(nums(b.get("time"), (12,))[0]))}
            if b.get("ampl"):
                sh["ampl"] = round(abs(nums(b["ampl"], (4,))[0]), 1)
            if b.get("freq"):
                sh["freq"] = round(nums(b["freq"], (60,))[0], 1)
            shakes.append(sh)
        elif kind == "envcolor":
            col = nums(b.get("value"), (255, 255, 255))
            fl = {"at": frame, "time": max(1, int(nums(b.get("time"), (10,))[0])), "color": [int(col[0]), int(col[1]), int(col[2])]}
            if b.get("under", "").strip().startswith("1"):
                fl["under"] = True
            if b.get("supermove", "").strip().startswith("1"):
                fl["superMove"] = True
            flashes.append(fl)
        elif kind == "superpause" and super_pause is None:
            sp = {"at": frame, "time": max(1, int(nums(b.get("time"), (55,))[0]))}
            if b.get("pos"):
                p = nums(b["pos"], (0, -60))
                sp["pos"] = [round(p[0], 1), round(p[1], 1)]
            if b.get("anim"):
                sp["anim"] = int(nums(b["anim"], (30,))[0])
            super_pause = sp
        elif kind == "nothitby" and invuln is None:
            dur = max(1, int(nums(b.get("time"), (8,))[0]))
            end = min(len(frames), frame + max(1, dur // 4))
            invuln = [frame, max(frame, end)]
        elif kind == "posadd":
            vals = " ".join(b.values()).lower()
            x, y = round(nums(b.get("x"), (0,))[0], 1), round(nums(b.get("y"), (0,))[0], 1)
            key = (frame, x, y, "p2dist" in vals)
            if key in seen_tp:
                continue
            if "p2dist" in vals:
                seen_tp.add(key)
                teleports.append({"at": frame, "mode": "behind", "offset": [x if x else 40.0, y], "turn": True})
            elif abs(x) >= 60 or abs(y) >= 40:
                seen_tp.add(key)
                near_turn = any(abs(tf - frame) <= 4 for tf in turn_frames)
                teleports.append({"at": frame, "mode": "offset", "offset": [x, y], "turn": near_turn})
    tp_out = None if not teleports else (teleports[0] if len(teleports) == 1 else teleports)
    return {"explods": explods, "afterimage": afterimage, "shakes": shakes, "flashes": flashes,
            "super_pause": super_pause, "invuln": invuln, "teleport": tp_out}, extra_anims


def detect_cancels(cmd_ctrls, state_to_id):
    cancels = {}
    for c in cmd_ctrls:
        if c["type"] != "changestate":
            continue
        dst = int(nums(c["body"].get("value"), (-1,))[0])
        if dst not in state_to_id:
            continue
        dst_id = state_to_id[dst]
        groups = {}
        for k, v in c["pairs"]:
            if k.startswith("trigger") and k[7:].isdigit():
                groups.setdefault(int(k[7:]), []).append(v.lower())
        for lines in groups.values():
            joined = " & ".join(lines)
            if "movecontact" in joined or "movehit" in joined:
                for m in re.findall(r"stateno\s*=\s*(\d+)", joined):
                    src = int(m)
                    if src in state_to_id and state_to_id[src] != dst_id:
                        cancels.setdefault(state_to_id[src], [])
                        if dst_id not in cancels[state_to_id[src]]:
                            cancels[state_to_id[src]].append(dst_id)
    return cancels
