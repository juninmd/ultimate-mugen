from pathlib import Path
import hashlib
import json

import numpy as np
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[6]
sprite_dir = root / "assets/chars/sprites/garan"
out_dir = Path(__file__).resolve().parent
source_path = sprite_dir / "sg_run_mid.png"
candidate_path = out_dir / "sg_run_mid-edge-v1.png"

candidate = np.array(Image.open(source_path).convert("RGBA"))
for y in range(5):
    candidate[y, :, 3] = 0

taper = {
    5: (201, 205),
    6: (197, 211),
    7: (192, 218),
    8: (188, 224),
    9: (184, 231),
    10: (180, 238),
    11: (177, 246),
    12: (175, 255),
}
for y, (left, right) in taper.items():
    candidate[y, :left, 3] = 0
    candidate[y, right + 1:, 3] = 0
Image.fromarray(candidate, "RGBA").save(candidate_path, optimize=True)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def stats(path):
    im = np.array(Image.open(path).convert("RGBA"))
    alpha = im[:, :, 3]
    ys, xs = np.where(alpha >= 16)
    return {
        "sha256": sha(path),
        "bbox": [int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())],
        "width": int(xs.max() - xs.min() + 1),
        "height": int(ys.max() - ys.min() + 1),
        "topRowOpaquePixels": int(np.count_nonzero(alpha[0] >= 16)),
        "bottomRowOpaquePixels": int(np.count_nonzero(alpha[-1] >= 16)),
        "partialAlphaPixels": int(np.count_nonzero((alpha > 0) & (alpha < 255))),
        "visibleColors": len({tuple(px[:3]) for px in im.reshape(-1, 4) if px[3] >= 16}),
    }

names = ["04_special", "sg_run_mid", "sg_run_mid-edge-v1", "sg_run_run0", "sg_run_run1", "sg_run_run2", "sg_run_run3"]
images = {}
for name in names:
    path = candidate_path if name == "sg_run_mid-edge-v1" else sprite_dir / f"{name}.png"
    images[name] = Image.open(path).convert("RGBA")
font = ImageFont.load_default()
for scale, filename in ((1, "run-neighbor-native.png"), (2, "run-neighbor-nearest-2x.png")):
    cell = 420 * scale
    sheet = Image.new("RGBA", (cell * len(names), (430 if scale == 1 else 870)), (32, 32, 40, 255))
    draw = ImageDraw.Draw(sheet)
    for index, name in enumerate(names):
        x = index * cell + 10 * scale
        draw.text((x, 5), name, font=font, fill=(245, 245, 245, 255))
        sprite = images[name]
        if scale == 2:
            sprite = sprite.resize((800, 800), Image.Resampling.NEAREST)
        sheet.alpha_composite(sprite, (x, 25 * scale))
    sheet.save(out_dir / filename)

pair2x = Image.new("RGBA", (1640, 850), (32, 32, 40, 255))
pair_draw = ImageDraw.Draw(pair2x)
for index, name in enumerate(("sg_run_mid", "sg_run_mid-edge-v1")):
    x = index * 820 + 10
    pair_draw.text((x, 5), name, font=font, fill=(245, 245, 245, 255))
    pair2x.alpha_composite(images[name].resize((800, 800), Image.Resampling.NEAREST), (x, 25))
pair2x.save(out_dir / "active-candidate-nearest-2x.png")

neighbors2x = Image.new("RGBA", (1640, 1640), (32, 32, 40, 255))
neighbor_draw = ImageDraw.Draw(neighbors2x)
for index, name in enumerate(("sg_run_run0", "sg_run_run1", "sg_run_run2", "sg_run_run3")):
    x = (index % 2) * 820 + 10
    y = (index // 2) * 820 + 5
    neighbor_draw.text((x, y), name, font=font, fill=(245, 245, 245, 255))
    neighbors2x.alpha_composite(images[name].resize((800, 800), Image.Resampling.NEAREST), (x, y + 20))
neighbors2x.save(out_dir / "run-neighbors-nearest-2x.png")

report = {
    "candidateOnly": True,
    "identityCanonical": "assets/chars/sprites/garan/04_special.png",
    "poseAndScaleGuide": "assets/chars/sprites/garan/sg_run_mid.png and sg_run_run0..3.png",
    "sourceActive": "assets/chars/sprites/garan/sg_run_mid.png",
    "sourceActiveSha256": sha(source_path),
    "candidate": candidate_path.name,
    "candidateSha256": sha(candidate_path),
    "canvas": [400, 400],
    "sharedMetadata": {"axisX": 200, "axisY": 363, "refHeight": 360},
    "operation": "Retouched only the clipped top mane contour within rows 0-12. No resize, body movement, palette replacement, metadata change, or active integration.",
    "activeMetrics": stats(source_path),
    "candidateMetrics": stats(candidate_path),
    "neighborMetrics": {name: stats(sprite_dir / f"{name}.png") for name in names if name.startswith("sg_run_run")},
    "visualReview": {
        "identity": "Garan's red oni identity remains unchanged.",
        "poseAndScale": "All pixels in rows 13-399 are preserved; the active 400x400 canvas and shared axis/refHeight remain unchanged.",
        "topEdge": "Candidate has five transparent rows above the mane and no opaque pixels on row 0.",
        "nativeComparison": "run-neighbor-native.png",
        "nearest2xComparison": "active-candidate-nearest-2x.png and run-neighbors-nearest-2x.png",
        "runtimeOrTests": "Not run; isolated visual candidate only."
    }
}
(out_dir / "qa.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report, indent=2))
