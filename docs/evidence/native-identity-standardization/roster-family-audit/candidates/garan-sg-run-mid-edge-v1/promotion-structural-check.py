from pathlib import Path
import hashlib
import json

import numpy as np
from PIL import Image

root = Path(__file__).resolve().parents[6]
candidate_dir = Path(__file__).resolve().parent
active_path = root / "assets/chars/sprites/garan/sg_run_mid.png"
before_path = candidate_dir / "sg_run_mid-before.png"
candidate_path = candidate_dir / "sg_run_mid-edge-v1.png"
meta_path = root / "assets/chars/sprites/garan/meta.json"

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

before = np.array(Image.open(before_path))
active = np.array(Image.open(active_path))
candidate = np.array(Image.open(candidate_path))
meta = json.loads(meta_path.read_text(encoding="utf-8"))
frame_meta = meta["frames"]["sg_run_mid"]
visible = active[:, :, 3] >= 16
ys, xs = np.where(visible)
alpha_values = set(np.unique(active[:, :, 3]).tolist())
changed = np.any(before != active, axis=2)
changed_y, changed_x = np.where(changed)
rgb_equal = bool(np.array_equal(before[:, :, :3], active[:, :, :3]))
checks = {
    "canvas400x400": list(active.shape[:2]) == [400, 400],
    "rgbaPng": Image.open(active_path).mode == "RGBA",
    "hardAlphaOnly": alpha_values.issubset({0, 255}),
    "transparentTopEdge": int(np.count_nonzero(active[0, :, 3] >= 16)) == 0,
    "sameFloorEdgeAsSource": int(np.count_nonzero(active[-1, :, 3] >= 16)) == int(np.count_nonzero(before[-1, :, 3] >= 16)),
    "bodyPixelsUnmovedOutsideTopRetouch": int(changed_y.max()) <= 12,
    "rgbPalettePixelsUnchanged": rgb_equal,
    "candidateMatchesPromotedActive": bool(np.array_equal(candidate, active)),
    "backupMatchesOriginalSourceHash": sha(before_path) == "cf150843db3171c61a11c6d69d4d97abb581d92a5aacc73763c98018505b09b8",
    "promotedHashMatchesReviewedCandidate": sha(active_path) == "685cacd4ae27e67c74f644f940afe6dfc269247636d1ce9d449a94b2e6ca06ef",
    "sharedMetadataUnchangedValues": frame_meta == {"axisX": 200.0, "axisY": 363.0, "refHeight": 360.0},
}
result = {
    "scope": "Structural QA only for the promoted Garan run-frame edge correction. No engine tests or runtime checks.",
    "active": str(active_path.relative_to(root)),
    "backup": str(before_path.relative_to(root)),
    "candidate": str(candidate_path.relative_to(root)),
    "hashes": {"backupOriginal": sha(before_path), "candidate": sha(candidate_path), "promotedActive": sha(active_path)},
    "promotedMetrics": {
        "dimensions": list(Image.open(active_path).size),
        "mode": Image.open(active_path).mode,
        "alphaValues": sorted(alpha_values),
        "alphaBBox": [int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())],
        "topRowOpaquePixels": int(np.count_nonzero(active[0, :, 3] >= 16)),
        "bottomRowOpaquePixels": int(np.count_nonzero(active[-1, :, 3] >= 16)),
        "uniqueVisibleRGBColors": len({tuple(pixel[:3]) for pixel in active[visible]}),
        "changedPixelsFromBackup": int(changed.sum()),
        "changedYExtent": [int(changed_y.min()), int(changed_y.max())],
        "changedRGBPixelCount": int(np.count_nonzero(np.any(before[:, :, :3] != active[:, :, :3], axis=2))),
        "metaFrame": frame_meta,
    },
    "checks": checks,
    "result": "PASS" if all(checks.values()) else "FAIL",
}
(candidate_dir / "promotion-structural-qa.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2))
if not all(checks.values()):
    raise SystemExit(1)
