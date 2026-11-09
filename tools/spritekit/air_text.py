import json
from pathlib import Path

import numpy as np
from PIL import Image

from .collision_boxes import anatomical_boxes, attack_boxes


def sprite_mask(path):
    image = Image.open(path)
    if image.mode == "P" and "transparency" not in image.info:
        return np.asarray(image) != 0
    return np.asarray(image.convert("RGBA"))[..., 3] > 0


def format_boxes(kind, boxes):
    if not boxes:
        return []
    lines = [f"Clsn{kind}: {len(boxes)}"]
    lines += [f"  Clsn{kind}[{i}] = {x0}, {y0}, {x1}, {y1}" for i, (x0, y0, x1, y1) in enumerate(boxes)]
    return lines


def action_text(manifest_path, action, ticks=6, attack=False, zones=3):
    manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
    base = Path(manifest_path).parent
    lines = [f"[Begin Action {action}]"]
    for entry in manifest["sprites"]:
        mask = sprite_mask(base / entry["file"])
        axis = tuple(entry["axis"])
        lines += format_boxes(2, anatomical_boxes(mask, axis, zones))
        if attack:
            lines += format_boxes(1, attack_boxes(mask, axis))
        lines.append(f"{entry['group']}, {entry['item']}, 0, 0, {ticks}")
    return "\n".join(lines) + "\n"
