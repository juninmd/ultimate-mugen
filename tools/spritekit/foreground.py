from collections import Counter

import numpy as np
from PIL import Image


def parse_color(value):
    if isinstance(value, str):
        text = value.lstrip("#")
        return tuple(int(text[i:i + 2], 16) for i in (0, 2, 4))
    return tuple(int(v) for v in value[:3])


def foreground_mask(image, background=None, tolerance=8, ignore_colors=None, alpha_threshold=16):
    rgba = np.asarray(image.convert("RGBA")).astype(np.int16)
    alpha = rgba[..., 3]
    if background is None and (alpha < 255).mean() > 0.02:
        return alpha > alpha_threshold
    if background is None:
        border = np.concatenate([rgba[0, :, :3], rgba[-1, :, :3], rgba[:, 0, :3], rgba[:, -1, :3]])
        background = Counter(map(tuple, border.tolist())).most_common(1)[0][0]
    else:
        background = parse_color(background)
    mask = (np.abs(rgba[..., :3] - np.array(background)).max(axis=2) > tolerance) & (alpha > alpha_threshold)
    for color in ignore_colors or []:
        mask &= np.abs(rgba[..., :3] - np.array(parse_color(color))).max(axis=2) > tolerance
    return mask


def open_rgba(path):
    return Image.open(path).convert("RGBA")
