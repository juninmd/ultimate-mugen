import json, glob, sys, collections, os
KINDS = ("normal", "special", "super")
def load(path): return json.load(open(path, encoding="utf8"))
def audit(d):
    out = {"id": d["id"], "problems": []}
    moves = d["moves"]; anims = d["anims"]
    sp = [m for m in moves if m["kind"] == "special"]; su = [m for m in moves if m["kind"] == "super"]
    feats = collections.Counter()
    for m in moves:
        for k in ("cancels", "armor", "parry", "throw", "timestop", "teleport", "flight", "burst", "invuln", "projectile", "airPhysics"):
            if m.get(k): feats[k] += 1
    out.update(specials=len(sp), supers=len(su), normals=sum(1 for m in moves if m["kind"] == "normal"), features=dict(feats), abilities=list((d.get("abilities") or {}).keys()))
    for m in moves:
        a = anims.get(m["anim"])
        if a is None: out["problems"].append(f"{m['id']}: anim '{m['anim']}' missing"); continue
        n = len(a["frames"])
        for h in m.get("hits", []):
            if not 1 <= h["at"] <= n: out["problems"].append(f"{m['id']}: hit at {h['at']} outside anim ({n} frames)"); continue
            f = a["frames"][h["at"] - 1]
            if "hit" not in f and "effect" not in f: out["problems"].append(f"{m['id']}: hit at frame {h['at']} has no hit box")
            elif "hit" in f and f["hit"].get("radius", 99) < 7 and m["kind"] != "normal": out["problems"].append(f"{m['id']}: tiny hit radius {f['hit']['radius']}")
        for p in (m["projectile"] if isinstance(m.get("projectile"), list) else [m["projectile"]] if m.get("projectile") else []):
            if p["anim"] not in anims: out["problems"].append(f"{m['id']}: projectile anim {p['anim']} missing")
        if m["kind"] != "normal" and not m.get("hits") and not m.get("projectile") and not m.get("throw"):
            out["problems"].append(f"{m['id']}: special has no damage source")
    for m in moves:
        a = anims.get(m["anim"])
        airborne = m.get("when") == "air" or any(v["vel"][1] for v in m.get("moves", []))
        if a and not airborne and a["frames"][-1].get("ticks", 1) < 0:
            out["problems"].append(f"{m['id']}: ground move ends on an infinite frame (never returns control)")
    ids = {m["id"] for m in moves}
    seen = {}
    for m in moves:
        key = (m["input"].replace(" ", ""), m.get("when", "stand") if m["kind"] == "normal" else "")
        if key in seen: out["problems"].append(f"{m['id']}: input '{m['input']}' duplicates {seen[key]}")
        seen[key] = m["id"]
        for c in m.get("cancels") or []:
            if c not in ids: out["problems"].append(f"{m['id']}: cancels unknown move '{c}'")
        t = m.get("throw")
        if t and m["anim"] in anims:
            fr = anims[m["anim"]]["frames"]; at = t.get("at", 1)
            if len(fr) < at + 2: out["problems"].append(f"{m['id']}: throw anim needs at least {at + 2} frames")
            elif "hit" not in fr[at - 1] or "box" not in fr[at - 1]["hit"] and fr[at - 1]["hit"].get("radius", 0) < 20: out["problems"].append(f"{m['id']}: throw grab frame {at} needs a hit box (radius >= 20 or box)")
    if not any(m.get("cancels") for m in moves): out["problems"].append("no explicit cancel chains (only default normal>special cancel)")
    if len(sp) < 5: out["problems"].append(f"only {len(sp)} specials")
    if len(su) < 2: out["problems"].append(f"only {len(su)} super(s)")
    if not any(m.get("projectile") and (m["projectile"] if isinstance(m["projectile"], list) else [m["projectile"]])[0].get("hits", 1) > 1 for m in su): out["problems"].append("no multi-hit area super")
    if not any(m.get("throw") for m in moves): out["problems"].append("no throw")
    if not any(m.get("armor") or m.get("parry") for m in moves): out["problems"].append("no armor/parry tool")
    out["maxSuperDamage"] = max([sum(h["damage"] for h in m.get("hits", [])) + sum(p["damage"] * p.get("hits", 1) for p in (m["projectile"] if isinstance(m.get("projectile"), list) else [m["projectile"]] if m.get("projectile") else [])) for m in su] or [0])
    return out
if __name__ == "__main__":
    root = sys.argv[1] if len(sys.argv) > 1 else "assets/chars"
    res = [audit(load(p)) for p in sorted(glob.glob(os.path.join(root, "*.json"))) if not p.endswith("index.json")]
    if "--json" in sys.argv: print(json.dumps(res, indent=1))
    else:
        for r in res:
            print(f"{r['id']:8} n={r['normals']} sp={r['specials']} su={r['supers']} superDmg={r['maxSuperDamage']:.0f} feats={r['features']} abil={r['abilities']}")
            for p in r["problems"]: print("   -", p)
        print("total problems:", sum(len(r["problems"]) for r in res))
