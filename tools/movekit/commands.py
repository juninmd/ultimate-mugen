from .controller import controller

STATE_ORDER = {"super": 0, "air_special": 1, "special": 1, "projectile": 1, "throw": 2}


def command_name(move):
    return move.get("command_name", f"spec_{move['state']}")


def command_entries(moves, default_time=15):
    out = []
    seen = set()
    for move in moves:
        if not move.get("command") or command_name(move) in seen:
            continue
        seen.add(command_name(move))
        out.append(f'[Command]\nname = "{command_name(move)}"\ncommand = {move["command"]}\ntime = {move.get("command_time", default_time)}\n')
    return "\n".join(out)


def entry_block(moves):
    normals = sorted(int(m["state"]) for m in moves if m.get("kind", "normal") in ("normal", "crouch", "air"))
    contact = f"((stateno = [{normals[0]}, {normals[-1]}]) && (movecontact))" if normals else "0"
    commanded = [m for m in moves if m.get("command")]
    ordered = sorted(commanded, key=lambda m: (STATE_ORDER.get(m.get("kind", "normal"), 3), -len(m["command"])))
    out = "[Statedef -1]\n\n"
    for move in ordered:
        kind = move.get("kind", "normal")
        conditions = ["roundstate = 2", "statetype = A" if kind in ("air", "air_special") else "statetype != A"]
        conditions.append("ctrl" if kind in ("normal", "crouch", "air", "throw") else f"(ctrl) || {contact}")
        if move.get("power"):
            conditions.append(f"power >= {abs(int(move['power']))}")
        if move.get("condition"):
            conditions.append(move["condition"])
        out += controller("ChangeState", f"spec: {move.get('name', move['state'])}", value=move["state"],
                          triggerall=[f"({c})" for c in conditions], trigger1=f'command = "{command_name(move)}"')
    return out
