import os
import sys
import json
import numpy as np
from PIL import Image
from scripts.slice_character_sheet import background_mask, cell_boxes

def process_sheet(image_path, char_id, pose_names, target_canvas=400, baseline_y=391, ref_height=320.0):
    image = Image.open(image_path).convert("RGB")
    arr = np.array(image)
    mask = background_mask(arr)
    alpha = np.where(mask, 0, 255).astype(np.uint8)
    sheet = Image.fromarray(np.dstack([arr, alpha]))

    boxes = cell_boxes(~mask)
    print(f"[{char_id}] Found {len(boxes)} boxes for {len(pose_names)} poses")
    if len(boxes) != len(pose_names):
        print(f"Warning: box count {len(boxes)} != pose count {len(pose_names)}")
        if len(boxes) < len(pose_names):
            pose_names = pose_names[:len(boxes)]
        else:
            boxes = boxes[:len(pose_names)]

    char_dir = f"assets/chars/sprites/{char_id}"
    os.makedirs(char_dir, exist_ok=True)
    meta_path = f"{char_dir}/meta.json"
    meta = {}
    if os.path.exists(meta_path):
        with open(meta_path, "r", encoding="utf-8") as f:
            meta = json.load(f)
    meta.setdefault("frames", {})

    heights = [b[3] - b[1] for b in boxes]
    ref_src_height = max(heights) if heights else 1.0

    for i, (pose_name, box) in enumerate(zip(pose_names, boxes)):
        crop = sheet.crop(box)
        c_arr = np.array(crop)
        c_alpha = c_arr[:, :, 3]
        ys, xs = np.nonzero(c_alpha > 16)
        if len(xs) == 0:
            continue

        scale = ref_height / ref_src_height
        new_w = max(1, int(round(crop.width * scale)))
        new_h = max(1, int(round(crop.height * scale)))
        resized = crop.resize((new_w, new_h), Image.NEAREST)

        canvas = Image.new("RGBA", (target_canvas, target_canvas), (0, 0, 0, 0))
        paste_x = int(round(target_canvas / 2.0 - new_w / 2.0))
        paste_y = baseline_y - new_h
        canvas.paste(resized, (paste_x, paste_y))

        out_file = f"{char_dir}/{pose_name}.png"
        canvas.save(out_file)
        meta["frames"][pose_name] = {
            "axisX": float(target_canvas / 2.0),
            "axisY": float(baseline_y + 1),
            "refHeight": float(ref_height)
        }
        print(f"  Saved {out_file}")

    with open(meta_path, "w", encoding="utf-8") as f:
        json.dump(meta, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"[{char_id}] Updated meta.json successfully")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python scripts/process_spritesheet.py <image_path> <char_id> <pose1,pose2,...>")
        sys.exit(1)
    img_path = sys.argv[1]
    character = sys.argv[2]
    poses = sys.argv[3].split(",")
    process_sheet(img_path, character, poses)
