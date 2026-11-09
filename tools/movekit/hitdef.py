from .controller import controller, pair
from .kinds import AIR_KINDS, HIT_ATTRIBUTE, kind_defaults


def default_attribute(move, kind):
    if kind in AIR_KINDS:
        stance = "A"
    elif kind == "crouch" or move.get("type") == "C":
        stance = "C"
    else:
        stance = "S"
    return f"{stance}, {HIT_ATTRIBUTE[kind]}"


def hit_controllers(hit, element, move, defaults, next_element=None):
    kind = move.get("kind", "normal")
    base = kind_defaults(kind)
    fall = hit.get("fall", base["fall"])
    fields = dict(
        trigger1=f"animelem = {element}",
        attr=hit.get("attr", default_attribute(move, kind)),
        animtype=hit.get("animtype", "Medium"),
        damage=pair(hit.get("damage", base["damage"])),
        guardflag=hit.get("guardflag", base.get("guardflag", "MA")),
        pausetime=pair(hit.get("pausetime", base["pausetime"])),
        hitsound=hit.get("hitsound", defaults.get("hitsound", "S5,0")),
        guardsound=hit.get("guardsound", defaults.get("guardsound", "S6,0")),
        sparkno=hit.get("sparkno", defaults.get("sparkno", -1)),
        guard__sparkno=hit.get("guard_sparkno", defaults.get("guard_sparkno", -1)),
        sparkxy=pair(hit.get("sparkxy", [0, -30])),
        ground__type=hit.get("ground_type", base.get("ground_type", "High")),
        ground__slidetime=hit.get("slidetime", 12),
        ground__hittime=hit.get("hittime", 15),
        ground__velocity=pair(hit.get("ground_velocity", base["ground_velocity"])),
        air__velocity=pair(hit.get("air_velocity", [-2.5, -3])),
        fall=int(bool(fall)),
    )
    if fall:
        fields["fall__recover"] = int(bool(hit.get("fall_recover", True)))
    for key, value in hit.get("extra", {}).items():
        fields[key.replace(".", "__")] = value
    return controller("HitDef", f"hit elem {element}", **fields)


def hit_elements(hit, hit_frame_numbers):
    if "elems" in hit:
        return hit["elems"]
    if "elem" in hit:
        return [hit["elem"]]
    return hit_frame_numbers[:1]
