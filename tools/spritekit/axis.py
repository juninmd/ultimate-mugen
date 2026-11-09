import numpy as np

AXIS_MODES = ("feet", "row-feet", "bottom-center", "center")


def assign_axes(regions, mask, mode="feet", feet_band=6):
    if mode not in AXIS_MODES:
        raise ValueError(f"axis mode must be one of {AXIS_MODES}")
    baseline = {}
    for region in regions:
        baseline[region.row] = max(baseline.get(region.row, 0), region.bottom - 1)
    for region in regions:
        if region.fixed_axis:
            continue
        width, height = region.width, region.height
        if mode == "center":
            region.axis_x, region.axis_y = width // 2, height // 2
            continue
        if mode == "bottom-center":
            region.axis_x, region.axis_y = width // 2, height - 1
            continue
        sub = mask[region.top:region.bottom, region.left:region.right]
        columns = np.nonzero(sub[max(0, height - feet_band):, :].any(axis=0))[0]
        region.axis_x = int(round((columns.min() + columns.max()) / 2)) if columns.size else width // 2
        region.axis_y = baseline[region.row] - region.top if mode == "feet" or mode == "row-feet" else height - 1
