import re

from .parse import nums

PASSTHROUGH = range(170, 200)
ANIM_KEYS = {"changeanim": ("value",), "changeanim2": ("value",), "explod": ("anim",), "gamemakeanim": ("value", "anim"),
             "projectile": ("projanim", "projhitanim", "projremanim", "projcancelanim")}
START_REMAP = "x+y"
HELPER_BASE = 70000
GLOBAL_STATES = (-3, -2)
STATE_KEYS = {"changestate": ("value",), "selfstate": ("value",), "helper": ("stateno",), "hitoverride": ("stateno",),
              "hitdef": ("p1stateno", "p2stateno"), "reversaldef": ("p1stateno", "p2stateno"), "targetstate": ("value",)}


def _anim_ref(value, actions, anim_name_fn, used):
    text = value.strip()
    if not re.fullmatch(r"-?\d+", text) or int(text) not in actions:
        return value
    used.append(int(text))
    return f"@anim({anim_name_fn(int(text))})"


def _renumber(value, remap, refs=None):
    text, refs = value.strip(), refs or {}
    if not re.fullmatch(r"-?\d+", text):
        return value
    num = int(text)
    return str(remap[num]) if num in remap else f"@move({refs[num]})" if num in refs else value


def _state_ref(m, remap, refs):
    num = int(m.group(3))
    if re.match(r"(?i)(p2|enemy|target)", m.group(1) or ""):
        return m.group(0)
    new = remap.get(num) or (f"@move({refs[num]})" if num in refs else m.group(3))
    return f"{m.group(1) or ''}{m.group(2)}{new}"


def _renumber_triggers(value, remap, refs=None):
    pattern = r"(?i)(\b\w+(?:\s*\(\s*\d+\s*\))?\s*,\s*|p2)?(stateno\s*!?=\s*)(\d+)"
    return re.sub(pattern, lambda m: _state_ref(m, remap, refs or {}), value)


def _block(header, pairs, kind, actions, anim_name_fn, used, remap=None, refs=None):
    lines = [header]
    for key, value in pairs:
        if (kind == "statedef" and key == "anim") or key in ANIM_KEYS.get(kind, ()):
            value = _anim_ref(value, actions, anim_name_fn, used)
        if (remap or refs) and key in STATE_KEYS.get(kind, ()):
            value = _renumber(value, remap or {}, refs)
        if (remap or refs) and key.startswith("trigger"):
            value = _renumber_triggers(value, remap or {}, refs)
        lines.append(f"{key} = {value}")
    return "\n".join(lines)


def helper_closure(seeds, states):
    out, queue = [], [s for s in seeds if s in states]
    while queue:
        sid = queue.pop(0)
        if sid in out or sid not in states:
            continue
        out.append(sid)
        for c in states[sid]["ctrl"]:
            for key in STATE_KEYS.get(c["type"], ()):
                nxt = int(nums(c["body"].get(key), (-1,))[0])
                if nxt >= 200 and nxt in states and nxt not in out:
                    queue.append(nxt)
    return {sid: HELPER_BASE + k for k, sid in enumerate(out)}


def _closure(seeds, states, skip):
    out, queue = [], list(seeds)
    while queue:
        sid = queue.pop(0)
        if sid in out or sid in skip or sid not in states:
            continue
        out.append(sid)
        for c in states[sid]["ctrl"]:
            if c["type"] == "changestate":
                nxt = int(nums(c["body"].get("value"), (-1,))[0])
                if nxt in PASSTHROUGH or nxt >= 200 and nxt not in skip:
                    queue.append(nxt)
    return out


def _cmd_name(name):
    return "umf_" + re.sub(r"\W+", "_", name.lower())


def _command_input(raw):
    text = ", ".join(p.strip().replace("/", "").replace("$", "").replace("~", "") for p in raw.split(","))
    return START_REMAP if text.strip().lower() == "s" else text


def passthrough(states, commands, actions, anim_name_fn, skip, remap=None, refs=None):
    seeds = [s for s in PASSTHROUGH if s in states]
    remap, refs = remap or {}, refs or {}
    ids = _closure(seeds, states, set(skip) | set(remap))
    globals_ = [s for s in GLOBAL_STATES if states.get(s, {}).get("ctrl")]
    if not ids and not remap and not globals_:
        return None, []
    used, blocks, cmds, entries = [], [], {}, []
    lower = {k.lower(): v for k, v in commands.items()}
    for ctrl in states.get(-1, {"ctrl": []})["ctrl"]:
        if ctrl["type"] != "changestate" or int(nums(ctrl["body"].get("value"), (-1,))[0]) not in ids:
            continue
        pairs = []
        for key, value in ctrl["pairs"]:
            for name in re.findall(r'command\s*=\s*"([^"]+)"', value, re.I):
                if name.lower() in lower and name.lower() not in cmds:
                    cmds[name.lower()] = _command_input(lower[name.lower()][0])
                value = value.replace(f'"{name}"', f'"{_cmd_name(name)}"')
            pairs.append((key, value))
        entries.append(_block(f"[State -1, umf {len(entries)}]", pairs, "changestate", actions, anim_name_fn, used))
    if entries:
        blocks.append("[Statedef -1]\n\n" + "\n\n".join(entries))
    for sid in globals_ + ids + list(remap):
        st, num = states[sid], remap.get(sid, sid)
        blocks.append(_block(f"[Statedef {num}]", list(st["def"].items()), "statedef", actions, anim_name_fn, used))
        scoped = remap if sid in remap or sid < 0 else None
        for k, c in enumerate(st["ctrl"]):
            blocks.append(_block(f"[State {num}, {k}]", c["pairs"], c["type"], actions, anim_name_fn, used, scoped, refs))
    spec = {"cns": "\n\n".join(blocks) + "\n", "override": True}
    if cmds:
        spec["commands"] = [{"name": _cmd_name(n), "input": i, "display": "Taunt" if n == "start" else n.title()} for n, i in cmds.items()]
    return spec, used
