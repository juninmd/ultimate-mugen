import re

from .parse import nums


AIRBORNE_TICKS = 240


def triggers(ctrl):
    return [v.lower() for k, v in ctrl["pairs"] if k.startswith("trigger")]


def state_anim(state):
    anim_no = int(nums(state["def"].get("anim"), (-1,))[0]) if state["def"].get("anim") else None
    for ctrl in state["ctrl"]:
        if anim_no is None and ctrl["type"] == "changeanim":
            anim_no = int(nums(ctrl["body"].get("value"), (0,))[0])
    return anim_no


def chain(state_no, states, actions, limit=12, skip=frozenset()):
    out, confirm = [state_no], None
    while len(out) < limit:
        nxt_uncond, nxt_cond = [], []
        for c in states[out[-1]]["ctrl"]:
            if c["type"] != "changestate":
                continue
            t = " ".join(triggers(c))
            if "command" in t:
                continue
            val = int(nums(c["body"].get("value"), (0,))[0])
            if val < 200 or val in out or val in skip or val not in states or state_anim(states[val]) not in actions:
                continue
            if "movehit" in t or "movecontact" in t:
                if "!= 1" not in t and "!movehit" not in t and "!movecontact" not in t:
                    nxt_cond.append((val, "contact" if "movecontact" in t else "hit"))
            else:
                nxt_uncond.append(val)
        if nxt_cond:
            confirm = nxt_cond[0]
            break
        if not nxt_uncond:
            break
        out.append(nxt_uncond[0])
    return out, confirm


def _exit_time(state, nxt):
    for c in state["ctrl"]:
        if c["type"] == "changestate" and int(nums(c["body"].get("value"), (-1,))[0]) == nxt:
            m = re.search(r"(?<!anim)time\s*>?=\s*(\d+)", " ".join(triggers(c)))
            return int(m.group(1)) if m else None
    return None


def lands(state, nxt):
    for c in state["ctrl"]:
        if c["type"] == "changestate" and int(nums(c["body"].get("value"), (-1,))[0]) == nxt:
            return bool(re.search(r"pos\s*y\s*>=?\s*-?\d", " ".join(triggers(c))))
    return False


def segment(state, action, nxt):
    frames = action["frames"]
    ticks = AIRBORNE_TICKS if nxt is not None and lands(state, nxt) else _exit_time(state, nxt) if nxt is not None else None
    if not ticks or not frames:
        return list(frames)
    out, total, k, loop = [], 0, 0, action.get("loop") or 0
    while total < ticks and len(out) < 400:
        fr = frames[k]
        t = fr["t"] if fr["t"] > 0 else ticks - total
        t = min(t, ticks - total)
        out.append(dict(fr, t=t))
        total += t
        k = k + 1 if k + 1 < len(frames) else loop
    return out
