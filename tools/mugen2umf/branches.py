import re

from .moves import build_move
from .routes import convert_input

HOLD = re.compile(r"/\$?[a-z]+")


def _window(text, t0, t1):
    m = re.search(r"time\s*(>=?)\s*(\d+)", text)
    lo = t0 + (int(m.group(2)) + (1 if m.group(1) == ">" else 0) if m else 1)
    return [lo, max(lo, t1)]


def classify(branch, commands):
    text = branch["text"]
    for op, name in re.findall(r'command\s*(!?=)\s*"([^"]+)"', text):
        variants = commands.get(name.lower())
        if not variants:
            continue
        held = all(HOLD.fullmatch(v.strip().lower()) for v in variants)
        if held and op == "!=":
            return {"kind": "release", "name": name.lower(), "input": variants[0].replace("$", "")}
        if held or branch.get("buffered"):
            return {"kind": "command", "name": name.lower(), "input": variants[0].replace("$", "") if held else convert_input(variants[0]),
                    "at": (branch["at"] or [None])[0], "buffer": 1 if held else 30}
        if op == "=":
            return {"kind": "rekka", "name": name.lower(), "input": convert_input(variants[0]), "window": _window(text, branch["t0"], branch["t1"]),
                    "contact": bool(re.search(r"movecontact|movehit", text))}
    return None


def _child(parent, target, child_id, cmd_input, states, actions, consumed, kind):
    child, anims = build_move(target, states, actions, child_id, cmd_input, parent.get("when", "stand"), 0, skip=consumed)
    if child:
        child["kind"] = parent["kind"] if kind != "rekka" else child["kind"]
        child.pop("cost", None)
    return child, anims


def expand(moves, states, actions, commands, consumed, state_to_id):
    wanted, script_commands, k = [], {}, 0
    while k < len(moves):
        move = moves[k]
        k += 1
        fu = move.pop("_followUpState", None)
        if fu and not move.get("followUp"):
            child, anims = _child(move, fu[0], f"{move['id']}_followup", "", states, actions, consumed, "followup")
            if child:
                move["followUp"] = {"to": child["id"], "on": fu[1]}
                moves.append(child)
                state_to_id.setdefault(fu[0], child["id"])
                wanted += anims
        for branch in move.pop("_branches", []):
            spec = classify(branch, commands)
            target = branch["target"]
            if not spec or target in consumed or any(m["id"] == f"{move['id']}_{spec['name']}" for m in moves):
                continue
            if spec["kind"] == "rekka":
                child, anims = _child(move, target, f"{move['id']}_{spec['name']}", spec["input"], states, actions, consumed, "rekka")
                if child:
                    child["rekka"] = {"after": [move["id"]], "window": spec["window"], **({"contact": True} if spec["contact"] else {})}
            elif not move.get("followUp"):
                child, anims = _child(move, target, f"{move['id']}_{spec['name']}", "", states, actions, consumed, spec["kind"])
                if child:
                    cmd = f"umf_{spec['name']}" + ("_buf" if spec.get("buffer", 1) > 1 else "")
                    script_commands[cmd] = {"name": cmd, "input": spec["input"], "time": 1, "buffer": spec.get("buffer", 1)}
                    move["followUp"] = {"to": child["id"], "on": spec["kind"], "command": cmd, **({"at": spec["at"]} if spec.get("at") else {})}
            else:
                child = None
            if child:
                moves.append(child)
                state_to_id.setdefault(target, child["id"])
                wanted += anims
    return wanted, list(script_commands.values())
