import re

from .chain import triggers
from .parse import nums

KINDS = ("velmul", "posfreeze", "posset", "palfx")
ALWAYS = re.compile(r"^\s*(1|time\s*>=\s*0|animtime\s*<=\s*0)\s*$")
TIMED = re.compile(r"^\s*(time\s*=\s*\d+|animelem\s*=\s*\d+)\s*$")


def _number(text):
    m = re.match(r"\s*(-?\d*\.?\d+)", text or "")
    return float(m.group(1)) if m else None


def _triple(text):
    v = nums(text)
    return [int(x) for x in v[:3]] if len(v) >= 3 else None


def _when(ctrl, span):
    lines = triggers(ctrl)
    if lines and all(ALWAYS.match(line) for line in lines):
        return "range", span
    if ctrl.get("at") and all(TIMED.match(line) for line in lines):
        return "at", ctrl["at"][0]
    return None, None


def _axes(body, out):
    for key in ("x", "y"):
        if key in body and _number(body[key]) is not None:
            out[key] = _number(body[key])
    return out


def motion(ctrls):
    out, dropped = {"velMul": [], "velAdd": [], "posFreeze": [], "posSet": [], "palFx": []}, []
    for c in ctrls:
        kind = c["type"]
        span = c.get("span", (1, 0))
        mode, where = _when(c, span)
        if kind == "veladd" and mode == "range":
            spec = _axes(c["body"], {"from": where[0], "to": where[1]})
            if "x" in spec or "y" in spec:
                out["velAdd"].append(spec)
            continue
        if kind not in KINDS:
            continue
        if kind in ("velmul", "posfreeze") and mode == "at":
            mode, where = "range", (where, span[1])
        body = c["body"]
        if kind == "velmul" and mode == "range":
            spec = _axes(body, {"from": where[0], "to": where[1]})
            if "x" in spec or "y" in spec:
                out["velMul"].append(spec)
                continue
        elif kind == "posfreeze" and mode == "range" and _number(body.get("value", "1")) != 0:
            out["posFreeze"].append({"from": where[0], "to": where[1]})
            continue
        elif kind == "posset" and mode == "at" or kind == "posset" and mode == "range" and where[0] == span[0]:
            spec = _axes(body, {"at": where if mode == "at" else where[0]})
            if "x" in spec or "y" in spec:
                out["posSet"].append(spec)
                continue
        elif kind == "palfx" and mode == "at":
            spec = {"at": where, "time": int(nums(body.get("time"), (1,))[0])}
            for key, name in (("add", "add"), ("mul", "mul")):
                if _triple(body.get(key)):
                    spec[name] = _triple(body.get(key))
            sin = nums(body.get("sinadd"))
            if len(sin) >= 4:
                spec["sinAdd"] = [int(x) for x in sin[:4]]
            if body.get("color") and _number(body["color"]) is not None:
                spec["color"] = _number(body["color"])
            if _number(body.get("invertall")):
                spec["invertAll"] = True
            out["palFx"].append(spec)
            continue
        dropped.append(kind)
    return {k: v for k, v in out.items() if v}, dropped
