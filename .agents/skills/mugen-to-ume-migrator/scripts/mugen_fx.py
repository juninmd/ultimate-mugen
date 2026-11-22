from mugen_parse import nums


def extract_fx(ctrls, actions, anim_name_fn, frames):
    explods, shakes, flashes, teleports, extra_anims, seen_tp = [], [], [], [], [], set()
    afterimage, super_pause, invuln = None, None, None
    turn_frames = {(c["at"] or [1])[0] for c in ctrls if c["type"] == "turn"}
    for c in ctrls:
        kind, b, at = c["type"], c["body"], c["at"]
        frame = (at or [1])[0]
        if kind == "explod":
            an = int(nums(b.get("anim"), (-1,))[0])
            if an in actions:
                ex = {"at": frame, "anim": anim_name_fn(an)}
                if b.get("pos"):
                    p = nums(b["pos"], (0, 0))
                    ex["pos"] = [round(p[0], 1), round(p[1], 1)]
                if b.get("postype"):
                    ex["postype"] = b["postype"].strip().lower()
                if b.get("bindtime"):
                    ex["bind"] = int(nums(b["bindtime"], (1,))[0])
                if b.get("removetime"):
                    ex["removeTime"] = int(nums(b["removetime"], (-2,))[0])
                if b.get("sprpriority"):
                    ex["priority"] = int(nums(b["sprpriority"], (3,))[0])
                if b.get("scale"):
                    sc = nums(b["scale"], (1, 1))
                    ex["scale"] = [round(sc[0], 2), round(sc[1], 2)]
                if b.get("supermove", "").strip().startswith("1"):
                    ex["superMove"] = True
                explods.append(ex)
                extra_anims.append(an)
        elif kind == "afterimage" and afterimage is None:
            afterimage = {"at": frame, "time": int(nums(b.get("time"), (25,))[0]), "length": int(nums(b.get("length"), (8,))[0])}
            if b.get("framegap"):
                afterimage["framegap"] = int(nums(b["framegap"], (2,))[0])
            if b.get("trans"):
                afterimage["trans"] = b["trans"].strip().lower()
            for mk, uk in (("palbright", "palBright"), ("palcontrast", "palContrast"), ("paladd", "palAdd")):
                if b.get(mk):
                    v = nums(b[mk], (0, 0, 0))
                    afterimage[uk] = [int(v[0]), int(v[1]), int(v[2])]
        elif kind == "envshake":
            sh = {"at": frame, "time": max(1, int(nums(b.get("time"), (12,))[0]))}
            if b.get("ampl"):
                sh["ampl"] = round(abs(nums(b["ampl"], (4,))[0]), 1)
            if b.get("freq"):
                sh["freq"] = round(nums(b["freq"], (60,))[0], 1)
            shakes.append(sh)
        elif kind == "envcolor":
            col = nums(b.get("value"), (255, 255, 255))
            fl = {"at": frame, "time": max(1, int(nums(b.get("time"), (10,))[0])), "color": [int(col[0]), int(col[1]), int(col[2])]}
            if b.get("under", "").strip().startswith("1"):
                fl["under"] = True
            if b.get("supermove", "").strip().startswith("1"):
                fl["superMove"] = True
            flashes.append(fl)
        elif kind == "superpause" and super_pause is None:
            sp = {"at": frame, "time": max(1, int(nums(b.get("time"), (30,))[0]))}
            if b.get("pos"):
                p = nums(b["pos"], (0, -60))
                sp["pos"] = [round(p[0], 1), round(p[1], 1)]
            if b.get("anim"):
                sp["anim"] = int(nums(b["anim"], (30,))[0])
            super_pause = sp
        elif kind == "nothitby" and invuln is None:
            dur = max(1, int(nums(b.get("time"), (8,))[0]))
            end = min(len(frames), frame + max(1, dur // 4))
            invuln = [frame, max(frame, end)]
        elif kind == "posadd":
            vals = " ".join(b.values()).lower()
            x, y = round(nums(b.get("x"), (0,))[0], 1), round(nums(b.get("y"), (0,))[0], 1)
            key = (frame, x, y, "p2dist" in vals)
            if key in seen_tp:
                continue
            if "p2dist" in vals:
                seen_tp.add(key)
                teleports.append({"at": frame, "mode": "behind", "offset": [x if x else 40.0, y], "turn": True})
            elif abs(x) >= 60 or abs(y) >= 40:
                seen_tp.add(key)
                near_turn = any(abs(tf - frame) <= 4 for tf in turn_frames)
                teleports.append({"at": frame, "mode": "offset", "offset": [x, y], "turn": near_turn})
    tp_out = None if not teleports else (teleports[0] if len(teleports) == 1 else teleports)
    return {"explods": explods, "afterimage": afterimage, "shakes": shakes, "flashes": flashes,
            "super_pause": super_pause, "invuln": invuln, "teleport": tp_out}, extra_anims
