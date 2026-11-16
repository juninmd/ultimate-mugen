from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[6]
sprite_dir = root / "assets/chars/sprites/garan"
out_dir = Path(__file__).resolve().parent
names = ["sg_run_mid", "sg_run_run0", "sg_run_run1", "sg_run_run2", "sg_run_run3"]
font = ImageFont.load_default()
sheet = Image.new("RGBA", (len(names) * 420, 445), (32, 32, 40, 255))
draw = ImageDraw.Draw(sheet)
for index, name in enumerate(names):
    sprite = Image.open(sprite_dir / f"{name}.png").convert("RGBA")
    draw.text((index * 420 + 10, 6), name, fill="white", font=font)
    sheet.alpha_composite(sprite, (index * 420 + 10, 30))
sheet.save(out_dir / "run-family-native.png")
sheet.resize((sheet.width * 2, sheet.height * 2), Image.Resampling.NEAREST).save(out_dir / "run-family-nearest-2x.png")
