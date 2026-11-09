def resolve(names, target, base):
    if isinstance(target, int) or (isinstance(target, str) and target.lstrip("-").isdigit()):
        return int(target)
    if target not in names:
        raise ValueError(f"move {base}: phase {target!r} does not exist (phases: {', '.join(names)})")
    return names[target]


def expand_phases(move):
    phases = move.get("phases") or []
    if not phases:
        return [move]
    base = int(move["state"])
    names = {"start": base}
    built = []
    for position, phase in enumerate(phases, 1):
        if phase.get("like"):
            source = next((p for p in built if p.get("name") == phase["like"]), None)
            if source is None:
                raise ValueError(f"phase {phase.get('name', position)} of move {base}: like = {phase['like']!r} must name an earlier phase")
            phase = {**{k: v for k, v in source.items() if k not in ("name", "state", "_phase_of")},
                     **{k: v for k, v in phase.items() if k != "like"}}
        phase = dict(phase, kind=move.get("kind", "normal"), _phase_of=base)
        phase.setdefault("state", base + position)
        if phase.get("name"):
            names[phase["name"]] = int(phase["state"])
        built.append(phase)
    root = {k: v for k, v in move.items() if k != "phases"}
    chain = [root] + [p for p in built if not p.get("helper")]
    for index, state in enumerate(chain):
        if "next" in state:
            state["_next"] = resolve(names, state["next"], base)
        elif index + 1 < len(chain):
            state["_next"] = int(chain[index + 1]["state"])
    return [root] + built
