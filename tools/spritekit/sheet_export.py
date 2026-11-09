import json
from pathlib import Path

from PIL import Image

from .axis import assign_axes
from .foreground import foreground_mask, open_rgba
from .indexed import shared_palette, to_indexed
from .regions import detect_auto, detect_grid


def slice_sheet(sheet_path, out_dir, mode="auto", group=0, start_item=0, axis_mode="feet",
                background=None, tolerance=8, max_colors=256, grid=None, merge_distance=4, indexed=True):
    image = open_rgba(sheet_path)
    mask = foreground_mask(image, background, tolerance)
    if mode == "grid":
        regions = detect_grid(mask, **(grid or {}))
    else:
        regions = detect_auto(mask, merge_distance)
    assign_axes(regions, mask, axis_mode)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    crops = []
    for region in regions:
        crop = image.crop((region.left, region.top, region.right, region.bottom))
        alpha = Image.fromarray((mask[region.top:region.bottom, region.left:region.right] * 255).astype("uint8"))
        crop.putalpha(alpha)
        crops.append(crop)
    palette, exact = shared_palette(crops, max_colors)
    entries = []
    for offset, (region, crop) in enumerate(zip(regions, crops)):
        item = start_item + offset
        name = f"{group}_{item}.png"
        if indexed:
            to_indexed(crop, palette).save(out / name, transparency=0)
        else:
            crop.save(out / name)
        entries.append({"group": group, "item": item, "file": name, "box": [region.left, region.top, region.right, region.bottom],
                        "axis": [region.axis_x, region.axis_y], "row": region.row})
    manifest = {"source": str(sheet_path), "exact_palette": exact, "sprites": entries}
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    return manifest
