import re

from .parse import nums


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


def charge_spec(sid, states, actions, anim_name_fn, route_targets=()):
    st = states[sid]
    ch, anims, consumed = {"auraColor": [90, 70, 15]}, [], {sid}
    an = _anim(st)
    if an in actions:
        ch["anim"] = anim_name_fn(an)
        anims.append(an)
    if any(c["type"] == "envshake" for c in st["ctrl"]):
        ch["shake"] = True
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
