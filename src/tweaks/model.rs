#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct RuleOverrides {
    pub rounds_to_win: Option<i32>,
    pub round_seconds: Option<u32>,
    pub life_percent: Option<u32>,
    pub game_speed: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tweaks {
    pub background: [u8; 3],
    pub glow_bottom: [u8; 3],
    pub glow_left: [u8; 3],
    pub glow_right: [u8; 3],
    pub accent: [u8; 3],
    pub title: [u8; 3],
    pub info: [u8; 3],
    pub muted: [u8; 3],
    pub text: [u8; 3],
    pub life: [u8; 3],
    pub life_low: [u8; 3],
    pub damage: [u8; 3],
    pub p1_tint: [u8; 3],
    pub p2_tint: [u8; 3],
    pub bar_offset_y: f32,
    pub hud_opacity: f32,
    pub hud_scale: f32,
    pub colorblind: bool,
    pub reduce_motion: bool,
    pub title_crt: bool,
    pub touch_scale: f32,
    pub touch_opacity: f32,
    pub touch_left_handed: bool,
    pub touch_floating_stick: bool,
    pub rules: RuleOverrides,
}

pub type Labels = Vec<(String, String)>;

impl Default for Tweaks {
    fn default() -> Self {
        Tweaks {
            background: [0x0b, 0x07, 0x16],
            glow_bottom: [0xff, 0x4d, 0x6e],
            glow_left: [0x21, 0xe6, 0xff],
            glow_right: [0xff, 0xcc, 0x33],
            accent: [0xff, 0x4d, 0x6d],
            title: [0xff, 0xcc, 0x33],
            info: [0x22, 0xe8, 0xff],
            muted: [0xa4, 0x9c, 0xc0],
            text: [0xd8, 0xd0, 0xf0],
            life: [0xf0, 0xa8, 0x12],
            life_low: [0xe0, 0x50, 0x1a],
            damage: [0xbf, 0x1a, 0x33],
            p1_tint: [0x4d, 0xb8, 0xff],
            p2_tint: [0xff, 0x61, 0x85],
            bar_offset_y: 0.0,
            hud_opacity: 1.0,
            hud_scale: 1.0,
            colorblind: false,
            reduce_motion: false,
            title_crt: true,
            touch_scale: 1.0,
            touch_opacity: 1.0,
            touch_left_handed: false,
            touch_floating_stick: true,
            rules: RuleOverrides::default(),
        }
    }
}

impl Tweaks {
    pub fn tint(&self, side: usize) -> [f32; 3] {
        let c = if side == 0 { self.p1_tint } else { self.p2_tint };
        [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
    }
}
