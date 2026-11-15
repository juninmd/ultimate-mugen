use super::model::Tweaks;

type Rgb = [u8; 3];

struct Preset {
    name: &'static str,
    background: Rgb,
    glows: [Rgb; 3],
    accent: Rgb,
    title: Rgb,
    info: Rgb,
    muted: Rgb,
    text: Rgb,
    life: Rgb,
    sides: [Rgb; 2],
}

const PRESETS: [Preset; 3] = [
    Preset {
        name: "emerald",
        background: [0x04, 0x12, 0x10],
        glows: [[0x1f, 0xd6, 0x8a], [0x2a, 0xb8, 0xff], [0xd9, 0xf2, 0x6b]],
        accent: [0x14, 0xb8, 0x7a],
        title: [0xd9, 0xf2, 0x6b],
        info: [0x6b, 0xe8, 0xd0],
        muted: [0x8f, 0xb5, 0xa6],
        text: [0xd6, 0xf0, 0xe4],
        life: [0x2f, 0xc9, 0x6b],
        sides: [[0x4d, 0xd6, 0xff], [0xd9, 0xf2, 0x6b]],
    },
    Preset {
        name: "sunset",
        background: [0x1a, 0x08, 0x06],
        glows: [[0xff, 0x7a, 0x2e], [0xff, 0xc2, 0x4d], [0xff, 0x4d, 0x8a]],
        accent: [0xf0, 0x6a, 0x1e],
        title: [0xff, 0xe0, 0x8a],
        info: [0xff, 0xa8, 0x6b],
        muted: [0xc4, 0xa0, 0x94],
        text: [0xf5, 0xdc, 0xd0],
        life: [0xf2, 0x8a, 0x1e],
        sides: [[0xff, 0xc2, 0x4d], [0xff, 0x4d, 0x8a]],
    },
    Preset {
        name: "mono",
        background: [0x0a, 0x0a, 0x0c],
        glows: [[0x8a, 0x8a, 0x96], [0x5a, 0x5a, 0x66], [0xc8, 0xc8, 0xd2]],
        accent: [0x6a, 0x6a, 0x78],
        title: [0xf2, 0xf2, 0xf5],
        info: [0xb8, 0xb8, 0xc4],
        muted: [0x8a, 0x8a, 0x96],
        text: [0xdc, 0xdc, 0xe4],
        life: [0xd8, 0xd8, 0xe0],
        sides: [[0xf2, 0xf2, 0xf5], [0x8a, 0x8a, 0x96]],
    },
];

pub fn names() -> Vec<&'static str> {
    std::iter::once("default")
        .chain(PRESETS.iter().map(|preset| preset.name))
        .collect()
}

pub fn apply(name: &str, tweaks: &mut Tweaks) -> Result<(), String> {
    if name == "default" {
        let rules = tweaks.rules;
        *tweaks = Tweaks {
            rules,
            ..Tweaks::default()
        };
        return Ok(());
    }
    let preset = PRESETS
        .iter()
        .find(|preset| preset.name == name)
        .ok_or_else(|| names().join(", "))?;
    tweaks.background = preset.background;
    [tweaks.glow_bottom, tweaks.glow_left, tweaks.glow_right] = preset.glows;
    tweaks.accent = preset.accent;
    tweaks.title = preset.title;
    tweaks.info = preset.info;
    tweaks.muted = preset.muted;
    tweaks.text = preset.text;
    tweaks.life = preset.life;
    [tweaks.p1_tint, tweaks.p2_tint] = preset.sides;
    Ok(())
}
