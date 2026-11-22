from .parse import nums

NUMBER_KEYS = {"id": "id", "supermovetime": "superMoveTime", "pausemovetime": "pauseMoveTime", "facing": "facing", "ownpal": "ownPal"}


def _spec(spawn, remap):
    body = spawn["body"]
    out = {"at": spawn["at"], "state": remap[spawn["state"]]}
    if body.get("name"):
        out["name"] = body["name"].strip().strip('"')
    if body.get("pos"):
        x, y = nums(body["pos"], (0, 0))[:2]
        out["pos"] = [x, y]
    if body.get("postype"):
        out["postype"] = body["postype"].strip().lower()
    for key, field in NUMBER_KEYS.items():
        if body.get(key) and nums(body[key]):
            out[field] = max(-2**31, min(2**31 - 1, int(nums(body[key])[0])))
    return out


def attach(moves, remap):
    dropped = []
    for move in moves:
        specs = []
        for spawn in move.pop("_spawns", []):
            if spawn["state"] in remap:
                specs.append(_spec(spawn, remap))
            else:
                dropped.append("helper")
        if specs:
            move["spawn"] = specs
    return dropped


def seeds(moves):
    return [sp["state"] for m in moves for sp in m.get("_spawns", [])]
