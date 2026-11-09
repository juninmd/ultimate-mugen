import colorsys
from pathlib import Path


def load_act(path):
    data = Path(path).read_bytes()[:768]
    colors = [tuple(data[i:i + 3]) for i in range(0, len(data) - 2, 3)]
    colors += [(0, 0, 0)] * (256 - len(colors))
    return colors[::-1]


def save_act(colors, path):
    padded = list(colors) + [(0, 0, 0)] * (256 - len(colors))
    Path(path).write_bytes(bytes(c for color in padded[:256][::-1] for c in color))


def variant_colors(colors, hue_shift=0.0, saturation=1.0, brightness=1.0, keep=()):
    keep_set = set(keep) | {0}
    result = []
    for index, (r, g, b) in enumerate(colors):
        if index in keep_set:
            result.append((r, g, b))
            continue
        hue, light, sat = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
        hue = (hue + hue_shift / 360) % 1
        nr, ng, nb = colorsys.hls_to_rgb(hue, min(1, light * brightness), min(1, sat * saturation))
        result.append((round(nr * 255), round(ng * 255), round(nb * 255)))
    return result


PRESETS = {
    "red": {"hue_shift": -120},
    "blue": {"hue_shift": 120},
    "green": {"hue_shift": 60},
    "violet": {"hue_shift": 180},
    "dark": {"saturation": 0.9, "brightness": 0.65},
    "pale": {"saturation": 0.6, "brightness": 1.25},
    "gray": {"saturation": 0.0},
}


def write_variants(act_path, out_dir, names=None, keep=()):
    base = load_act(act_path)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    written = []
    for name in names or PRESETS:
        target = out / f"{name}.act"
        save_act(variant_colors(base, keep=keep, **PRESETS[name]), target)
        written.append(str(target))
    return written
