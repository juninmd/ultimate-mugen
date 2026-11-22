import os

from .sff import read_act


def _colors(raw):
    return [tuple(raw[i * 3:i * 3 + 3]) for i in range(256)]


def _hex(color):
    return "#%02x%02x%02x" % color


def exact_palettes(base_act, alt_paths):
    base = _colors(read_act(base_act))
    palettes = []
    for path in alt_paths:
        if not path or not os.path.exists(path):
            continue
        with open(path, "rb") as handle:
            alt = _colors(read_act(handle.read()))
        mapping = {}
        for index in range(1, 256):
            if base[index] != alt[index]:
                mapping.setdefault(_hex(base[index]), _hex(alt[index]))
        palettes.append({"exact": mapping})
    return palettes if any(p["exact"] for p in palettes) else []
