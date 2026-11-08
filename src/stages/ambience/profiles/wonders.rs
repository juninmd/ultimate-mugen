use super::super::MoteKind::*;
use super::super::Profile;
use super::*;

static GREAT_WALL: Profile = Profile {
    mist: &[mist(0.50, 0.24, 4.0, 0.2, 0xe8e0f0, 0.14, 6)],
    motes: &[
        motes(Bird, 4, 0.0, 16.0, 6.0, [0.08, 0.3], 0x303038, 0.7),
        motes(Dust, 18, 2.0, 14.0, 1.5, [0.3, 0.95], 0xf0e8d8, 0.4),
    ],
    glows: &[glow(0.622, 0.255, 0.07, 0xffe8a0, 0.35, 0.15, 6.0)],
    beams: &[
        beam(0.622, 0.10, -0.16, 0xfff0c0, 0.10, 12.0),
        beam(0.622, 0.06, 0.14, 0xfff0c0, 0.07, 9.0),
    ],
};

static PETRA: Profile = Profile {
    mist: &[mist(0.82, 0.12, 5.0, 0.6, 0xe0a070, 0.10, 4)],
    motes: &[
        motes(Dust, 34, 3.0, 20.0, 1.6, [0.2, 1.0], 0xf0b880, 0.5),
        motes(Ember, 10, 10.0, 2.0, 1.4, [0.5, 0.75], 0xffa040, 0.8),
    ],
    glows: &[
        glow(0.126, 0.574, 0.05, 0xff9030, 0.40, 0.6, 1.4),
        glow(0.873, 0.574, 0.05, 0xff9030, 0.40, 0.6, 1.7),
        glow(0.50, 0.45, 0.12, 0xffb070, 0.10, 0.2, 6.0),
    ],
    beams: &[beam(0.14, 0.10, 0.10, 0xffe0b0, 0.08, 10.0)],
};

static CHRIST: Profile = Profile {
    mist: &[mist(0.60, 0.28, 4.5, 0.2, 0xe0c8e8, 0.12, 6)],
    motes: &[motes(Bird, 4, 0.0, 14.0, 6.0, [0.15, 0.4], 0x303040, 0.7)],
    glows: &[
        glow(0.47, 0.31, 0.12, 0xffe890, 0.35, 0.15, 8.0),
        glow(0.07, 0.52, 0.05, 0xffd070, 0.30, 0.5, 2.8),
        glow(0.22, 0.57, 0.04, 0xffd070, 0.25, 0.5, 3.4),
        glow(0.36, 0.64, 0.04, 0xffd070, 0.25, 0.5, 3.1),
        glow(0.62, 0.60, 0.04, 0xffd070, 0.25, 0.5, 2.6),
        glow(0.47, 0.85, 0.12, 0xffa090, 0.10, 0.1, 9.0),
    ],
    beams: &[beam(0.47, 0.12, 0.04, 0xfff0c0, 0.09, 11.0)],
};

static TEXAS: Profile = Profile {
    mist: &[
        mist(0.36, 0.12, 3.0, 0.3, 0xffb0b0, 0.08, 4),
        mist(0.85, 0.10, 6.0, 0.6, 0xe8b890, 0.10, 4),
    ],
    motes: &[
        motes(Dust, 30, 1.0, 26.0, 1.7, [0.4, 1.0], 0xe8c8a0, 0.5),
        motes(Bird, 2, 0.0, 12.0, 6.0, [0.08, 0.3], 0x302020, 0.7),
    ],
    glows: &[glow(0.275, 0.292, 0.08, 0xffb040, 0.35, 0.1, 9.0)],
    beams: &[beam(0.275, 0.09, 0.06, 0xffc070, 0.07, 14.0)],
};

static CHICHEN: Profile = Profile {
    mist: &[
        mist(0.72, 0.22, 3.0, 0.3, 0xc0d8c0, 0.10, 5),
        mist(0.34, 0.14, 2.0, 0.2, 0xff8a9a, 0.06, 4),
    ],
    motes: &[
        motes(Leaf, 8, -9.0, 6.0, 4.0, [0.05, 0.9], 0x3a8a40, 0.6),
        motes(Firefly, 10, 3.0, 3.0, 1.5, [0.5, 0.9], 0xc0ff90, 0.8),
    ],
    glows: &[glow(0.74, 0.13, 0.05, 0xf0f0ff, 0.25, 0.05, 10.0)],
    beams: &[beam(0.74, 0.10, -0.12, 0xd8d0ff, 0.07, 12.0)],
};

static COLOSSEUM: Profile = Profile {
    mist: &[mist(0.42, 0.14, 4.0, 0.5, 0xf0d0a8, 0.10, 4)],
    motes: &[
        motes(Dust, 30, 3.0, 12.0, 1.7, [0.55, 1.0], 0xf8d8a0, 0.5),
        motes(Bird, 5, 0.0, 18.0, 6.0, [0.12, 0.35], 0x302828, 0.7),
    ],
    glows: &[
        glow(0.62, 0.36, 0.10, 0xffc068, 0.14, 0.2, 7.0),
        glow(0.50, 0.85, 0.18, 0xffd080, 0.08, 0.05, 10.0),
    ],
    beams: &[beam(0.68, 0.12, -0.08, 0xffe0b0, 0.10, 11.0)],
};

static TAJ: Profile = Profile {
    mist: &[
        mist(0.60, 0.20, 3.0, 0.3, 0xffe8ee, 0.10, 5),
        mist(0.80, 0.14, 3.0, 0.4, 0xffffff, 0.14, 5),
    ],
    motes: &[
        motes(Petal, 14, -7.0, 8.0, 4.0, [0.0, 0.95], 0xffd0e0, 0.7),
        motes(Dust, 16, 2.0, 10.0, 1.5, [0.3, 0.95], 0xffffff, 0.4),
    ],
    glows: &[
        glow(0.68, 0.48, 0.10, 0xffe8c0, 0.30, 0.1, 9.0),
        glow(0.50, 0.78, 0.14, 0xffe8d8, 0.08, 0.1, 9.0),
    ],
    beams: &[
        beam(0.72, 0.10, -0.06, 0xfff4e0, 0.09, 12.0),
        beam(0.66, 0.06, -0.10, 0xfff4e0, 0.07, 9.0),
    ],
};

static GIZA: Profile = Profile {
    mist: &[
        mist(0.55, 0.16, 5.0, 0.3, 0xf0c080, 0.14, 5),
        mist(0.82, 0.12, 6.0, 0.6, 0xf0c080, 0.12, 4),
    ],
    motes: &[motes(Dust, 44, 2.0, 30.0, 1.6, [0.3, 1.0], 0xf0c890, 0.55)],
    glows: &[glow(0.75, 0.34, 0.12, 0xffd860, 0.32, 0.15, 8.0)],
    beams: &[beam(0.75, 0.10, -0.08, 0xffe0a0, 0.09, 12.0)],
};

static HANGING: Profile = Profile {
    mist: &[mist(0.65, 0.24, 3.0, 0.3, 0xe0f4ec, 0.12, 5)],
    motes: &[
        motes(Petal, 16, -6.0, 6.0, 4.0, [0.0, 0.95], 0xffb0d0, 0.7),
        motes(Bird, 3, 0.0, 12.0, 6.0, [0.08, 0.3], 0x203028, 0.6),
        dust(14, 0xfff8d0, 0.4),
    ],
    glows: &[
        glow(0.75, 0.46, 0.04, 0xa0f0ff, 0.15, 0.4, 2.2),
        glow(0.59, 0.67, 0.035, 0xa0f0ff, 0.12, 0.4, 2.6),
        glow(0.36, 0.72, 0.03, 0xa0f0ff, 0.10, 0.3, 3.0),
    ],
    beams: &[
        beam(0.50, 0.12, 0.05, 0xf0ffd8, 0.08, 10.0),
        beam(0.85, 0.08, -0.05, 0xf0ffd8, 0.07, 12.0),
    ],
};

static ARTEMIS: Profile = Profile {
    mist: &[
        mist(0.60, 0.18, 3.0, 0.3, 0x9ccfd8, 0.12, 5),
        fog_front(mist(0.92, 0.10, 5.0, 0.7, 0xa8d0c8, 0.08, 4)),
    ],
    motes: &[
        motes(Dust, 16, 2.0, 4.0, 1.5, [0.1, 0.9], 0xe0f8f0, 0.4),
        motes(Leaf, 8, -9.0, 7.0, 4.0, [0.0, 0.95], 0x78b878, 0.6),
        motes(Petal, 6, -8.0, 4.0, 3.5, [0.0, 0.9], 0xf8fff8, 0.6),
    ],
    glows: &[
        glow(0.266, 0.138, 0.08, 0xe0f0ff, 0.30, 0.05, 9.0),
        glow(0.69, 0.53, 0.05, 0xffd870, 0.20, 0.1, 7.0),
        glow(0.263, 0.69, 0.04, 0xffb050, 0.40, 0.6, 1.2),
        glow(0.335, 0.70, 0.03, 0xffb050, 0.30, 0.6, 1.4),
        glow(0.50, 0.69, 0.04, 0xffb050, 0.40, 0.6, 1.1),
        glow(0.676, 0.69, 0.03, 0xffb050, 0.30, 0.6, 1.5),
        glow(0.742, 0.69, 0.04, 0xffb050, 0.40, 0.6, 1.3),
        glow(0.885, 0.70, 0.03, 0xffb050, 0.30, 0.6, 1.6),
    ],
    beams: &[beam(0.266, 0.10, 0.08, 0xe0f0ff, 0.08, 12.0)],
};

static ZEUS: Profile = Profile {
    mist: &[mist(0.55, 0.25, 2.0, 0.2, 0xffe0b0, 0.08, 4)],
    motes: &[motes(Dust, 18, 1.0, 4.0, 1.5, [0.1, 0.8], 0xfff0d0, 0.4)],
    glows: &[
        glow(0.158, 0.34, 0.04, 0xffb050, 0.35, 0.5, 1.6),
        glow(0.272, 0.43, 0.03, 0xffb050, 0.30, 0.5, 1.9),
        glow(0.323, 0.43, 0.03, 0xffb050, 0.30, 0.5, 1.7),
        glow(0.727, 0.42, 0.03, 0xffb050, 0.30, 0.5, 2.1),
        glow(0.84, 0.35, 0.04, 0xffb050, 0.35, 0.5, 1.5),
        glow(0.50, 0.36, 0.18, 0xffc060, 0.12, 0.15, 6.0),
    ],
    beams: &[],
};

static MAUSOLEUM: Profile = Profile {
    mist: &[
        mist(0.52, 0.12, 4.0, 0.5, 0xf8d8b8, 0.12, 4),
        mist(0.80, 0.12, 4.0, 0.5, 0xf0e0c0, 0.10, 4),
    ],
    motes: &[
        motes(Dust, 26, 2.0, 16.0, 1.6, [0.2, 1.0], 0xf0dcb0, 0.5),
        motes(Bird, 2, 0.0, 14.0, 6.0, [0.08, 0.25], 0x303030, 0.6),
    ],
    glows: &[glow(0.75, 0.42, 0.10, 0xffd8a0, 0.30, 0.1, 8.0)],
    beams: &[beam(0.74, 0.10, -0.10, 0xffe8b8, 0.09, 12.0)],
};

static RHODES: Profile = Profile {
    mist: &[
        mist(0.25, 0.30, 4.0, 0.2, 0x3a7a88, 0.14, 6),
        fog_front(mist(0.75, 0.10, 6.0, 0.6, 0x9ad8e0, 0.08, 4)),
    ],
    motes: &[motes(
        Dust,
        34,
        -40.0,
        -8.0,
        1.0,
        [0.0, 0.85],
        0xb8e8ff,
        0.35,
    )],
    glows: &[
        glow(0.449, 0.064, 0.07, 0xff7040, 0.40, 0.3, 2.0),
        glow(0.80, 0.38, 0.08, 0xc8a8ff, 0.25, 0.9, 4.5),
        glow(0.50, 0.62, 0.08, 0xff9060, 0.12, 0.4, 2.5),
    ],
    beams: &[],
};

static PHAROS: Profile = Profile {
    mist: &[
        mist(0.55, 0.12, 4.0, 0.3, 0x9ab8d0, 0.10, 5),
        mist(0.72, 0.10, 6.0, 0.5, 0x6a8ab0, 0.08, 4),
    ],
    motes: &[motes(Dust, 30, -2.0, 0.0, 1.0, [0.0, 0.5], 0xf0f8ff, 0.35)],
    glows: &[
        glow(0.347, 0.138, 0.07, 0xffe8a0, 0.55, 0.15, 2.4),
        glow(0.206, 0.154, 0.05, 0xd8e0f0, 0.15, 0.05, 10.0),
        glow(0.80, 0.72, 0.05, 0xc8f0ff, 0.10, 0.5, 2.8),
    ],
    beams: &[],
};

pub(super) fn profile(id: &str) -> Option<&'static Profile> {
    Some(match id {
        "great_wall" => &GREAT_WALL,
        "petra" => &PETRA,
        "christ_redeemer" => &CHRIST,
        "texas" => &TEXAS,
        "chichen_itza" => &CHICHEN,
        "colosseum" => &COLOSSEUM,
        "taj_mahal" => &TAJ,
        "pyramids_giza" => &GIZA,
        "hanging_gardens" => &HANGING,
        "temple_artemis" => &ARTEMIS,
        "statue_zeus" => &ZEUS,
        "mausoleum_halicarnassus" => &MAUSOLEUM,
        "colossus_rhodes" => &RHODES,
        "lighthouse_alexandria" => &PHAROS,
        _ => return None,
    })
}
