#!/usr/bin/env python3
import collections
import glob
import os
import re
import struct
import sys

HITDEF_KEYS = [
    "damage", "pausetime", "guardflag", "ground.hittime", "ground.velocity",
    "air.velocity", "air.juggle", "getpower", "givepower", "hitsound",
    "sparkno", "ground.type", "animtype", "fall",
]
NORMAL_STATES = ("200", "210", "230", "400", "410", "600", "610")


def text_of(path):
    return open(path, "rb").read().decode("latin1").replace("\r", "")


def files(folder, extension):
    found = {}
    for name in os.listdir(folder):
        if name.lower().endswith("." + extension):
            found[name] = os.path.join(folder, name)
    return [found[name] for name in sorted(found)]


def sff_summary(path):
    data = open(path, "rb").read()
    version = (data[15], data[14], data[13], data[12])
    out = {"version": ".".join(map(str, version)), "bytes": len(data)}
    if version[0] == 1:
        groups, images, first = struct.unpack("<III", data[16:28])
        per_group = collections.Counter()
        linked = 0
        offset, seen = first, 0
        while offset and seen < images:
            nxt, length = struct.unpack("<II", data[offset:offset + 8])
            group = struct.unpack("<h", data[offset + 12:offset + 14])[0]
            per_group[group] += 1
            linked += length == 0
            seen += 1
            offset = nxt
        out.update(images=images, groups=len(per_group), linked=linked)
        out["largest_groups"] = per_group.most_common(6)
    else:
        sprites, = struct.unpack("<I", data[0x28:0x2C])
        palettes, = struct.unpack("<I", data[0x30:0x34])
        out.update(sprites=sprites, palettes=palettes)
    return out


def snd_summary(path):
    data = open(path, "rb").read()
    count, offset = struct.unpack("<II", data[16:24])
    groups = collections.Counter()
    sizes = []
    for _ in range(count):
        nxt, length, group, _sound = struct.unpack("<IIII", data[offset:offset + 16])
        groups[group] += 1
        sizes.append(length)
        if nxt == 0:
            break
        offset = nxt
    return {"sounds": count, "groups": len(groups), "largest_bytes": max(sizes or [0])}


def air_summary(path):
    text = text_of(path)
    actions = re.findall(r"(?im)^\s*\[begin action\s+(\d+)\]", text)
    return {
        "actions": len(actions),
        "clsn1_boxes": len(re.findall(r"(?i)clsn1\[\d+\]", text)),
        "clsn2_boxes": len(re.findall(r"(?i)clsn2\[\d+\]", text)),
        "default_clsn_blocks": len(re.findall(r"(?i)clsn\d+default", text)),
        "loopstart": text.lower().count("loopstart"),
        "ids": actions,
    }


def cmd_summary(path):
    text = text_of(path)
    pattern = r'(?im)^\s*\[command\]\s*\n\s*name\s*=\s*"?([^"\n;]+)"?\s*\n\s*command\s*=\s*([^\n;]+)'
    return [(n.strip(), c.strip()) for n, c in re.findall(pattern, text)]


def cns_summary(path):
    text = text_of(path)
    states = re.findall(r"(?im)^\s*\[statedef\s+(-?\d+)\s*\]", text)
    controllers = collections.Counter(
        x.lower() for x in re.findall(r"(?im)^\s*type\s*=\s*(\w+)", text)
    )
    return {"statedefs": states, "controllers": controllers}


def hitdef_rows(path, wanted=NORMAL_STATES):
    text = text_of(path)
    rows = []
    for block in re.split(r"(?im)^\s*\[statedef\s+", text)[1:]:
        number = block.split("]")[0].strip()
        if number not in wanted:
            continue
        hit = re.search(r"(?is)type\s*=\s*hitdef(.*?)(?=^\s*\[)", block, re.M)
        if not hit:
            continue
        values = {}
        for key in HITDEF_KEYS:
            match = re.search(r"(?im)^\s*" + re.escape(key) + r"\s*=\s*([^;\n]+)", hit.group(1))
            if match:
                values[key] = match.group(1).strip()
        juggle = re.search(r"(?im)^\s*juggle\s*=\s*(\d+)", block)
        poweradd = re.search(r"(?im)^\s*poweradd\s*=\s*(-?\d+)", block)
        values["juggle"] = juggle and juggle.group(1)
        values["poweradd"] = poweradd and poweradd.group(1)
        rows.append((number, values))
    return rows


def summary(folder):
    for path in files(folder, "sff"):
        print("SFF", os.path.basename(path), sff_summary(path))
    for path in files(folder, "snd"):
        print("SND", os.path.basename(path), snd_summary(path))
    for path in files(folder, "air"):
        info = air_summary(path)
        ids = info.pop("ids")
        print("AIR", os.path.basename(path), info)
        print("  action ids:", ",".join(ids))
    for path in files(folder, "cmd"):
        commands = cmd_summary(path)
        print("CMD", os.path.basename(path), len(commands), "commands")
        for name, motion in commands:
            print("  ", name, "=", motion)
    for path in files(folder, "cns"):
        info = cns_summary(path)
        print("CNS", os.path.basename(path), len(info["statedefs"]), "statedefs",
              sum(info["controllers"].values()), "controllers")
        print("  ", dict(info["controllers"].most_common(30)))
        print("   states:", ",".join(info["statedefs"]))


def hitdefs(path):
    for number, values in hitdef_rows(path):
        print(number, values)


def main(argv):
    if len(argv) == 3 and argv[1] == "summary":
        summary(argv[2])
    elif len(argv) == 3 and argv[1] == "hitdefs":
        hitdefs(argv[2])
    else:
        print("usage: dissect.py summary <char_dir> | hitdefs <file.cns>")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
