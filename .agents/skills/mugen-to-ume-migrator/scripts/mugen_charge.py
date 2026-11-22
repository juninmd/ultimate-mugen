import re

from mugen_parse import nums


def _trigs(ctrl):
    return [v.lower() for k, v in ctrl["pairs"] if k.startswith("trigger")]


def _anim(state):
    value = state["def"].get("anim")
    return int(nums(value, (-1,))[0]) if value else None


def _snd(ctrl):
    s = nums(ctrl["body"].get("value", "").lstrip("FfSs"), (0, 0))
    return [int(s[0]), int(s[1])]


def _target(ctrl):
    return int(nums(ctrl["body"].get("value"), (-1,))[0])


def _elems(ctrl):
    return [int(m) for t in _trigs(ctrl) for m in re.findall(r"animelem\s*=\s*(\d+)", t)]


def _loop_ticks(state, actions):
    an = _anim(state)
    frames = actions.get(an, {}).get("frames", [])
    ticks = [f["t"] for f in frames]
    return sum(ticks) if ticks and all(t > 0 for t in ticks) else 0


def _window(ctrl):
    spans = []
    for t in _trigs(ctrl):
        m = re.fullmatch(r"time\s*=\s*\[\s*(\d+)\s*,\s*(\d+)\s*\]", t)
        if m:
            spans.append(int(m.group(2)) - int(m.group(1)) + 1)
        elif re.fullmatch(r"time\s*=\s*\d+", t):
            spans.append(1)
        elif t.strip() not in ("1", "time >= 0"):
            return None
    return min(spans) if spans else 0


def _target_of_end(state, states):
    for c in state["ctrl"]:
        if c["type"] == "changestate" and any("command !=" in t for t in _trigs(c)):
            return _target(c)
    return None


def _only_exits(state):
    return bool(state["ctrl"]) and all(c["type"] == "changestate" and any("animtime" in t for t in _trigs(c)) for c in state["ctrl"])


def charge_gain(state, actions):
    loop = _loop_ticks(state, actions)
    cycles = any(c["type"] == "changestate" and any("animtime" in t for t in _trigs(c)) for c in state["ctrl"])
    if not loop or not cycles:
        return None
    total = 0.0
    for c in state["ctrl"]:
        if c["type"] != "poweradd":
            continue
        span = _window(c)
        if span is None:
            return None
        total += nums(c["body"].get("value"), (0,))[0] * (loop if span == 0 else min(span, loop))
    per_tick = round(total / loop, 2)
    return {"start": per_tick, "growth": 0, "max": per_tick} if per_tick > 0 else None


def charge_spec(sid, states, actions, anim_name_fn, route_targets=()):
    st = states[sid]
    ch, anims, consumed = {"auraColor": [90, 70, 15]}, [], {sid}
    an = _anim(st)
    if an in actions:
        ch["anim"] = anim_name_fn(an)
        anims.append(an)
    if any(c["type"] == "envshake" for c in st["ctrl"]):
        ch["shake"] = True
    gain = charge_gain(st, actions)
    if gain:
        ch["gain"] = gain
    sounds = [c for c in st["ctrl"] if c["type"] == "playsnd"]
    if sounds:
        ch["sound"] = _snd(sounds[0])
        at = sorted({e for c in sounds if _snd(c) == ch["sound"] for e in _elems(c)})
        if at:
            ch["soundAt"] = at
    for c in st["ctrl"]:
        if c["type"] == "changestate" and any("command !=" in t for t in _trigs(c)):
            end = states.get(_target(c))
            if end and _anim(end) in actions and _target(c) != sid:
                ch["endAnim"] = anim_name_fn(_anim(end))
                anims.append(_anim(end))
                consumed.add(_target(c))
            break
    for c in st["ctrl"]:
        full = states.get(_target(c)) if c["type"] == "changestate" else None
        if full and any(re.search(r"power\s*>=", t) for t in _trigs(c)) and _anim(full) == _anim(states.get(_target_of_end(st, states)) or {"def": {}}) and _only_exits(full):
            consumed.add(_target(c))
    for no, other in states.items():
        if no == sid or (route_targets and no not in route_targets):
            continue
        enters = [c for c in other["ctrl"] if c["type"] == "changestate" and _target(c) == sid and any("animtime" in t for t in _trigs(c))]
        if enters and _anim(other) in actions:
            ch["startAnim"] = anim_name_fn(_anim(other))
            anims.append(_anim(other))
            consumed.add(no)
            snd = next((c for c in other["ctrl"] if c["type"] == "playsnd"), None)
            if snd:
                ch["startSound"] = _snd(snd)
            break
    return ch, anims, consumed
