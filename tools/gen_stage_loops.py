from pathlib import Path
import math
import random

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "stages"
OUTPUT = SOURCE / "animation"
SIZE = (1672, 941)
FRAME_COUNT = 8
STAGES = [
    "dojo", "neon", "bamboo", "plains", "spirit", "volcano", "brazil",
    "great_wall", "petra", "christ_redeemer", "texas", "chichen_itza",
    "colosseum", "taj_mahal", "pyramids_giza", "hanging_gardens",
    "temple_artemis", "statue_zeus", "mausoleum_halicarnassus",
    "colossus_rhodes", "lighthouse_alexandria",
]

EVENTS = {
    "petals": {"stages": {"dojo", "hanging_gardens"}, "colors": [(255, 174, 207), (255, 222, 236)]},
    "leaves": {"stages": {"bamboo", "brazil", "plains"}, "colors": [(151, 205, 89), (224, 180, 74)]},
    "embers": {"stages": {"volcano", "texas", "colosseum", "statue_zeus"}, "colors": [(255, 116, 42), (255, 211, 102)]},
    "rain": {"stages": {"neon", "colossus_rhodes", "lighthouse_alexandria"}, "colors": [(156, 218, 255), (224, 244, 255)]},
    "motes": {"stages": {"spirit", "christ_redeemer", "taj_mahal"}, "colors": [(95, 255, 228), (226, 228, 255)]},
    "sand": {"stages": {"petra", "pyramids_giza", "chichen_itza", "mausoleum_halicarnassus"}, "colors": [(246, 190, 118), (255, 230, 167)]},
    "clouds": {"stages": {"great_wall", "temple_artemis"}, "colors": [(221, 233, 255), (255, 245, 218)]},
}


def event_for(stage):
    return next((name for name, config in EVENTS.items() if stage in config["stages"]), "motes")


def draw_event(draw, event, x, y, phase, color, scale):
    if event in {"petals", "leaves"}:
        angle = phase * math.tau
        dx, dy = math.cos(angle) * scale, math.sin(angle) * scale * 0.55
        draw.ellipse((x - dx - scale, y - dy - scale * 0.45, x - dx + scale, y - dy + scale * 0.45), fill=color)
        draw.ellipse((x + dx - scale * 0.7, y + dy - scale * 0.35, x + dx + scale * 0.7, y + dy + scale * 0.35), fill=color)
    elif event == "rain":
        draw.line((x, y, x - scale * 1.1, y + scale * 4), fill=color, width=max(1, int(scale / 2)))
    elif event == "embers":
        draw.ellipse((x - scale, y - scale, x + scale, y + scale), fill=color)
        draw.ellipse((x - scale * 0.4, y - scale * 2.3, x + scale * 0.4, y + scale * 2.3), fill=color)
    else:
        draw.ellipse((x - scale, y - scale, x + scale, y + scale), fill=color)


def build_stage(stage):
    source = Image.open(SOURCE / f"{stage}.png").convert("RGB")
    if source.size != SIZE:
        source = source.resize(SIZE, Image.Resampling.LANCZOS)
    event = event_for(stage)
    colors = EVENTS[event]["colors"]
    rng = random.Random(stage)
    particles = []
    for index in range(120):
        particles.append((
            rng.uniform(0, SIZE[0]), rng.uniform(90, 775),
            rng.uniform(18, 55), rng.uniform(8, 22),
            rng.uniform(2.5, 8.0), colors[index % len(colors)],
            rng.uniform(0, math.tau),
        ))
    target = OUTPUT / stage
    target.mkdir(parents=True, exist_ok=True)
    for frame in range(FRAME_COUNT):
        layer = Image.new("RGBA", SIZE)
        draw = ImageDraw.Draw(layer)
        phase = frame / FRAME_COUNT
        for x0, y0, vx, vy, scale, rgb, offset in particles:
            theta = math.tau * phase + offset
            x = (x0 + vx * phase + math.sin(theta) * 12) % SIZE[0]
            y = 90 + ((y0 - 90 + vy * phase + math.sin(theta * 0.7) * 10) % 680)
            alpha = 100 + int(80 * (0.5 + 0.5 * math.sin(theta)))
            color = (*rgb, alpha)
            draw_event(draw, event, x, y, phase + offset, color, scale)
        layer.save(target / f"{frame:02}.png", optimize=True)


if __name__ == "__main__":
    for stage_id in STAGES:
        build_stage(stage_id)
    print(f"Generated {len(STAGES) * FRAME_COUNT} frames for {len(STAGES)} stages")
