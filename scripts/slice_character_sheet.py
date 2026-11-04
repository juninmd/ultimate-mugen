import os, glob
from PIL import Image
import numpy as np
from collections import deque

TWO_ROW_CHARS = {"shion", "suzume", "bruno", "volta", "lian", "marlow", "garan", "kaname", "jin", "daichi", "valdric", "haruki", "oboro"}

def slice_character(char_id):
    sheet_jpg = f"assets/chars/sprites/{char_id}.jpg"
    if not os.path.exists(sheet_jpg):
        print(f"File not found: {sheet_jpg}")
        return

    out_char_dir = f"assets/chars/sprites/{char_id}"
    os.makedirs(out_char_dir, exist_ok=True)

    img = Image.open(sheet_jpg).convert("RGB")
    arr = np.array(img)
    h, w, _ = arr.shape
    # Correct canvas background is below header (e.g. y=250, x=15)
    bg = arr[250, 15].astype(int)

    # Clean transparent full sheet via outer flood fill
    visited = np.zeros((h, w), dtype=bool)
    diff_all = np.abs(arr.astype(int) - bg).sum(axis=-1)
    q = deque()
    for x in range(w):
        if diff_all[0, x] < 40: q.append((0, x)); visited[0, x] = True
        if diff_all[h-1, x] < 40: q.append((h-1, x)); visited[h-1, x] = True
    for y in range(h):
        if diff_all[y, 0] < 40: q.append((y, 0)); visited[y, 0] = True
        if diff_all[y, w-1] < 40: q.append((y, w-1)); visited[y, w-1] = True
    # Also seed from margin (250, 15)
    if not visited[250, 15]:
        q.append((250, 15))
        visited[250, 15] = True

    while q:
        cy, cx = q.popleft()
        for dy, dx in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            ny, nx = cy + dy, cx + dx
            if 0 <= ny < h and 0 <= nx < w and not visited[ny, nx]:
                if diff_all[ny, nx] < 40:
                    visited[ny, nx] = True
                    q.append((ny, nx))
    alpha_all = np.where(visited, 0, 255).astype(np.uint8)
    rgba_all = np.dstack([arr, alpha_all])
    full_transparent = Image.fromarray(rgba_all)
    full_transparent.save(f"assets/chars/sprites/{char_id}_sheet.png")

    is_two_row = char_id in TWO_ROW_CHARS
    pose_names = ["idle", "attack1", "attack2", "special", "projectile", "victory"]
    aliases = {
        "idle": ["idle", "idle0", "stand"],
        "attack1": ["jab", "jab0", "jab1", "attack1", "punch"],
        "attack2": ["kick", "hkick", "attack2"],
        "special": ["special", "rise", "uppercut"],
        "projectile": ["projectile", "blast", "gale", "fireball"],
        "victory": ["win", "victory", "win0", "win1", "taunt"],
    }

    if is_two_row:
        # 3 columns x 2 rows
        col_w = w / 3.0
        # Exclude text labels below each row (Row 1 ends at 405, Row 2 ends at h - 45)
        r1_bottom = int(h * 0.52) - 35
        r2_bottom = h - 45
        boxes = [
            (int(0 * col_w), 30, int(1 * col_w), r1_bottom),
            (int(1 * col_w), 30, int(2 * col_w), r1_bottom),
            (int(2 * col_w), 30, w, r1_bottom),
            (int(0 * col_w), int(h * 0.52), int(1 * col_w), r2_bottom),
            (int(1 * col_w), int(h * 0.52), int(2 * col_w), r2_bottom),
            (int(2 * col_w), int(h * 0.52), w, r2_bottom),
        ]
    else:
        # 1 row with 6 columns
        col_w = w / 6.0
        boxes = [
            (int(i * col_w), 140, int((i + 1) * col_w) if i < 5 else w, h - 10)
            for i in range(6)
        ]

    for idx, (x1, y1, x2, y2) in enumerate(boxes):
        pose_name = pose_names[idx]
        cropped = full_transparent.crop((x1, y1, x2, y2))
        bbox = cropped.getbbox()
        if bbox:
            cropped = cropped.crop(bbox)
        
        out_fn = f"{out_char_dir}/{idx+1:02d}_{pose_name}.png"
        cropped.save(out_fn)

        for alias in aliases.get(pose_name, [pose_name]):
            cropped.save(f"{out_char_dir}/{alias}.png")
        print(f"  [{char_id}] Saved {out_fn} ({cropped.size[0]}x{cropped.size[1]})")

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        for cid in sys.argv[1:]:
            slice_character(cid)
    else:
        for p in glob.glob("assets/chars/sprites/*.jpg"):
            cid = os.path.splitext(os.path.basename(p))[0]
            slice_character(cid)
