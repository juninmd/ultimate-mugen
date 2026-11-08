import os
from PIL import Image, ImageDraw, ImageFont

chars = ["daichi", "garan", "shion", "suzume", "sylene", "valdric", "volta"]
poses = ["walk0", "walk1", "walk2", "walk3", "walk4", "walk5", "run0", "run1", "run2", "run3"]

tile_size = 120
pad = 10
header_w = 120
w = header_w + len(poses) * (tile_size + pad) + pad
h = 60 + len(chars) * (tile_size + pad) + pad

img = Image.new("RGBA", (w, h), (24, 28, 38, 255))
draw = ImageDraw.Draw(img)

try:
    font = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 16)
    font_small = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 12)
except:
    font = ImageFont.load_default()
    font_small = ImageFont.load_default()

draw.text((pad, 15), "Native Fighter Motion Sprites (Walk0-5 & Run0-3)", fill=(255, 255, 255), font=font)

for col_idx, pose in enumerate(poses):
    x = header_w + col_idx * (tile_size + pad)
    draw.text((x + 10, 38), pose, fill=(180, 200, 220), font=font_small)

for row_idx, char_id in enumerate(chars):
    y = 60 + row_idx * (tile_size + pad)
    draw.text((pad, y + 45), char_id.capitalize(), fill=(220, 230, 245), font=font)
    for col_idx, pose in enumerate(poses):
        x = header_w + col_idx * (tile_size + pad)
        draw.rectangle([(x, y), (x + tile_size, y + tile_size)], fill=(34, 40, 54, 255), outline=(50, 60, 80))
        pose_path = f"assets/chars/sprites/{char_id}/{pose}.png"
        if os.path.exists(pose_path):
            sprite = Image.open(pose_path).convert("RGBA")
            sprite = sprite.resize((tile_size - 10, tile_size - 10), Image.NEAREST)
            img.paste(sprite, (x + 5, y + 5), sprite)

os.makedirs("docs/assets/pixel-art", exist_ok=True)
out_path = "docs/assets/pixel-art/motion_sprites_showcase.png"
img.save(out_path)
print(f"Saved motion showcase to {out_path}")
