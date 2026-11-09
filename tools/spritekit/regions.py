from dataclasses import dataclass, field

import numpy as np


@dataclass
class Region:
    left: int
    top: int
    right: int
    bottom: int
    row: int = 0
    axis_x: int = 0
    axis_y: int = 0
    fixed_axis: bool = False
    extra: dict = field(default_factory=dict)

    @property
    def width(self):
        return self.right - self.left

    @property
    def height(self):
        return self.bottom - self.top


def dilate(mask, iterations):
    grown = mask
    for _ in range(iterations):
        padded = np.pad(grown, 1)
        grown = np.zeros_like(grown)
        for dy in range(3):
            for dx in range(3):
                grown |= padded[dy:dy + mask.shape[0], dx:dx + mask.shape[1]]
    return grown


def label_components(mask):
    height, width = mask.shape
    parent = [0]
    labels = np.zeros(mask.shape, dtype=np.int32)

    def find(node):
        while parent[node] != node:
            parent[node] = parent[parent[node]]
            node = parent[node]
        return node

    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[max(ra, rb)] = min(ra, rb)

    previous_runs = []
    for y in range(height):
        row = mask[y].astype(np.int8)
        edges = np.diff(np.concatenate([[0], row, [0]]))
        starts = np.nonzero(edges == 1)[0]
        ends = np.nonzero(edges == -1)[0]
        runs = []
        for start, end in zip(starts.tolist(), ends.tolist()):
            label = 0
            for p_start, p_end, p_label in previous_runs:
                if p_end >= start and p_start <= end:
                    if label == 0:
                        label = p_label
                    else:
                        union(label, p_label)
            if label == 0:
                label = len(parent)
                parent.append(label)
            labels[y, start:end] = label
            runs.append((start, end, label))
        previous_runs = runs
    lookup = np.array([find(i) for i in range(len(parent))], dtype=np.int32)
    return lookup[labels]


def order_rows(regions):
    rows = []
    for region in sorted(regions, key=lambda r: (r.top + r.bottom) / 2):
        for row in rows:
            top = min(r.top for r in row)
            bottom = max(r.bottom for r in row)
            overlap = min(bottom, region.bottom) - max(top, region.top)
            if overlap > 0.5 * min(bottom - top, region.height):
                row.append(region)
                break
        else:
            rows.append([region])
    rows.sort(key=lambda row: min(r.top for r in row))
    ordered = []
    for index, row in enumerate(rows):
        for region in sorted(row, key=lambda r: r.left):
            region.row = index
            ordered.append(region)
    return ordered


def detect_auto(mask, merge_distance=4, min_area=12):
    grown = dilate(mask, merge_distance) if merge_distance > 0 else mask
    labels = np.where(mask, label_components(grown), 0)
    regions = []
    for label in np.unique(labels):
        if label == 0:
            continue
        ys, xs = np.nonzero(labels == label)
        if xs.size < min_area:
            continue
        regions.append(Region(int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1))
    return order_rows(regions)


def detect_grid(mask, cell_width=None, cell_height=None, columns=None, rows=None,
                spacing=(0, 0), offset=(0, 0), axis_in_cell=(0.5, 1.0), min_area=1):
    height, width = mask.shape
    if cell_width is None:
        if not columns:
            raise ValueError("set cell_width or columns")
        cell_width = (width - offset[0] - spacing[0] * (columns - 1)) // columns
    if cell_height is None:
        if not rows:
            raise ValueError("set cell_height or rows")
        cell_height = (height - offset[1] - spacing[1] * (rows - 1)) // rows
    regions = []
    row_index = 0
    y = offset[1]
    while y + cell_height <= height:
        x = offset[0]
        while x + cell_width <= width:
            cell = mask[y:y + cell_height, x:x + cell_width]
            if cell.sum() >= min_area:
                ys, xs = np.nonzero(cell)
                region = Region(x + int(xs.min()), y + int(ys.min()), x + int(xs.max()) + 1, y + int(ys.max()) + 1, row_index)
                region.axis_x = round(x + cell_width * axis_in_cell[0]) - region.left
                region.axis_y = round(y + cell_height * axis_in_cell[1]) - region.top
                region.fixed_axis = True
                regions.append(region)
            x += cell_width + spacing[0]
        y += cell_height + spacing[1]
        row_index += 1
    return regions
