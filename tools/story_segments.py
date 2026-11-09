import random
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

SEGMENTS = 5
SEG_W, SEG_H = 836, 470
SEAM_BAND = 70
GROUND_Y = 398
ROOT = Path(__file__).resolve().parent.parent
STAGES = ROOT / "assets" / "stages"
OUT = STAGES / "story"

ARENAS = {
    "brazil": {
        "core": (210, 120, 330, 190),
        "grade": (0.82, 0.72, 0.95),
        "props": ["lamp", "barrel", "crate", "pot"],
        "glow": (255, 190, 90),
    },
    "neon": {
        "core": (200, 160, 180, 170),
        "grade": (0.85, 0.8, 1.05),
        "props": ["sign", "crate", "barrel", "lamp"],
        "glow": (60, 230, 255),
    },
    "bamboo": {
        "core": (200, 150, 175, 190),
        "grade": (0.85, 0.95, 0.85),
        "props": ["bamboo", "stone_lantern", "rock", "bamboo"],
        "glow": (255, 210, 120),
    },
    "volcano": {
        "core": (210, 160, 185, 175),
        "grade": (1.1, 0.8, 0.7),
        "props": ["rock", "spire", "rock", "ember"],
        "glow": (255, 110, 40),
    },
    "dojo": {
        "core": (200, 150, 180, 190),
        "grade": (0.85, 0.78, 1.0),
        "props": ["stone_lantern", "barrel", "crate", "lamp"],
        "glow": (255, 160, 80),
    },
}


def load_base(name):
    img = Image.open(STAGES / f"{name}.png").convert("RGB")
    return img.resize((SEG_W, SEG_H), Image.NEAREST)


def mirror_chain(window, width):
    flipped = window.transpose(Image.FLIP_LEFT_RIGHT)
    out = Image.new("RGB", (width, window.height))
    x, flip = 0, False
    while x < width:
        out.paste(flipped if flip else window, (x, 0))
        x += window.width
        flip = not flip
    return out


def middle_segment(base, start, span):
    window = base.crop((start, 0, start + span, SEG_H))
    return mirror_chain(window, SEG_W)


def smooth(t):
    return t * t * (3 - 2 * t)


def blend_seams(strip, seams):
    arr = np.asarray(strip).astype(np.float32)
    out = arr.copy()
    width = arr.shape[1]
    for seam in seams:
        for dx in range(-SEAM_BAND, SEAM_BAND):
            x = seam + dx
            if not 0 <= x < width:
                continue
            left = arr[:, min(x, seam - 1 - (x - seam)) if x >= seam else x]
            right = arr[:, x if x >= seam else min(width - 1, seam + (seam - 1 - x))]
            t = smooth((dx + SEAM_BAND) / (2 * SEAM_BAND - 1))
            out[:, x] = left * (1 - t) + right * t
    return Image.fromarray(np.clip(out, 0, 255).astype(np.uint8))


def grade(strip, end):
    arr = np.asarray(strip).astype(np.float32)
    width = arr.shape[1]
    ramp = np.linspace(0, 1, width, dtype=np.float32)[None, :, None]
    mult = 1 + (np.array(end, dtype=np.float32)[None, None, :] - 1) * ramp
    return Image.fromarray(np.clip(arr * mult, 0, 255).astype(np.uint8))


def palette_from(base):
    arr = np.asarray(base.crop((0, 330, SEG_W, 395))).reshape(-1, 3).astype(np.float32)
    luma = arr @ np.array([0.299, 0.587, 0.114], dtype=np.float32)
    order = np.argsort(luma)
    pick = lambda q: tuple(int(v) for v in arr[order[int(q * (len(order) - 1))]])
    return {"dark": pick(0.04), "shade": pick(0.25), "mid": pick(0.5), "light": pick(0.8), "bright": pick(0.97)}


def darken(color, k):
    return tuple(int(c * k) for c in color)


class Sprite:
    def __init__(self, w, h):
        self.layer = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        self.draw = ImageDraw.Draw(self.layer)

    def rect(self, x0, y0, x1, y1, color):
        self.draw.rectangle((x0, y0, x1, y1), fill=color + (255,))

    def poly(self, points, color):
        self.draw.polygon(points, fill=color + (255,))

    def ellipse(self, box, color):
        self.draw.ellipse(box, fill=color + (255,))

    def outlined(self, outline):
        alpha = self.layer.split()[3]
        grown = alpha.filter(ImageFilter.MaxFilter(3))
        halo = Image.new("RGBA", self.layer.size, outline + (255,))
        halo.putalpha(grown)
        halo.alpha_composite(self.layer)
        return halo


def barrel(pal, rng):
    s = Sprite(26, 34)
    s.rect(3, 2, 22, 31, pal["mid"])
    s.rect(3, 2, 8, 31, pal["light"])
    s.rect(18, 2, 22, 31, pal["shade"])
    for y in (7, 25):
        s.rect(2, y, 23, y + 2, pal["dark"])
    s.rect(4, 2, 21, 3, pal["bright"])
    return s


def crate(pal, rng):
    s = Sprite(34, 32)
    s.rect(1, 1, 32, 30, pal["mid"])
    s.rect(1, 1, 32, 4, pal["light"])
    s.rect(1, 27, 32, 30, pal["shade"])
    s.rect(4, 4, 6, 27, pal["dark"])
    s.rect(27, 4, 29, 27, pal["dark"])
    for i in range(24):
        s.rect(5 + i, 5 + i if 5 + i < 27 else 26, 6 + i, 6 + i if 6 + i < 27 else 27, pal["shade"])
    return s


def rock(pal, rng):
    w = rng.randint(34, 52)
    h = rng.randint(22, 34)
    s = Sprite(w, h)
    s.poly([(0, h - 1), (4, h * 0.45), (w * 0.3, 2), (w * 0.62, h * 0.2), (w - 3, h * 0.55), (w - 1, h - 1)], pal["shade"])
    s.poly([(w * 0.3, 2), (w * 0.62, h * 0.2), (w * 0.5, h * 0.55), (w * 0.2, h * 0.5)], pal["light"])
    s.rect(2, h - 5, w - 2, h - 1, pal["dark"])
    return s


def spire(pal, rng):
    h = rng.randint(70, 96)
    s = Sprite(32, h)
    s.poly([(2, h - 1), (10, 14), (16, 0), (23, 18), (30, h - 1)], pal["shade"])
    s.poly([(16, 0), (23, 18), (30, h - 1), (18, h - 1)], pal["dark"])
    s.poly([(10, 14), (16, 0), (14, h - 1), (6, h - 1)], pal["light"])
    return s


def lamp(pal, rng, glow):
    s = Sprite(28, 92)
    s.rect(12, 18, 15, 88, pal["dark"])
    s.rect(9, 86, 18, 91, pal["shade"])
    s.rect(7, 4, 20, 18, pal["dark"])
    s.rect(9, 6, 18, 16, glow)
    s.rect(10, 7, 14, 10, (255, 255, 235))
    return s


def pot(pal, rng, glow):
    s = Sprite(30, 40)
    s.poly([(5, 16), (24, 16), (22, 39), (7, 39)], (190, 98, 56))
    s.rect(3, 12, 26, 17, (214, 126, 74))
    s.ellipse((2, 0, 27, 15), (70, 130, 60))
    for _ in range(6):
        x, y = rng.randint(5, 22), rng.randint(2, 11)
        s.rect(x, y, x + 2, y + 2, rng.choice([(236, 70, 90), (250, 210, 80), (240, 240, 250)]))
    return s


def bamboo(pal, rng, glow):
    s = Sprite(34, 140)
    for i in range(3):
        x = 3 + i * 11
        top = rng.randint(0, 24)
        s.rect(x, top, x + 5, 139, (92, 150, 70))
        s.rect(x, top, x + 1, 139, (140, 196, 100))
        for y in range(top + 12, 139, 20):
            s.rect(x - 1, y, x + 6, y + 2, (52, 96, 48))
        s.poly([(x + 5, top + 30), (x + 18, top + 22), (x + 6, top + 36)], (110, 176, 84))
    return s


def stone_lantern(pal, rng, glow):
    s = Sprite(30, 70)
    s.rect(9, 52, 20, 69, pal["shade"])
    s.rect(11, 30, 18, 52, pal["mid"])
    s.rect(6, 14, 23, 30, pal["mid"])
    s.rect(9, 17, 20, 27, glow)
    s.poly([(0, 14), (14, 0), (29, 14)], pal["dark"])
    s.rect(7, 66, 22, 69, pal["dark"])
    return s


def sign(pal, rng, glow):
    s = Sprite(40, 90)
    s.rect(18, 40, 21, 89, pal["dark"])
    s.rect(2, 2, 37, 40, pal["dark"])
    s.rect(5, 5, 34, 37, glow)
    s.rect(9, 10, 30, 14, (255, 255, 255))
    s.rect(9, 22, 24, 26, (255, 255, 255))
    return s


def ember(pal, rng, glow):
    s = Sprite(40, 26)
    s.poly([(0, 25), (6, 10), (20, 4), (34, 10), (39, 25)], pal["dark"])
    s.rect(8, 14, 30, 16, glow)
    s.rect(14, 19, 24, 21, (255, 210, 120))
    return s


def build_prop(kind, pal, rng, glow):
    table = {
        "barrel": lambda: barrel(pal, rng),
        "crate": lambda: crate(pal, rng),
        "rock": lambda: rock(pal, rng),
        "spire": lambda: spire(pal, rng),
        "lamp": lambda: lamp(pal, rng, glow),
        "pot": lambda: pot(pal, rng, glow),
        "bamboo": lambda: bamboo(pal, rng, glow),
        "stone_lantern": lambda: stone_lantern(pal, rng, glow),
        "sign": lambda: sign(pal, rng, glow),
        "ember": lambda: ember(pal, rng, glow),
    }
    return table[kind]()


def place_props(strip, name, pal, spec, rng):
    canvas = strip.convert("RGBA")
    for seg in range(SEGMENTS):
        count = 3 if seg in (0, SEGMENTS - 1) else 5
        slots = [SEAM_BAND + 40 + int(i * (SEG_W - 2 * (SEAM_BAND + 40)) / count) for i in range(count)]
        for slot in slots:
            kind = rng.choice(spec["props"])
            sprite = build_prop(kind, pal, rng, spec["glow"])
            if rng.random() < 0.5:
                sprite.layer = sprite.layer.transpose(Image.FLIP_LEFT_RIGHT)
            img = sprite.outlined(darken(pal["dark"], 0.45))
            img = img.resize((int(img.width * 1.7), int(img.height * 1.7)), Image.NEAREST)
            x = seg * SEG_W + slot + rng.randint(-24, 24)
            base_y = GROUND_Y - rng.randint(4, 22)
            shadow = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
            ImageDraw.Draw(shadow).ellipse(
                (x - 4, base_y - 3, x + img.width + 4, base_y + 6), fill=(0, 0, 0, 90)
            )
            canvas.alpha_composite(shadow)
            canvas.alpha_composite(img, (x, base_y - img.height))
    return canvas.convert("RGB")


def build(name):
    spec = ARENAS[name]
    base = load_base(name)
    flipped = base.transpose(Image.FLIP_LEFT_RIGHT)
    starts = [spec["core"][0], spec["core"][1], spec["core"][2]]
    spans = [spec["core"][3] - 0] * 3
    middles = [middle_segment(base, s, min(sp, SEG_W - s)) for s, sp in zip(starts, spans)]
    pieces = [base] + middles + [flipped]
    strip = Image.new("RGB", (SEG_W * SEGMENTS, SEG_H))
    for i, piece in enumerate(pieces):
        strip.paste(piece, (i * SEG_W, 0))
    strip = blend_seams(strip, [i * SEG_W for i in range(1, SEGMENTS)])
    strip = grade(strip, spec["grade"])
    rng = random.Random(f"story:{name}")
    strip = place_props(strip, name, palette_from(base), spec, rng)
    OUT.mkdir(parents=True, exist_ok=True)
    target = OUT / f"{name}.png"
    strip.save(target, optimize=True)
    print(target, strip.size)


if __name__ == "__main__":
    for arena in sys.argv[1:] or ARENAS:
        build(arena)
