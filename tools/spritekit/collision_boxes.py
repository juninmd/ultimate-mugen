import numpy as np


def anatomical_boxes(mask, axis, zones=3, margin=2):
    ys, xs = np.nonzero(mask)
    if not xs.size:
        return []
    ax, ay = axis
    min_x, max_x, min_y, max_y = int(xs.min()), int(xs.max()), int(ys.min()), int(ys.max())
    height = max_y - min_y
    if height < 30 or zones <= 1:
        return [(min_x - ax + margin, min_y - ay + margin, max_x - ax - margin, max_y - ay)]
    if height < 50 or zones == 2:
        splits = [min_y, min_y + int(height * 0.48), max_y + 1]
    else:
        splits = [min_y, min_y + int(height * 0.22), min_y + int(height * 0.62), max_y + 1]
    boxes = []
    for y0, y1 in zip(splits[:-1], splits[1:]):
        band = mask[y0:y1, :]
        band_ys, band_xs = np.nonzero(band)
        if not band_xs.size:
            continue
        columns = np.nonzero(band.sum(axis=0) >= max(1, (y1 - y0) * 0.08))[0]
        if not columns.size:
            columns = band_xs
        x0 = int(columns.min()) - ax + margin
        x1 = int(columns.max()) - ax - margin
        if x1 - x0 < 8:
            mid = (x0 + x1) // 2
            x0, x1 = mid - 4, mid + 4
        top = y0 - ay + margin
        bottom = y1 - 1 - ay
        if y1 >= max_y:
            bottom = max(bottom, 0)
        boxes.append((x0, top, x1, bottom))
    return boxes


def attack_boxes(mask, axis, reach_margin=2):
    ys, xs = np.nonzero(mask)
    ax, ay = axis
    front = xs >= ax
    if not front.any():
        return []
    fx, fy = xs[front], ys[front]
    reach = int(fx.max())
    distance = reach - ax
    if distance < 12:
        return []
    keep = fx >= reach - max(12, int(distance * 0.35))
    hx, hy = fx[keep], fy[keep]
    return [(int(hx.min()) - ax - reach_margin, int(hy.min()) - ay - reach_margin,
             int(hx.max()) - ax + reach_margin, int(hy.max()) - ay + reach_margin)]


def interpolate_boxes(first, second, steps):
    if len(first) != len(second):
        raise ValueError("box lists must have the same length")
    frames = []
    for step in range(1, steps + 1):
        t = step / (steps + 1)
        frames.append([tuple(round(a + (b - a) * t) for a, b in zip(box_a, box_b)) for box_a, box_b in zip(first, second)])
    return frames
