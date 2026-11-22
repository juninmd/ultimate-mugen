import re

from .parse import nums


def convert_input(text):
    parts = [p.strip() for p in text.split(",")]
    if len(parts) == 2 and parts[0].startswith("/"):
        direction = parts[0].lstrip("/$~")
        return f"{direction}+{parts[1].lstrip('/$')}"
    return ", ".join(p.replace("/", "").replace("$", "") if not re.match(r"~\d+\$", p) else p for p in parts)


def _hold_dir(variants):
    m = re.fullmatch(r"/\$?([FBUD])", variants[0].strip()) if len(variants) == 1 else None
    return m.group(1) if m else None


def trigger_groups(ctrl):
    shared, groups = [], {}
    for key, value in ctrl["pairs"]:
        if key == "triggerall":
            shared.append(value.lower())
        elif key.startswith("trigger") and key[7:].isdigit():
            groups.setdefault(int(key[7:]), []).append(value.lower())
    return [" & ".join(shared + lines) for _, lines in sorted(groups.items())] or [" & ".join(shared)]


def _pick_group(groups, fly_states):
    for g in groups:
        if re.search(r"ctrl\s*=\s*0", g) or re.search(r"stateno\s*=\s*\d", g) and not re.search(r"\bctrl\b", g):
            continue
        if re.search(r"\bctrl\b", g) or not re.search(r"movecontact|movehit|time\s*>", g):
            return g, None
    for g in groups:
        if any(re.search(rf"stateno\s*=\s*{f}\b", g) for f in fly_states):
            return g, "fly"
    return None, None


def routes(cmd_states, commands, fly_states=()):
    commands = {k.lower(): v for k, v in commands.items()}
    out = []
    for ctrl in cmd_states:
        if ctrl["type"] != "changestate":
            continue
        value = int(nums(ctrl["body"].get("value"), (-1,))[0])
        text, mode = _pick_group(trigger_groups(ctrl), fly_states)
        if text is None:
            continue
        names = [n for n in re.findall(r'command\s*=\s*"([^"]+)"', text) if n in commands]
        holds = [_hold_dir(commands[n]) for n in names if _hold_dir(commands[n])]
        names = [n for n in names if not _hold_dir(commands[n])]
        if not names or all(re.fullmatch(r"/\$?\w+", c.strip()) for c in commands[names[0]]):
            continue
        state_type = re.search(r"statetype\s*=\s*([sca])", text)
        when = mode or {"a": "air", "c": "crouch"}.get(state_type.group(1) if state_type else "s", "stand")
        if "D" in holds and not state_type:
            when = "crouch"
        cmd_input = commands[names[0]][0]
        lead = next((h for h in holds if h in "FBU"), None)
        if lead and when != "crouch" and "," not in cmd_input:
            cmd_input = f"{lead}+{cmd_input}"
        cost = int(max([float(x) for x in re.findall(r"power\s*>=?\s*(\d+)", text)] or [0]))
        out.append((value, names[0], cmd_input, when, cost))
    return out
