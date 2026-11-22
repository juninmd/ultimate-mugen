import re

from mugen_charge import charge_spec
from mugen_parse import nums


def _trigs(ctrl):
    return [v.lower() for k, v in ctrl["pairs"] if k.startswith("trigger")]


def _state_anim(state):
    anim = int(nums(state["def"].get("anim"), (-1,))[0]) if state["def"].get("anim") else None
    for c in state["ctrl"]:
        if anim is None and c["type"] == "changeanim":
            anim = int(nums(c["body"].get("value"), (-1,))[0])
    return anim if anim is not None and anim >= 0 else None


def is_fly_state(state):
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
            if "p2dist" in vals:
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
        if is_fly_state(st):
            consumed.add(sid)
    for value, label, cmd_input, when, cost in cmd_routes:
        st = states.get(value)
        if not st:
            continue
        if is_fly_state(st) or "fly" in label.lower():
            consumed.add(value)
            if "fly" not in abilities:
                fly_st = states[next((s for s in consumed if is_fly_state(states.get(s))), value)]
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
            ch, anims, used = charge_spec(sid, states, actions, anim_name_fn)
            abilities["charge"] = ch
            extra_anims += anims
            consumed |= used
            break
    return abilities, consumed, extra_anims


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
