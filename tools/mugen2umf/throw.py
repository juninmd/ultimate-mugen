import re

from .parse import nums


def _trigs(ctrl):
    return [v.lower() for k, v in ctrl["pairs"] if k.startswith("trigger")]


def _elem(ctrl):
    for t in _trigs(ctrl):
        m = re.search(r"animelem\s*=\s*(\d+)", t)
        if m:
            return int(m.group(1))
    return None


def _first(body, key, default):
    return int(nums(body.get(key), (default,))[0])


def _velset(state):
    values = nums(state["def"].get("velset"), ())
    if len(values) >= 2:
        return [values[0], values[1]]
    for c in state["ctrl"]:
        if c["type"] == "velset" and ("x" in c["body"] or "y" in c["body"]):
            return [nums(c["body"].get("x"), (0,))[0], nums(c["body"].get("y"), (0,))[0]]
    return None


def _life_loss(state, only_start):
    total = 0
    for c in state["ctrl"]:
        if c["type"] not in ("targetlifeadd", "lifeadd") or (c["type"] == "lifeadd" and not only_start):
            continue
        if c["type"] == "lifeadd" and not any(re.fullmatch(r"time\s*=\s*0", t) for t in _trigs(c)):
            continue
        total += abs(round(nums(c["body"].get("value"), (0,))[0]))
    return total


def _steps(p1_state, states):
    steps = {}
    binds = sorted((e, nums(c["body"].get("pos"), (30, 0))[:2]) for c in p1_state["ctrl"] if c["type"] == "targetbind" for e in [_elem(c)] if e)
    previous = binds[0][1] if binds else None
    for at, pos in binds[1:]:
        if pos != previous:
            steps.setdefault(at, {})["bind"] = [round(pos[0], 1), round(pos[1], 1)]
            previous = pos
    release = None
    for c in p1_state["ctrl"]:
        at = _elem(c)
        if at and c["type"] == "targetstate" and _first(c["body"], "value", -1) in states:
            release = (at, states[_first(c["body"], "value", -1)])
        elif at and c["type"] == "playsnd":
            s = nums(c["body"].get("value", "").lstrip("FfSs"), (0, 0))
            steps.setdefault(at, {})["sound"] = [int(s[0]), int(s[1])]
    return steps, release


def throw_steps(p1_state, states):
    steps, release = _steps(p1_state, states)
    if not release:
        return None
    at, launch_state = release
    total = _life_loss(p1_state, False) + _life_loss(launch_state, True)
    step = steps.setdefault(at, {})
    if total:
        step["damage"] = total
    launch = _velset(launch_state)
    if launch:
        step["launch"] = launch
    return [{"at": k, **steps[k]} for k in sorted(steps)], total


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
        throw["_p1"] = p1
        timed = throw_steps(p1_st, states)
        if timed:
            throw["steps"], total = timed
            throw["damage"] = total or dmg
            launch = next((x.get("launch") for x in throw["steps"] if x.get("launch")), None)
            throw["launch"] = launch or throw["launch"]
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




def splice_grab(move, throw, frames, hit_at, states, actions, anim_name_fn, anim_of, state_no):
    p1 = throw.pop("_p1")
    if not throw.get("steps") or anim_of(states[p1]) not in actions:
        return None
    base = len(frames)
    grab = actions[anim_of(states[p1])]["frames"]
    number = 900000 + state_no
    actions[number] = {"frames": list(frames) + list(grab), "loop": None}
    move["anim"] = anim_name_fn(number)
    for step in throw["steps"]:
        step["at"] += base
    if hit_at != 1:
        throw["at"] = hit_at
    return number
