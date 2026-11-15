import math
import random
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "props" / "story"
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
INK = (26, 18, 30, 255)


def ramp_color(ramp, t, x, y):
    t = min(max(t, 0.0), 0.999) * (len(ramp) - 1)
    low = int(t)
    step = (BAYER[y % 4][x % 4] + 0.5) / 16.0
    return ramp[min(low + (1 if t - low > step else 0), len(ramp) - 1)]


class Canvas:
    def __init__(self, width, height):
        self.size = (width, height)
        self.image = Image.new("RGBA", self.size, (0, 0, 0, 0))

    def layer(self):
        mask = Image.new("L", self.size, 0)
        return mask, ImageDraw.Draw(mask)

    def fill(self, mask, ramp, light=None):
        box = mask.getbbox()
        if not box:
            return
        px = self.image.load()
        mp = mask.load()
        x0, y0, x1, y1 = box
        for y in range(y0, y1):
            for x in range(x0, x1):
                if mp[x, y]:
                    u = (x - x0) / max(1, x1 - x0 - 1)
                    v = (y - y0) / max(1, y1 - y0 - 1)
                    t = light(u, v) if light else v
                    px[x, y] = ramp_color(ramp, t, x, y) + (255,)

    def shape(self, ramp, draw_shape, light=None):
        mask, draw = self.layer()
        draw_shape(draw)
        self.fill(mask, ramp, light)
        return mask

    def draw(self):
        return ImageDraw.Draw(self.image)

    def outline(self, color=INK):
        alpha = self.image.split()[3].point(lambda a: 255 if a else 0)
        grown = alpha.filter(ImageFilter.MaxFilter(3))
        edge = ImageChops.subtract(grown, alpha)
        ink = Image.new("RGBA", self.size, color)
        self.image.paste(ink, (0, 0), edge)

    def erase(self, draw_shape):
        mask, draw = self.layer()
        draw_shape(draw)
        clear = Image.new("RGBA", self.size, (0, 0, 0, 0))
        self.image.paste(clear, (0, 0), mask)


def sheet(frames):
    width = sum(f.size[0] for f in frames)
    out = Image.new("RGBA", (width, frames[0].size[1]), (0, 0, 0, 0))
    x = 0
    for frame in frames:
        out.paste(frame, (x, 0))
        x += frame.size[0]
    return out


WOOD = [(236, 190, 120), (212, 158, 92), (178, 122, 66), (140, 90, 50), (100, 62, 40)]
WOOD_DARK = [(150, 100, 58), (118, 76, 46), (90, 56, 38), (66, 40, 32)]
IRON = [(190, 196, 208), (140, 148, 164), (98, 104, 124), (64, 68, 88)]


def crate_frame(stage):
    c = Canvas(96, 96)
    rng = random.Random(7 + stage)
    if stage == 2:
        for k in range(7):
            x = 10 + k * 11 + rng.randint(-3, 3)
            y = 82 + rng.randint(-4, 3)
            w = rng.randint(22, 34)
            tilt = rng.randint(-7, 7)
            c.shape(WOOD if k % 2 else WOOD_DARK, lambda d: d.polygon([(x, y), (x + w, y + tilt), (x + w, y + tilt + 7), (x, y + 7)], fill=255))
        c.shape(WOOD, lambda d: d.polygon([(30, 60), (40, 56), (52, 84), (42, 88)], fill=255))
        c.shape(WOOD_DARK, lambda d: d.polygon([(58, 64), (66, 62), (70, 86), (60, 88)], fill=255))
        d = c.draw()
        for _ in range(10):
            x, y = rng.randint(8, 86), rng.randint(84, 92)
            d.point((x, y), fill=(236, 190, 120, 255))
        c.outline()
        return c.image
    left, top, right, bottom = 8, 14, 88, 94
    c.shape(WOOD_DARK, lambda d: d.rectangle([left, top, right, bottom], fill=255))
    for k in range(5):
        x = left + 6 + k * 14
        c.shape(WOOD, lambda d: d.rectangle([x, top + 8, x + 11, bottom - 8], fill=255), lambda u, v: 0.25 + v * 0.5 + (0.12 if k % 2 else 0))
    for y in (top, bottom - 9):
        c.shape(WOOD, lambda d: d.rectangle([left, y, right, y + 9], fill=255), lambda u, v: 0.1 + v * 0.6)
    for x in (left, right - 9):
        c.shape(WOOD, lambda d: d.rectangle([x, top, x + 9, bottom], fill=255), lambda u, v: 0.2 + u * 0.4)
    c.shape(WOOD, lambda d: d.polygon([(left + 9, top + 16), (left + 16, top + 9), (right - 9, bottom - 16), (right - 16, bottom - 9)], fill=255), lambda u, v: 0.15 + v * 0.5)
    d = c.draw()
    for x, y in ((left + 4, top + 4), (right - 5, top + 4), (left + 4, bottom - 5), (right - 5, bottom - 5)):
        d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=IRON[0] + (255,))
        d.point((x + 1, y + 1), fill=IRON[3] + (255,))
    d.line([(left + 2, top + 1), (right - 2, top + 1)], fill=(250, 220, 160, 255))
    if stage == 1:
        crack = (58, 34, 30, 255)
        d.line([(30, top + 2), (36, 34), (30, 48), (40, 62), (34, bottom - 2)], fill=crack, width=2)
        d.line([(36, 34), (52, 40), (60, 52)], fill=crack, width=2)
        d.line([(66, top + 4), (62, 30)], fill=crack, width=2)
        c.erase(lambda m: m.polygon([(right - 16, top), (right + 1, top), (right + 1, top + 14)], fill=255))
        c.erase(lambda m: m.polygon([(left - 1, 60), (left + 8, 66), (left - 1, 74)], fill=255))
    c.outline()
    return c.image


DRUM = [(120, 214, 226), (62, 170, 196), (34, 124, 160), (26, 84, 122), (24, 54, 86)]
DRUM_RUST = [(214, 132, 72), (170, 92, 52), (120, 60, 42)]


def barrel_frame(stage):
    c = Canvas(80, 112)
    rng = random.Random(21 + stage)
    light = lambda u, v: 0.05 + abs(u - 0.32) * 1.25
    if stage == 2:
        c.shape(DRUM, lambda d: d.polygon([(8, 110), (10, 92), (24, 84), (52, 88), (72, 96), (74, 110)], fill=255), light)
        c.shape(DRUM[2:], lambda d: d.ellipse([18, 80, 60, 96], fill=255))
        c.shape(IRON, lambda d: d.ellipse([22, 83, 56, 93], fill=255))
        d = c.draw()
        d.line([(12, 100), (70, 104)], fill=DRUM[4] + (255,), width=2)
        for _ in range(8):
            x, y = rng.randint(4, 76), rng.randint(100, 110)
            d.rectangle([x, y, x + 2, y + 1], fill=DRUM_RUST[rng.randint(0, 2)] + (255,))
        c.outline()
        return c.image
    lean = 3 if stage == 1 else 0
    body = [(12 + lean, 16), (68 + lean, 16), (70, 106), (10, 106)]
    c.shape(DRUM, lambda d: d.polygon(body, fill=255), light)
    c.shape(DRUM, lambda d: d.ellipse([10, 98, 70, 111], fill=255), lambda u, v: 0.5 + v * 0.5)
    for y in (38, 72):
        c.shape(DRUM, lambda d: d.rectangle([9 + lean * (106 - y) // 90, y, 71 + lean * (106 - y) // 90 - lean, y + 5], fill=255), lambda u, v: abs(u - 0.32) * 1.1 - 0.08 + v * 0.2)
    c.shape(IRON, lambda d: d.ellipse([12 + lean, 8, 68 + lean, 24], fill=255), lambda u, v: 0.2 + v * 0.6)
    c.shape(DRUM[1:], lambda d: d.ellipse([16 + lean, 10, 64 + lean, 21], fill=255), lambda u, v: 0.2 + u * 0.6)
    d = c.draw()
    d.ellipse([24 + lean, 13, 30 + lean, 17], fill=IRON[3] + (255,))
    d.rectangle([34, 48, 52, 64], fill=(250, 214, 80, 255))
    d.polygon([(43, 50), (50, 62), (36, 62)], fill=(40, 30, 40, 255))
    d.rectangle([42, 54, 43, 58], fill=(250, 214, 80, 255))
    d.point((42, 60), fill=(250, 214, 80, 255))
    for _ in range(9):
        x, y = rng.randint(14, 64), rng.randint(84, 104)
        d.rectangle([x, y, x + rng.randint(1, 3), y + 1], fill=DRUM_RUST[rng.randint(0, 2)] + (255,))
    if stage == 1:
        c.shape(DRUM[3:], lambda m: m.ellipse([40, 24, 66, 48], fill=255), lambda u, v: 0.2 + v * 0.8)
        c.shape(DRUM[3:], lambda m: m.ellipse([14, 76, 34, 96], fill=255), lambda u, v: 0.2 + v * 0.8)
        d = c.draw()
        d.line([(44, 28), (60, 40)], fill=DRUM[0] + (255,))
        d.line([(20, 30), (26, 44), (20, 58)], fill=DRUM[4] + (255,), width=2)
        for y in range(100, 110, 2):
            d.rectangle([6, y, 9, y + 1], fill=(40, 30, 40, 255))
    c.outline()
    return c.image


PAINT = [(255, 214, 120), (250, 160, 60), (226, 104, 40), (176, 62, 38), (116, 36, 40)]
BURNT = [(150, 130, 128), (108, 88, 92), (74, 58, 68), (50, 38, 50), (34, 26, 38)]
GLASS = [(214, 244, 250), (140, 208, 232), (76, 150, 200), (48, 96, 156)]
TIRE = [(92, 88, 104), (58, 54, 70), (36, 32, 46), (24, 20, 32)]
CHROME = [(244, 246, 250), (190, 198, 212), (128, 138, 160), (80, 86, 110)]
CABIN_DARK = [(52, 40, 60), (36, 28, 46), (24, 18, 34)]


def wheel(c, cx, cy, radius, flat):
    squash = 6 if flat else 0
    c.shape(TIRE, lambda d: d.ellipse([cx - radius - squash // 2, cy - radius + squash, cx + radius + squash // 2, cy + radius], fill=255), lambda u, v: 0.2 + v * 0.7)
    inner = radius - 9
    c.shape(CHROME, lambda d: d.ellipse([cx - inner, cy - inner + squash, cx + inner, cy + inner], fill=255), lambda u, v: u * 0.5 + v * 0.5)
    c.shape(TIRE[1:], lambda d: d.ellipse([cx - 5, cy - 5 + squash // 2, cx + 5, cy + 5 + squash // 2], fill=255))
    d = c.draw()
    for k in range(5):
        a = k * math.tau / 5 + 0.3
        d.line([(cx, cy + squash // 2), (cx + math.cos(a) * inner, cy + squash // 2 + math.sin(a) * (inner - squash // 2))], fill=CHROME[3] + (255,), width=2)


def car_frame(stage):
    c = Canvas(416, 176)
    rng = random.Random(40 + stage)
    wreck = stage == 3
    paint = BURNT if wreck else PAINT
    ground = 172
    sag = 14 if wreck else 0
    roof = 44 + (26 if wreck else 0)
    belt = 92 + sag // 2
    sill = 142 + sag // 2
    body_light = lambda u, v: 0.08 + v * 0.8
    for cx in (96, 318):
        c.shape(CABIN_DARK, lambda d: d.ellipse([cx - 40, sill - 44, cx + 40, sill + 30], fill=255))
    wheel(c, 96, ground - 26 + (4 if wreck else 0), 26, stage >= 2)
    wheel(c, 318, ground - 26 + (4 if wreck else 0), 26, wreck)
    lower = [(14, belt + 6), (30, belt - 2), (396, belt + 2), (404, belt + 16), (404, sill - 6), (392, sill), (22, sill), (12, sill - 10)]
    c.shape(paint, lambda d: d.polygon(lower, fill=255), body_light)
    cabin = [(116, belt), (150, roof + 2), (166, roof), (272, roof), (292, roof + 4), (336, belt)]
    if wreck:
        cabin = [(116, belt), (140, roof + 12), (170, roof + 2), (266, roof + 8), (300, roof + 14), (336, belt)]
    c.shape(paint, lambda d: d.polygon(cabin, fill=255), lambda u, v: 0.02 + v * 0.55)
    for cx in (96, 318):
        c.erase(lambda m: m.ellipse([cx - 34, sill - 38, cx + 34, sill + 30], fill=255))
        c.shape(CABIN_DARK, lambda d: d.pieslice([cx - 34, sill - 38, cx + 34, sill + 30], 180, 360, fill=255))
    wheel(c, 96, ground - 26 + (4 if wreck else 0), 26, stage >= 2)
    wheel(c, 318, ground - 26 + (4 if wreck else 0), 26, wreck)
    rear_glass = [(132, belt - 4), (156, roof + 10), (206, roof + 8), (206, belt - 4)]
    front_glass = [(216, belt - 4), (216, roof + 8), (268, roof + 8), (318, belt - 4)]
    if wreck:
        rear_glass = [(134, belt - 4), (150, roof + 20), (206, roof + 12), (206, belt - 4)]
        front_glass = [(216, belt - 4), (216, roof + 14), (270, roof + 18), (316, belt - 4)]
    for glass in (rear_glass, front_glass):
        if stage >= 2:
            c.shape(CABIN_DARK, lambda d: d.polygon(glass, fill=255), lambda u, v: v)
        else:
            c.shape(GLASS, lambda d: d.polygon(glass, fill=255), lambda u, v: 0.1 + v * 0.5 + u * 0.3)
    d = c.draw()
    ink = INK
    if stage < 2:
        for x0 in (150, 232, 252):
            d.line([(x0, belt - 8), (x0 + 22, roof + 12)], fill=(240, 252, 255, 255), width=3)
    d.line([(211, roof + 2), (211, sill - 4)], fill=ink, width=2)
    d.line([(124, belt + 2), (124, sill - 22)], fill=ink, width=1)
    d.line([(300, belt + 2), (300, sill - 8)], fill=ink, width=1)
    d.line([(18, belt + 8), (400, belt + 8)], fill=paint[0] + (255,), width=2)
    stripe = (250, 244, 228, 255) if not wreck else (120, 108, 110, 255)
    d.rectangle([40, belt + 22, 392, belt + 27], fill=stripe)
    d.rectangle([40, belt + 30, 392, belt + 31], fill=stripe)
    for cx in (96, 318):
        c.shape(CABIN_DARK, lambda m: m.pieslice([cx - 34, sill - 38, cx + 34, sill + 30], 180, 360, fill=255))
    wheel(c, 96, ground - 26 + (4 if wreck else 0), 26, stage >= 2)
    wheel(c, 318, ground - 26 + (4 if wreck else 0), 26, wreck)
    d = c.draw()
    for x in (176, 262):
        d.rectangle([x, belt + 12, x + 16, belt + 15], fill=CHROME[1] + (255,))
        d.line([(x, belt + 16), (x + 16, belt + 16)], fill=ink)
    c.shape(CHROME, lambda m: m.rounded_rectangle([386, sill - 20, 410, sill - 6], 4, fill=255))
    c.shape(CHROME, lambda m: m.rounded_rectangle([6, sill - 20, 30, sill - 6], 4, fill=255))
    d = c.draw()
    if stage < 2:
        d.rounded_rectangle([384, belt + 10, 402, belt + 22], 3, fill=(255, 250, 200, 255), outline=ink)
        d.point((396, belt + 13), fill=(255, 255, 255, 255))
    else:
        d.rounded_rectangle([384, belt + 10, 402, belt + 22], 3, fill=(60, 50, 66, 255), outline=ink)
    d.rounded_rectangle([14, belt + 10, 26, belt + 22], 3, fill=(236, 64, 70, 255) if stage < 3 else (90, 50, 60, 255), outline=ink)
    d.polygon([(292, belt - 2), (300, belt - 12), (306, belt - 2)], fill=paint[2] + (255,), outline=ink)
    if stage >= 1:
        crack = (236, 250, 255, 255) if stage == 1 else (20, 14, 26, 255)
        for ox, oy in ((176, roof + 26), (262, roof + 30)):
            for k in range(7):
                a = k * math.tau / 7 + 0.4
                reach = rng.randint(8, 20)
                d.line([(ox, oy), (ox + math.cos(a) * reach, oy + math.sin(a) * reach * 0.7)], fill=crack)
        for _ in range(4 + stage * 3):
            x, y = rng.randint(30, 380), rng.randint(belt + 12, sill - 12)
            w, h = rng.randint(10, 22), rng.randint(5, 10)
            d.ellipse([x, y, x + w, y + h], fill=paint[3] + (255,))
            d.arc([x, y, x + w, y + h], 200, 340, fill=paint[4] + (255,))
            d.arc([x, y + 1, x + w, y + h + 1], 20, 160, fill=paint[1] + (255,))
    if stage >= 2:
        hood = [(336, belt), (398, belt - 34 + sag), (404, belt - 28 + sag), (346, belt + 6)]
        c.shape(paint, lambda m: m.polygon(hood, fill=255), lambda u, v: 0.2 + v * 0.6)
        c.shape(CABIN_DARK, lambda m: m.polygon([(346, belt + 6), (396, belt + 4), (396, belt + 12), (346, belt + 12)], fill=255))
        d = c.draw()
        d.line([(336, belt), (398, belt - 34 + sag)], fill=ink, width=2)
        d.line([(388, sill - 8), (412, sill + 4)], fill=CHROME[2] + (255,), width=4)
        for _ in range(26):
            x, y = rng.randint(120, 330), rng.randint(belt - 8, belt + 2)
            d.point((x, y), fill=GLASS[rng.randint(0, 2)] + (255,))
    if wreck:
        d = c.draw()
        for _ in range(60):
            x, y = rng.randint(20, 398), rng.randint(roof + 8, sill - 4)
            if c.image.getpixel((x, y))[3]:
                d.point((x, y), fill=BURNT[rng.randint(3, 4)] + (255,))
        d.line([(120, belt + 4), (150, sill - 6), (210, sill - 2)], fill=ink, width=2)
    c.outline()
    return c.image


def glow_frame(base, phase):
    frame = base.copy()
    px = frame.load()
    w, h = frame.size
    band = int((phase / 4.0) * (w + h))
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a and (r, g, b) != INK[:3] and abs((x + y) - band) < 4:
                px[x, y] = (min(255, r + 70), min(255, g + 70), min(255, b + 70), a)
    return frame


def apple():
    c = Canvas(48, 48)
    red = [(255, 170, 150), (240, 82, 70), (196, 44, 56), (140, 28, 52)]
    c.shape(red, lambda d: d.ellipse([8, 16, 28, 44], fill=255), lambda u, v: 0.1 + u * 0.4 + v * 0.5)
    c.shape(red, lambda d: d.ellipse([20, 16, 40, 44], fill=255), lambda u, v: 0.25 + u * 0.4 + v * 0.45)
    d = c.draw()
    d.line([(24, 18), (26, 9)], fill=(110, 70, 40, 255), width=2)
    d.ellipse([27, 6, 38, 13], fill=(110, 200, 90, 255), outline=(50, 120, 60, 255))
    d.rectangle([13, 22, 16, 27], fill=(255, 236, 226, 255))
    c.outline()
    return c.image


def pizza():
    c = Canvas(48, 48)
    crust = [(255, 220, 150), (236, 176, 96), (196, 126, 62), (150, 88, 48)]
    cheese = [(255, 244, 170), (255, 214, 96), (244, 170, 60)]
    c.shape(crust, lambda d: d.polygon([(4, 14), (44, 10), (46, 20), (22, 46), (16, 44)], fill=255))
    c.shape(cheese, lambda d: d.polygon([(8, 19), (42, 17), (22, 41)], fill=255), lambda u, v: v * 0.8 + u * 0.2)
    d = c.draw()
    for x, y in ((16, 22), (30, 22), (23, 31)):
        d.ellipse([x - 4, y - 3, x + 4, y + 4], fill=(214, 60, 52, 255), outline=(140, 30, 40, 255))
        d.point((x - 1, y - 1), fill=(255, 150, 130, 255))
    for x, y in ((24, 24), (34, 19), (19, 30)):
        d.rectangle([x, y, x + 2, y + 1], fill=(70, 150, 70, 255))
    d.line([(6, 13), (42, 10)], fill=(255, 236, 190, 255))
    c.outline()
    return c.image


def roast():
    c = Canvas(48, 48)
    meat = [(255, 204, 130), (236, 150, 72), (196, 100, 48), (140, 64, 40)]
    plate = [(250, 250, 255), (206, 212, 230), (150, 158, 186)]
    c.shape(plate, lambda d: d.ellipse([2, 30, 46, 46], fill=255))
    c.shape(meat, lambda d: d.ellipse([8, 12, 38, 40], fill=255), lambda u, v: 0.1 + v * 0.6 + u * 0.25)
    c.shape(meat, lambda d: d.polygon([(30, 20), (40, 12), (44, 16), (36, 28)], fill=255), lambda u, v: 0.3 + v * 0.5)
    d = c.draw()
    d.ellipse([39, 7, 46, 14], fill=(250, 246, 236, 255), outline=(170, 160, 150, 255))
    d.arc([12, 15, 28, 30], 180, 270, fill=(255, 236, 190, 255), width=2)
    for x in (10, 20, 34):
        d.ellipse([x, 36, x + 5, 41], fill=(96, 180, 90, 255), outline=(50, 110, 60, 255))
    c.outline()
    return c.image


def gem():
    c = Canvas(48, 48)
    blue = [(214, 250, 255), (120, 220, 255), (60, 150, 240), (40, 84, 190)]
    c.shape(blue, lambda d: d.polygon([(24, 6), (40, 20), (24, 44), (8, 20)], fill=255), lambda u, v: u * 0.6 + v * 0.4)
    d = c.draw()
    d.polygon([(24, 6), (30, 20), (24, 44), (18, 20)], fill=(150, 232, 255, 255))
    d.line([(8, 20), (40, 20)], fill=(230, 252, 255, 255))
    d.line([(24, 6), (18, 20)], fill=(240, 255, 255, 255))
    c.outline()
    return c.image


def battery():
    c = Canvas(48, 48)
    can = [(255, 250, 170), (255, 214, 70), (236, 150, 40), (170, 96, 40)]
    c.shape(can, lambda d: d.rounded_rectangle([12, 10, 36, 44], 5, fill=255), lambda u, v: abs(u - 0.3) * 1.3)
    c.shape(CHROME, lambda d: d.rectangle([18, 5, 30, 11], fill=255))
    d = c.draw()
    d.polygon([(27, 14), (17, 28), (23, 28), (20, 40), (31, 24), (25, 24)], fill=(50, 40, 70, 255))
    c.outline()
    return c.image


def write(name, frames):
    OUT.mkdir(parents=True, exist_ok=True)
    sheet(frames).save(OUT / f"{name}.png")


def main():
    write("crate", [crate_frame(s) for s in range(3)])
    write("barrel", [barrel_frame(s) for s in range(3)])
    write("car", [car_frame(s) for s in range(4)])
    for name, make in (("apple", apple), ("pizza", pizza), ("roast", roast), ("gem", gem), ("battery", battery)):
        base = make()
        write(name, [glow_frame(base, phase) if phase < 4 else base for phase in (9, 0, 1, 2, 3, 9)])


if __name__ == "__main__":
    main()
