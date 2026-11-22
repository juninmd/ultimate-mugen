import re

from mugen_parse import nums

PASSTHROUGH = range(170, 200)
ANIM_KEYS = {"changeanim": ("value",), "changeanim2": ("value",), "explod": ("anim",), "gamemakeanim": ("value", "anim"),
             "projectile": ("projanim", "projhitanim", "projremanim", "projcancelanim")}
START_REMAP = "x+y"


def _anim_ref(value, actions, anim_name_fn, used):
    text = value.strip()
    if not re.fullmatch(r"-?\d+", text) or int(text) not in actions:
        return value
    used.append(int(text))
    return f"@anim({anim_name_fn(int(text))})"


def _block(header, pairs, kind, actions, anim_name_fn, used):
    lines = [header]
    for key, value in pairs:
        if (kind == "statedef" and key == "anim") or key in ANIM_KEYS.get(kind, ()):
            value = _anim_ref(value, actions, anim_name_fn, used)
        lines.append(f"{key} = {value}")
    return "\n".join(lines)


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


def passthrough(states, commands, actions, anim_name_fn, skip):
    seeds = [s for s in PASSTHROUGH if s in states]
    ids = _closure(seeds, states, skip)
    if not ids:
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
    for sid in ids:
        st = states[sid]
        blocks.append(_block(f"[Statedef {sid}]", list(st["def"].items()), "statedef", actions, anim_name_fn, used))
        for k, c in enumerate(st["ctrl"]):
            blocks.append(_block(f"[State {sid}, {k}]", c["pairs"], c["type"], actions, anim_name_fn, used))
    spec = {"cns": "\n\n".join(blocks) + "\n", "override": True}
    if cmds:
        spec["commands"] = [{"name": _cmd_name(n), "input": i, "display": "Taunt" if n == "start" else n.title()} for n, i in cmds.items()]
    return spec, used
