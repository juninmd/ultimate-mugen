KINDS = ("normal", "crouch", "air", "special", "air_special", "super", "projectile", "throw")

AIR_KINDS = ("air", "air_special")

HIT_ATTRIBUTE = {
    "normal": "NA", "crouch": "NA", "air": "NA", "special": "SA",
    "air_special": "SA", "super": "HA", "projectile": "SP", "throw": "NT",
}

KIND_DEFAULTS = {
    "normal": {"damage": [50, 6], "pausetime": [8, 9], "ground_velocity": [-3.5, 0], "fall": False, "power": 0},
    "crouch": {"damage": [45, 5], "pausetime": [8, 9], "ground_velocity": [-3, 0], "fall": False, "power": 0, "ground_type": "Low", "guardflag": "L"},
    "air": {"damage": [55, 6], "pausetime": [8, 9], "ground_velocity": [-3, 0], "fall": False, "power": 0},
    "special": {"damage": [100, 15], "pausetime": [10, 12], "ground_velocity": [-5, 0], "fall": True, "power": 1000},
    "air_special": {"damage": [100, 15], "pausetime": [10, 12], "ground_velocity": [-4.5, 0], "fall": True, "power": 1000},
    "super": {"damage": [280, 40], "pausetime": [14, 16], "ground_velocity": [-6.5, -2.5], "fall": True, "power": 3000},
    "projectile": {"damage": [70, 10], "pausetime": [6, 8], "ground_velocity": [-4, 0], "fall": False, "power": 1000},
    "throw": {"damage": [90, 0], "pausetime": [8, 8], "ground_velocity": [-4, -4], "fall": True, "power": 0, "guardflag": ""},
}


def kind_defaults(kind):
    if kind not in KIND_DEFAULTS:
        raise ValueError(f"kind must be one of {KINDS}, got {kind!r}")
    return KIND_DEFAULTS[kind]
