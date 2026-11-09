import numpy as np
from PIL import Image

TRANSPARENT_SLOT = (255, 0, 255)


def shared_palette(crops, max_colors=256):
    opaque = [np.asarray(c)[np.asarray(c)[..., 3] > 0][:, :3] for c in crops]
    pixels = np.concatenate(opaque) if opaque else np.zeros((0, 3), np.uint8)
    unique = np.unique(pixels, axis=0) if pixels.size else pixels
    if len(unique) <= max_colors - 1:
        return [TRANSPARENT_SLOT] + [tuple(int(v) for v in c) for c in unique], True
    tile = Image.fromarray(pixels.reshape(1, -1, 3).astype(np.uint8), "RGB")
    quantized = tile.quantize(colors=max_colors - 1, method=Image.Quantize.MEDIANCUT)
    flat = quantized.getpalette()[: (max_colors - 1) * 3]
    return [TRANSPARENT_SLOT] + [tuple(flat[i:i + 3]) for i in range(0, len(flat), 3)], False


def nearest_indices(rgb, palette):
    colors = np.array([c[:3] for c in palette[1:]], dtype=np.int32)
    flat = rgb.reshape(-1, 3).astype(np.int32)
    unique, inverse = np.unique(flat, axis=0, return_inverse=True)
    best = np.empty(len(unique), dtype=np.int32)
    for start in range(0, len(unique), 4096):
        chunk = unique[start:start + 4096]
        distance = ((chunk[:, None, :] - colors[None, :, :]) ** 2).sum(axis=2)
        best[start:start + 4096] = distance.argmin(axis=1) + 1
    return best[inverse.reshape(-1)].reshape(rgb.shape[:2])


def to_indexed(crop, palette):
    data = np.asarray(crop.convert("RGBA"))
    indices = nearest_indices(data[..., :3], palette)
    indices[data[..., 3] == 0] = 0
    image = Image.fromarray(indices.astype(np.uint8), "P")
    flat = [c for color in palette for c in color]
    image.putpalette(flat + [0] * (768 - len(flat)))
    return image
