import re
from collections import Counter

from .chain import chain, state_anim


def state_anim_missing(state, actions):
    anim = state_anim(state)
    return anim if anim is not None and anim >= 0 and anim not in actions else None

MAPPED = {"hitdef", "projectile", "velset", "veladd", "playsnd", "explod", "afterimage", "afterimagetime", "envshake", "envcolor",
          "superpause", "nothitby", "posadd", "turn", "changestate", "changeanim", "targetbind", "targetlifeadd", "targetstate",
          "changeanim2", "ctrlset", "poweradd", "null", "helper", "velmul", "posfreeze", "posset", "palfx"}
ENGINE_COMMON = set(range(0, 170)) | set(range(5000, 5300)) | {5500, 5900}


def _move_states(state_to_id, states, actions, consumed):
    owner = {}
    for sid, move_id in state_to_id.items():
        links, confirm = chain(sid, states, actions, skip=consumed)
        if confirm:
            links += chain(confirm[0], states, actions, skip=consumed)[0]
        for n in links:
            owner.setdefault(n, move_id)
    return owner


def _parents(states):
    out = {}
    for sid, st in states.items():
        for c in st["ctrl"]:
            for key in ("value", "p1stateno", "p2stateno", "stateno"):
                if key == "stateno" and c["type"] != "helper" or key == "value" and c["type"] not in ("changestate", "selfstate", "targetstate"):
                    continue
                m = re.match(r"\s*(\d+)\s*$", c["body"].get(key, ""))
                if m and int(m.group(1)) != sid:
                    out.setdefault(int(m.group(1)), (key if key != "value" else c["type"], sid))
    return out


def coverage(states, commands, actions, sff, bank, state_to_id, consumed, script_ids, fighter, sprite_files, shadowed=frozenset(), dropped=None):
    owner = _move_states(state_to_id, states, actions, consumed)
    parents = _parents(states)
    rows, lost, kinds = [], Counter(dropped or {}), {}
    for sid in sorted(s for s in states if s >= 0):
        if sid in owner:
            kind = f"move `{owner[sid]}`"
            for c in states[sid]["ctrl"]:
                if c["type"] not in MAPPED:
                    lost[c["type"]] += 1
        elif sid in consumed:
            kind = "ability"
        elif sid in script_ids:
            kind = "helper (verbatim CNS, spawned by a move)" if parents.get(sid, ("",))[0] == "stateno" else "script (verbatim CNS)"
        elif sid in ENGINE_COMMON:
            kind = "engine common state"
        elif sid in shadowed:
            kind = "shadowed: an earlier command with the same input wins (never fires in M.U.G.E.N either)"
        elif sid not in parents:
            kind = "unreachable in the original (dead code)"
        elif kinds.get(parents[sid][1], "").startswith("unreachable"):
            kind = kinds[parents[sid][1]]
        elif parents[sid][0] in ("p1stateno", "p2stateno", "targetstate") or kinds.get(parents[sid][1], "").startswith("partial: throw"):
            kind = "partial: throw states replaced by the UMF `throw` block"
        elif parents[sid][1] in consumed:
            kind = "ability (approximated by the native ability)"
        elif state_anim_missing(states[sid], actions):
            kind = f"broken in the original: animation {state_anim_missing(states[sid], actions)} is missing from the .air"
        else:
            via, src = parents[sid]
            label = owner.get(src) or ("script" if src in script_ids else "ability" if src in consumed else str(src))
            kind = f"NOT MIGRATED (entered from state {src} `{label}` via {via})"
        rows.append((sid, kind))
        kinds[sid] = kind
    missing_sprites = sorted({(f["g"], f["i"]) for a in actions.values() for f in a["frames"] if f["g"] >= 0 and (f["g"], f["i"]) not in sff.index})
    out = [f"# Coverage: {fighter['name']}", "",
           f"- States: {sum(1 for _, k in rows if not k.startswith('NOT MIGRATED'))}/{len(rows)} accounted for; the rest are NOT MIGRATED rows below",
           f"- Sprites: {sprite_files}/{len(sff.index)} exported" + (f"; {len(missing_sprites)} referenced by the .air but absent from the .sff (blank in M.U.G.E.N too)" if missing_sprites else ""),
           f"- Sounds: {len(bank)}/{len(bank)} exported",
           f"- Commands in .cmd: {len(commands)}; UMF moves: {len(fighter['moves'])}; abilities: {', '.join(sorted(fighter.get('abilities', {})))}",
           "", "| state | migrated as |", "|---|---|"]
    out += [f"| {sid} | {kind} |" for sid, kind in rows]
    if lost:
        out += ["", "Controllers inside migrated moves without a UMF field (dropped):", ""] + [f"- `{k}` x{v}" for k, v in lost.most_common()]
    return "\n".join(out) + "\n"
