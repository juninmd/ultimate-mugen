#!/usr/bin/env python3
import os
import sys
import numpy as np
from PIL import Image

CANONICAL_ANCHORS = {
    "akio": "walk0.png",
    "bruno": "walk0.png",
    "daichi": "walk0.png",
    "garan": "04_special.png",
    "haruki": "walk0.png",
    "jin": "walk0.png",
    "kaito": "walk0.png",
    "kaname": "walk0.png",
    "lian": "walk0.png",
    "marlow": "walk0.png",
    "oboro": "walk0.png",
    "shion": "walk0.png",
    "suzume": "walk0.png",
    "sylene": "walk0.png",
    "valdric": "walk0.png",
    "volta": "walk0.png",
}

FORBIDDEN_REF_PREFIXES = ("inline_", "df", "sg_run_")
FORBIDDEN_REF_NAMES = {"clk.png", "ajab.png", "akick.png", "cu0.png", "cu1.png", "cu2.png", "hop.png"}


def validate_reference(path: str) -> tuple[bool, str]:
    if not os.path.isfile(path):
        return False, f"missing file: {path}"
    base = os.path.basename(path)
    if base.startswith(FORBIDDEN_REF_PREFIXES) or base in FORBIDDEN_REF_NAMES:
        return False, f"rejected unverified/deprecated pose filename as reference: {base}"
    im = Image.open(path).convert("RGBA")
    if im.size != (400, 400):
        return False, f"invalid dimensions {im.size}, expected (400, 400)"
    arr = np.array(im)
    alphas = set(np.unique(arr[..., 3]))
    if not alphas.issubset({0, 255}):
        return False, f"partial alpha pixels found: {sorted(alphas)}"
    alpha_mask = arr[..., 3] > 0
    if int(alpha_mask.sum()) < 200:
        return False, "sprite foreground is empty or too small"
    colors = len(set(map(tuple, arr[alpha_mask][..., :3])))
    if colors > 48:
        return False, f"too many colors ({colors} > 48)"
    char_id = os.path.basename(os.path.dirname(path))
    expected_anchor = CANONICAL_ANCHORS.get(char_id)
    if expected_anchor and base != expected_anchor:
        return False, f"non-canonical reference {base} for {char_id} (expected {expected_anchor})"
    if char_id.startswith("npc_"):
        return False, f"NPC packs ({char_id}) must derive from their base fighter, not serve as generation references"
    return True, f"ok (canonical anchor for {char_id}, colors={colors})"


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("usage: validate-sprite-ref.py <sprite.png>...")
        sys.exit(2)
    failed = False
    for p in sys.argv[1:]:
        ok, msg = validate_reference(p)
        print(f"{'PASS' if ok else 'FAIL'} {p}: {msg}")
        if not ok:
            failed = True
    sys.exit(1 if failed else 0)
