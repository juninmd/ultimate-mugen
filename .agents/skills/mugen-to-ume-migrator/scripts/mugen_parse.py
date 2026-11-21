import io
import os
import re
import struct

from PIL import Image


def read_text(path):
    return open(path, encoding="latin-1").read()


def strip(line):
    return line.split(";")[0].strip()


def sections(text):
    out, cur = [], None
    for raw in text.splitlines():
        line = strip(raw)
        if not line:
            continue
        m = re.match(r"^\[(.+)\]$", line)
        if m:
            cur = (m.group(1).strip(), {}, [])
            out.append(cur)
        elif cur is not None and "=" in line:
            key, value = line.split("=", 1)
            cur[1].setdefault(key.strip().lower(), value.strip())
            cur[2].append((key.strip().lower(), value.strip()))
    return out


def nums(text, default=()):
    values = [float(x) for x in re.findall(r"-?\d*\.?\d+", text or "")]
    return values + list(default[len(values):])


def read_def(folder):
    defs = [f for f in os.listdir(folder) if f.lower().endswith(".def")]
    secs = sections(read_text(os.path.join(folder, defs[0])))
    get = {name.lower(): body for name, body, _ in secs}
    files = get.get("files", {})
    listing = {f.lower(): f for f in os.listdir(folder)}
    path = lambda key: os.path.join(folder, listing.get(files[key].strip('"').lower(), files[key].strip('"'))) if files.get(key) else None
    cns = [path(k) for k in ("cns", "st", "st1", "st2", "st3", "st4") if path(k)]
    return {"info": get.get("info", {}), "cmd": path("cmd"), "cns": list(dict.fromkeys(cns)), "sprite": path("sprite"),
            "anim": path("anim"), "sound": path("sound"), "pal": path("pal1")}


def read_air(path):
    parts = re.split(r"(?im)^\s*\[begin action\s+(-?\d+)\s*\]", read_text(path))
    actions = {}
    for k in range(1, len(parts), 2):
        number = int(parts[k])
        if number in actions:
            continue
        frames, clsn1, clsn2, default2, mode, loop = [], [], [], [], None, None
        for raw in parts[k + 1].splitlines():
            line = strip(raw)
            low = line.lower()
            if not line:
                continue
            if low.startswith("loopstart"):
                loop = len(frames)
                continue
            m = re.match(r"clsn([12])(default)?\s*:\s*(\d+)", low)
            if m:
                mode = ("1", "2d" if m.group(2) else "2")[m.group(1) == "2"]
                if mode == "2d":
                    default2 = []
                elif mode == "1":
                    clsn1 = []
                else:
                    clsn2 = []
                continue
            m = re.match(r"clsn([12])\[\d+\]\s*=\s*(.+)", low)
            if m:
                box = nums(m.group(2))[:4]
                x1, y1, x2, y2 = box
                box = [min(x1, x2), min(y1, y2), max(x1, x2), max(y1, y2)]
                (default2 if mode == "2d" else clsn1 if m.group(1) == "1" else clsn2).append(box)
                continue
            fields = [f.strip() for f in line.split(",")]
            if len(fields) >= 5 and re.match(r"^-?\d+$", fields[0]) and re.match(r"^-?\d+$", fields[1]):
                g, i, x, y, t = (int(float(v)) for v in fields[:5])
                flip = fields[5].upper() if len(fields) > 5 else ""
                frames.append({"g": g, "i": i, "x": x, "y": y, "t": t, "flip": flip,
                               "clsn1": clsn1, "clsn2": clsn2 or default2})
                clsn1, clsn2 = [], []
        actions[number] = {"frames": frames, "loop": loop}
    return actions


def read_commands(path):
    commands = {}
    for name, body, pairs in sections(read_text(path)):
        if name.lower() == "command":
            label = body.get("name", "").strip('"')
            commands.setdefault(label, []).extend(v for k, v in pairs if k == "command")
    return commands


def read_states(paths):
    states = {}
    for path in paths:
        current = None
        for name, body, pairs in sections(read_text(path)):
            m = re.match(r"(?i)statedef\s+(-?\d+)", name)
            if m:
                current = int(m.group(1))
                states.setdefault(current, {"def": body, "ctrl": []})
                continue
            if re.match(r"(?i)state\s", name) and current is not None:
                states[current]["ctrl"].append({"type": body.get("type", "").lower(), "body": body, "pairs": pairs})
    return states


def read_snd(path):
    raw = open(path, "rb").read()
    if not raw.startswith(b"ElecbyteSnd"):
        return {}
    count, offset = struct.unpack_from("<II", raw, 16)
    sounds = {}
    for _ in range(count):
        if offset <= 0 or offset + 16 > len(raw):
            break
        nxt, length, group, index = struct.unpack_from("<IIII", raw, offset)
        data = raw[offset + 16: offset + 16 + length]
        if data[:4] == b"RIFF":
            sounds.setdefault((group, index), data)
        offset = nxt
    return sounds


def sprite_image(sff, pal, g, i):
    s = sff.index.get((g, i))
    if s is None:
        return None, None
    p = pal if (pal is not None and (s.same_pal or s is sff.sprites[0])) else None
    return sff.rgba(s, p), s.axis


def contain_axis(image, axis):
    ax, ay = int(axis[0]), int(axis[1])
    left, top = max(0, -ax), max(0, -ay)
    right, bottom = max(0, ax - image.width), max(0, ay - image.height)
    if not (left or top or right or bottom):
        return image, (ax, ay)
    canvas = Image.new("RGBA", (image.width + left + right, image.height + top + bottom), (0, 0, 0, 0))
    canvas.paste(image, (left, top))
    return canvas, (ax + left, ay + top)


def png_bytes(image):
    out = io.BytesIO()
    image.save(out, "PNG", optimize=True)
    return out.getvalue()
