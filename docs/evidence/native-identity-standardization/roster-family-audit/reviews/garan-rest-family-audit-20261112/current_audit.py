from collections import Counter
from pathlib import Path
import hashlib
import json

from PIL import Image

root = Path(__file__).resolve().parents[6]
sprites = root / "assets/chars/sprites/garan"
old_report_path = root / "docs/evidence/native-identity-standardization/roster-family-audit/full-family-static-review-current.json"
old_report = json.loads(old_report_path.read_text(encoding="utf-8"))
review_dir = Path(__file__).resolve().parent

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def metrics(name):
    path = sprites / f"{name}.png"
    im = Image.open(path).convert("RGBA")
    alpha = im.getchannel("A")
    bbox = alpha.getbbox()
    return {
        "file": f"assets/chars/sprites/garan/{name}.png",
        "sha256": sha(path),
        "size": list(im.size),
        "bboxAlpha": list(bbox) if bbox else None,
        "bboxWidth": bbox[2] - bbox[0] if bbox else 0,
        "bboxHeight": bbox[3] - bbox[1] if bbox else 0,
        "topRowOpaquePixels": sum(alpha.getpixel((x, 0)) >= 16 for x in range(im.width)),
        "bottomRowOpaquePixels": sum(alpha.getpixel((x, im.height - 1)) >= 16 for x in range(im.width)),
    }

stale = {}
for family in old_report["families"]:
    if family["character"] != "garan":
        continue
    for frame in family.get("frames", []):
        if not frame.get("file"):
            continue
        rel = Path(frame["file"].replace("\\", "/"))
        path = root / rel
        if path.is_file():
            actual = sha(path)
            if actual != frame["sha256"]:
                stale[rel.as_posix()] = {"reportSha256": frame["sha256"], "currentSha256": actual}

garan_rows = [row for row in old_report["families"] if row["character"] == "garan"]
correction_families = sorted({row["animation"] for row in garan_rows if row.get("staticDisposition") == "correction_required"})
unresolved_families = sorted({row["animation"] for row in garan_rows if row.get("staticDisposition") == "unresolved"})
selected = [
    "guard", "guardHit", "crouchGuard", "crouchGuardHit", "airGuard", "airGuardHit",
    "09_hit1", "10_hit2", "11_hit3", "hitL1", "hitL2", "hitL3", "hitCrouch",
    "12_fall1", "13_fall2", "14_lie", "cjab0", "cjab1", "turn",
    "jumpTop", "jumpUp", "jumpDown", "hop", "ajab", "akick", "airLowKick", "land",
    "sg_run_mid", "sg_run_run0", "sg_run_run1", "sg_run_run2", "sg_run_run3",
]
result = {
    "auditDate": "2026-11-12",
    "scope": "Garan current active native sprite files and requested hit, guard, cjab, turn, plus aerial/run follow-up.",
    "baselineReport": old_report["reviewEpoch"],
    "baselineClaimCeiling": old_report["claimCeiling"],
    "baselineGaranRows": len(garan_rows),
    "baselineCorrectionFamilies": correction_families,
    "baselineUnresolvedFamilies": unresolved_families,
    "reportHashDrift": stale,
    "visualFindings": {
        "identityOrPoseMismatchesConfirmed": [],
        "cjabFalsePositive": "cjab0 and cjab1 are Garan and the crouched punches are legible; keep them distinct from any wrong-identity cjab pilot.",
        "hitGuardTurn": "Reviewed native and nearest-2x; identity and pose family pass.",
        "aerial": "After the current jumpTop/jumpUp/jumpDown/hop and airLowKick integrations, inspected Garan body poses read as canonical.",
        "confirmedRemainingVisualDefects": [],
        "integratedCorrectionsInThisAudit": [
            {"file": "assets/chars/sprites/garan/sg_run_mid.png", "defect": "Top mane touched row 0.", "activeSha256": "685cacd4ae27e67c74f644f940afe6dfc269247636d1ce9d449a94b2e6ca06ef", "backupSha256": "cf150843db3171c61a11c6d69d4d97abb581d92a5aacc73763c98018505b09b8"},
            {"file": "assets/chars/sprites/garan/airLowKick.png", "defect": "Top mane touched row 0.", "activeSha256": "ee7c8da21dc0deef320da012bd3336357ca566596cc4cf12a4d6663a6c8cd766", "integratedBy": "parent agent"}
        ],
        "followUpObservations": [
            {"files": "assets/chars/sprites/garan/sg_run_mid.png and sg_run_run0..3.png", "finding": "Their measured alpha bbox heights differ substantially despite shared run metadata; do not infer an error solely from this static comparison. Review in animation/runtime before changing scale."}
        ],
        "uncertain": [
            {"file": "assets/chars/sprites/garan/inline_c75becbcbc592dc9.png", "finding": "Top edge includes attack effect; body/hair crop is not confirmed separately, so not counted as a confirmed defect."}
        ],
        "effectOnlyUnresolved": unresolved_families
    },
    "frameMetrics": {name: metrics(name) for name in selected if (sprites / f"{name}.png").is_file()},
    "evidence": {
        "requestedFamiliesNative": "guard-hit-native.png",
        "requestedFamiliesNearest2x": "guard-hit-nearest-2x.png",
        "crouchCjabTurnNative": "crouch-cjab-turn-native.png",
        "crouchCjabTurnNearest2x": "crouch-cjab-turn-nearest-2x.png",
        "aerialNative": "aerial-family-native.png",
        "aerialNearest2x": "aerial-family-nearest-2x.png",
        "runNative": "run-family-native.png",
        "runNearest2x": "run-family-nearest-2x.png",
        "airLowKickIntegratedNative": "../candidates/garan-airlowkick-borderfix-20261112/native-comparison.png",
        "airLowKickIntegratedNearest2x": "../candidates/garan-airlowkick-borderfix-20261112/nearest-2x-comparison.png",
        "runCorrectionNative": "../candidates/garan-sg-run-mid-edge-v1/run-neighbor-native.png",
        "runCorrectionNearest2x": "../candidates/garan-sg-run-mid-edge-v1/active-candidate-nearest-2x.png",
        "runNeighborDetailNearest2x": "../candidates/garan-sg-run-mid-edge-v1/run-neighbors-nearest-2x.png"
    },
    "testsRuntimeAndIntegration": "Only the promoted sg_run_mid PNG structural checks were performed; no engine tests or runtime review."
}
(review_dir / "current-family-audit.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"reportHashDrift": stale, "baselineCorrectionFamilies": correction_families, "unresolvedFamilies": unresolved_families}, indent=2))
