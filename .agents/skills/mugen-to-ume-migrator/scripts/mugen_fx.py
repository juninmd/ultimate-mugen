from mugen_parse import nums

MAX_VFX = 3


def pair(text):
    values = nums(text, (0.0, 0.0))
    return [round(values[0], 2), round(values[1], 2)]


def vfx_from(ctrl, at, actions, name):
    b = ctrl["body"]
    anim = b.get("anim", "").strip()
    if not anim or anim.upper().startswith("F"):
        return None, []
    number = int(nums(anim, (0,))[0])
    if number not in actions:
        return None, []
    out = {"at": at, "anim": name(number), "offset": pair(b.get("pos"))}
    time = int(nums(b.get("removetime"), (-2,))[0])
    if time > 0:
        out["time"] = time
    if b.get("bindtime"):
        out["follow"] = True
    if b.get("scale"):
        scale = nums(b["scale"], (1.0,))[0]
        if scale != 1.0:
            out["scale"] = round(scale, 2)
    if b.get("ontop", "").strip() == "0":
        out["behind"] = True
    return out, [number]


def quake_from(ctrl, at):
    b = ctrl["body"]
    out = {"at": at, "time": int(nums(b.get("time"), (8,))[0])}
    amp = abs(nums(b.get("ampl"), (4,))[0])
    if amp != 4:
        out["amp"] = round(amp, 2)
    return out
