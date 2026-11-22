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


def _char_def(folder):
    defs = sorted(f for f in os.listdir(folder) if f.lower().endswith(".def"))
    parsed = [(f, {name.lower(): body for name, body, _ in sections(read_text(os.path.join(folder, f)))}) for f in defs]
    chars = [(f, get) for f, get in parsed if get.get("files", {}).get("sprite") and get.get("files", {}).get("cmd")]
    if not chars:
        raise ValueError(f"no character .def with [Files] sprite and cmd in {folder}")
    base = os.path.basename(os.path.normpath(folder)).lower()
    return next((get for f, get in chars if f[:-4].lower() == base), chars[0][1])


def read_def(folder):
    get = _char_def(folder)
    files = get.get("files", {})
    listing = {f.lower(): f for f in os.listdir(folder)}
    path = lambda key: os.path.join(folder, listing.get(files[key].strip('"').lower(), files[key].strip('"'))) if files.get(key) else None
    st_keys = sorted((k for k in files if re.fullmatch(r"st\d*", k)), key=lambda k: int(k[2:] or -1))
    cns = [path(k) for k in ["cns"] + st_keys if path(k)]
    extra = sorted((int(k[3:]), path(k)) for k in files if re.fullmatch(r"pal\d+", k) and int(k[3:]) > 1)
    return {"info": get.get("info", {}), "cmd": path("cmd"), "cns": list(dict.fromkeys(cns)), "sprite": path("sprite"),
            "anim": path("anim"), "sound": path("sound"), "pal": path("pal1"), "stcommon": path("stcommon"), "pals": [p for _, p in extra]}


def read_air(path):
    parts = re.split(r"(?im)^\s*\[begin action\s+(-?\d+)\s*\]", read_text(path))
    actions = {}
    for k in range(1, len(parts), 2):
        number = int(parts[k])
        if number in actions:
            continue
        frames, clsn1, clsn2, default1, default2, mode, loop = [], [], [], [], [], None, None
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
                mode = m.group(1) + ("d" if m.group(2) else "")
                if mode == "1d":
                    default1 = []
                elif mode == "2d":
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
                {"1d": default1, "2d": default2, "1": clsn1}.get(mode, clsn2).append(box)
                continue
            fields = [f.strip() for f in line.split(",")]
            if len(fields) >= 5 and re.match(r"^-?\d+$", fields[0]) and re.match(r"^-?\d+$", fields[1]):
                g, i, x, y, t = (int(float(v)) for v in fields[:5])
                flip = fields[5].upper() if len(fields) > 5 else ""
                frames.append({"g": g, "i": i, "x": x, "y": y, "t": t, "flip": flip,
                               "clsn1": clsn1 or default1, "clsn2": clsn2 or default2})
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
        current, seen = None, set()
        for name, body, pairs in sections(read_text(path)):
            m = re.match(r"(?i)statedef\s+(-?\d+)", name)
            if m:
                current = int(m.group(1))
                if current >= 0 and current in seen:
                    current = None
                elif current < 0 and current in states:
                    seen.add(current)
                else:
                    seen.add(current)
                    states[current] = {"def": body, "ctrl": []}
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


def _shares_act(sff, s, pal):
    if sff.legacy_shared or s.root.pal is None:
        return True
    return sum(s.root.pal[k:k + 3] == pal[k:k + 3] for k in range(0, 768, 3)) >= 250


def sprite_image(sff, pal, g, i):
    s = sff.index.get((g, i))
    if s is None:
        return None, None
    p = pal if pal is not None and (s is sff.sprites[0] or (s.same_pal and _shares_act(sff, s, pal))) else None
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
