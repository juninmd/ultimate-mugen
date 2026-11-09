import colorsys, copy, json, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent / "assets" / "chars"
KEEP_KEYS = {"skin", "eye", "lips", "beard", "weapon"}


def shift(color, degrees, sat=1.0, val=1.0):
    if not (isinstance(color, str) and color.startswith("#") and len(color) == 7):
        return color
    r, g, b = (int(color[i:i + 2], 16) / 255 for i in (1, 3, 5))
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    h = (h + degrees / 360) % 1
    r, g, b = colorsys.hsv_to_rgb(h, min(1, s * sat), min(1, v * val))
    return "#%02x%02x%02x" % (round(r * 255), round(g * 255), round(b * 255))


def recolor(mapping, degrees, sat=1.0, val=1.0):
    return {k: (v if k in KEEP_KEYS else shift(v, degrees, sat, val)) for k, v in mapping.items()}


def scale_pair(pair, k):
    return [round(x * k, 2) for x in pair]


def move(data, move_id):
    return next(m for m in data["moves"] if m["id"] == move_id)


def brute(d):
    for mid, hits in (("strong", 1), ("hkick", 1)):
        move(d, mid)["armor"] = {"hits": hits, "threshold": 90}


def blade(d):
    d["speed"] = round(d.get("speed", 1.0) * 1.25, 3)
    move(d, "hkick")["moves"] = [{"at": 1, "vel": [6.5, 0]}]
    move(d, "lkick")["moves"] = [{"at": 1, "vel": [4.5, 0]}]


def pyro(d):
    orb = copy.deepcopy(move(d, "starburst")["projectile"])
    orb["damage"] = 34
    orb["speed"] = [7.5, 0]
    move(d, "jab")["projectile"] = orb


def colossus(d):
    d["stats"]["life"] = round(d["stats"]["life"] * 1.6)
    d["stats"]["walk"] = scale_pair(d["stats"]["walk"], 0.78)
    d["stats"]["run"] = scale_pair(d["stats"]["run"], 0.78)
    d["stats"]["height"] = round(d["stats"]["height"] * 1.12)
    for mid in ("jab", "strong", "lkick", "hkick"):
        move(d, mid)["armor"] = {"hits": 2, "threshold": 120}


def phantom(d):
    d["speed"] = round(d.get("speed", 1.0) * 1.15, 3)
    for mid in ("lkick", "hkick"):
        move(d, mid)["invuln"] = [1, 3]
    move(d, "strong")["moves"] = [{"at": 1, "vel": [5.0, 0]}]


NPCS = [
    dict(base="bruno", id="npc_brute", name="Brute", hue=38, sat=1.0, val=1.05, life=0.55, atk=0.8,
         blurb="Street bruiser. His heavy swings carry armor: a light hit does not interrupt them.", ability=brute),
    dict(base="suzume", id="npc_razor", name="Razor", hue=150, sat=1.1, val=1.0, life=0.5, atk=0.85,
         blurb="Knife runner. Fast on his feet and lunges forward with every kick.", ability=blade),
    dict(base="akio", id="npc_spitfire", name="Spitfire", hue=95, sat=1.1, val=1.0, life=0.5, atk=0.8,
         blurb="Firebug. His jab throws a small orb, so he pressures from range.", ability=pyro),
    dict(base="daichi", id="npc_colossus", name="Colossus", hue=-60, sat=0.9, val=0.9, life=0.9, atk=0.95,
         blurb="Mad Gear giant. Slow, tall and armored on every basic strike.", ability=colossus),
    dict(base="jin", id="npc_phantom", name="Phantom", hue=210, sat=0.8, val=0.85, life=0.5, atk=0.85,
         blurb="Shadow runner. Quick on foot and invulnerable while he kicks.", ability=phantom),
]


def build(spec):
    d = json.loads((ROOT / f"{spec['base']}.json").read_text(encoding="utf-8"))
    d["id"] = spec["id"]
    d["name"] = spec["name"]
    d["series"] = "Story Mode · Street Thugs"
    d["description"] = spec["blurb"]
    d["boss"] = False
    d["npc"] = True
    for key in ("intro", "taunt", "winQuote", "form"):
        d.pop(key, None)
    d.pop("abilities", None)
    d["spriteSet"] = spec["base"]
    style = d["art"]["style"]
    base_colors = {k: v for k, v in style["colors"].items() if k not in KEEP_KEYS}
    first = recolor(base_colors, spec["hue"], spec["sat"], spec["val"])
    second = recolor(base_colors, spec["hue"] + 120, spec["sat"], spec["val"])
    d["palettes"] = [first, second]
    d["stats"]["life"] = round(d["stats"]["life"] * spec["life"])
    d["stats"]["attack"] = round(d["stats"]["attack"] * spec["atk"])
    d["color"] = first.get("top", d.get("color", "#cccccc"))
    spec["ability"](d)
    return d


def main():
    index_path = ROOT / "index.json"
    index = json.loads(index_path.read_text(encoding="utf-8"))
    for spec in NPCS:
        data = build(spec)
        (ROOT / f"{spec['id']}.json").write_text(json.dumps(data, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
        if spec["id"] not in index:
            index.append(spec["id"])
    index_path.write_text(json.dumps(index, ensure_ascii=False) + "\n", encoding="utf-8")


main()
