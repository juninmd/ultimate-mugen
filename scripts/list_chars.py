import json, glob

files = sorted(glob.glob("assets/chars/*.json"))
for f in files:
    if "index.json" in f:
        continue
    with open(f, encoding="utf-8") as fp:
        d = json.load(fp)
    cid = d.get("id")
    name = d.get("name")
    series = d.get("series")
    desc = d.get("description")
    style = d.get("art", {}).get("style", {})
    colors = style.get("colors", {})
    hair = style.get("hair", {})
    top = style.get("top")
    pants = style.get("pants")
    print(f"[{cid}] {name} ({series})")
    print(f"  Desc: {desc}")
    print(f"  Visuals: top={top}, pants={pants}, hair={hair.get('type')}, colors={colors}")
