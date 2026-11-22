from .abilities import detect_wall_bounce
from .parse import nums

STANDARD = {
    0: "idle", 5: "turn", 6: "crouchTurn", 10: "crouchStart", 11: "crouch", 12: "crouchEnd", 20: "walk", 21: "walkBack",
    40: "jumpStart", 41: "jumpUp", 42: "jumpFwd", 43: "jumpBack", 44: "jumpDown", 47: "land", 100: "run", 105: "hop",
    120: "guardStart", 121: "crouchGuardStart", 122: "airGuardStart", 130: "guard", 131: "crouchGuard", 132: "airGuard",
    140: "guardEnd", 150: "guardHit", 151: "airGuardHit", 152: "crouchGuardHit", 170: "lose", 175: "draw", 181: "win",
    190: "intro", 195: "taunt", 5000: "hitLight", 5001: "hitMedium", 5002: "hitHeavy", 5010: "hitLowLight",
    5011: "hitLowMedium", 5012: "hitLowHeavy", 5020: "hitCrouch", 5030: "hitAir", 5040: "airRecover", 5050: "fall",
    5070: "trip", 5100: "bounce", 5101: "bounceAir", 5110: "lieDown", 5120: "getUp", 5150: "dead",
}
LEVEL = {"light": "light", "medium": "medium", "med": "medium", "hard": "heavy", "heavy": "heavy", "back": "heavy", "up": "heavy", "diagup": "heavy"}


def anim_name(number):
    return f"seq{number - 900000}" if number >= 900000 else STANDARD.get(number, f"a{number}")


def pair(text, default=(0.0, 0.0)):
    values = nums(text, default)
    return [round(values[0], 2), round(values[1], 2)]


def hit_from(ctrl, at, states=None):
    b = ctrl["body"]
    damage = nums(b.get("damage"), (0, 0))
    hit = {"at": at, "damage": max(round(damage[0]), 1)}
    if len(damage) > 1 and damage[1]:
        hit["guardDamage"] = round(damage[1])
    level = LEVEL.get(b.get("animtype", "light").strip().lower())
    if level:
        hit["level"] = level
    flag = b.get("guardflag", "MA").upper()
    if "L" in flag and "H" not in flag:
        hit["guard"] = "low"
    elif "H" in flag and "L" not in flag:
        hit["guard"] = "high"
    elif not flag.strip():
        hit["guard"] = "unblockable"
    if b.get("pausetime"):
        p = nums(b["pausetime"], (0, 0))
        hit["pausetime"] = [int(p[0]), int(p[1])]
    if b.get("ground.hittime"):
        hit["hitstun"] = int(nums(b["ground.hittime"], (12,))[0])
    if b.get("ground.velocity"):
        hit["knockback"] = pair(b["ground.velocity"])
    if b.get("air.velocity"):
        hit["airKnockback"] = pair(b["air.velocity"])
    if b.get("fall", "0").strip().startswith("1") or b.get("ground.type", "").strip().lower() == "trip":
        hit["launch"] = True
    if b.get("fall.recover", "1").strip().startswith("0"):
        hit["recover"] = False
    hit.update(hit_sound(b))
    if b.get("hitflag"):
        hit["hitflag"] = b["hitflag"].strip().upper()
    if states and detect_wall_bounce(ctrl, states):
        hit["wallBounce"] = True
    return hit


def hit_sound(b):
    sound = b.get("hitsound", "").strip()
    if not sound:
        return {}
    s = nums(sound, (5, 0))
    own = sound.upper().startswith("S")
    return {"hitSound": [int(s[0]), int(s[1])], **({"ownSound": True} if own else {})}


def projectile_from(ctrl, at, actions):
    b = ctrl["body"]
    dmg = nums(b.get("damage"), (30, 0))
    proj = {"at": at, "anim": anim_name(int(nums(b.get("projanim"), (0,))[0])), "speed": pair(b.get("velocity"), (4, 0)),
            "offset": pair(b.get("offset")), "damage": max(round(dmg[0]), 1)}
    if len(dmg) > 1 and dmg[1]:
        proj["guardDamage"] = round(dmg[1])
    hit_anim = int(nums(b.get("projhitanim"), (-1,))[0])
    if hit_anim in actions:
        proj["hitAnim"] = anim_name(hit_anim)
    for key, field in (("projhits", "hits"), ("projmisstime", "missTime"), ("projpriority", "priority"), ("projremovetime", "removeTime")):
        if b.get(key):
            proj[field] = int(nums(b[key], (1,))[0])
    if b.get("accel"):
        proj["accel"] = pair(b["accel"])
    if b.get("ground.velocity"):
        proj["knockback"] = pair(b["ground.velocity"])
    if b.get("fall", "0").strip().startswith("1"):
        proj["launch"] = True
    if b.get("fall.recover", "1").strip().startswith("0"):
        proj["recover"] = False
    if proj.get("hits", 1) > 1 and "missTime" not in proj:
        proj["missTime"] = 0
    if b.get("pausetime"):
        p = nums(b["pausetime"], (0, 0))
        proj["pausetime"] = [int(p[0]), int(p[1])]
    if b.get("hitflag"):
        proj["hitflag"] = b["hitflag"].strip().upper()
    proj.update(hit_sound(b))
    level = LEVEL.get(b.get("animtype", "").strip().lower())
    if level:
        proj["level"] = level
    return proj, [int(nums(b.get("projanim"), (0,))[0])] + ([hit_anim] if hit_anim in actions else [])
