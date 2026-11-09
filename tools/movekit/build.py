import tomllib
from pathlib import Path

from .animation import build_action
from .commands import command_entries, entry_block
from .kinds import KINDS
from .phases import expand_phases
from .sprites import SpriteLibrary
from .statedef import build_state


def validate_moves(moves):
    seen = set()
    for move in moves:
        if "state" not in move:
            raise ValueError("every move needs a state number")
        if move.get("kind", "normal") not in KINDS:
            raise ValueError(f"state {move['state']}: kind must be one of {KINDS}")
        for state in expand_phases(move):
            number = int(state["state"])
            if number in seen:
                raise ValueError(f"state {number} is defined twice")
            seen.add(number)


def build_moves(moves, defaults, library):
    validate_moves(moves)
    actions, states = [], []
    for move in moves:
        for state in expand_phases(move):
            text = ""
            if "frames" in state:
                _, text = build_action(state, library, defaults.get("clsn1_height"))
                actions.append(text)
            states.append(build_state(state, defaults, text))
    return {"air": "\n".join(actions), "cns": "".join(states), "cmd": command_entries(moves),
            "entry": entry_block(moves)}


def build_from_spec(spec_path, out_dir):
    spec_path = Path(spec_path)
    spec = tomllib.loads(spec_path.read_text("utf-8"))
    defaults = spec.get("defaults", {})
    library = SpriteLibrary(spec_path.parent / p for p in defaults.get("sprite_manifests", []))
    result = build_moves(spec.get("move", []), defaults, library)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    for key, name in (("air", "moves.air"), ("cns", "moves.cns"), ("cmd", "moves.cmd"), ("entry", "statedef-1.cns")):
        (out / name).write_text(result[key], encoding="utf-8")
    return result
