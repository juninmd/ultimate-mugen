from pathlib import Path

from PIL import Image


stages = Path(__file__).resolve().parent.parent / "assets" / "stages"
thumbnails = stages / "thumbs"
thumbnails.mkdir(exist_ok=True)
for source in sorted(stages.glob("*.png")):
    with Image.open(source) as image:
        image.convert("RGB").resize((240, 135), Image.Resampling.LANCZOS).save(
            thumbnails / source.name, optimize=True
        )
