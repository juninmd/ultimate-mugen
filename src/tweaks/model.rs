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
    pub p1_tint: [u8; 3],
    pub p2_tint: [u8; 3],
    pub bar_offset_y: f32,
    pub hud_opacity: f32,
    pub colorblind: bool,
    pub reduce_motion: bool,
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
            p1_tint: [0x4d, 0xb8, 0xff],
            p2_tint: [0xff, 0x61, 0x85],
            bar_offset_y: 0.0,
            hud_opacity: 1.0,
            colorblind: false,
            reduce_motion: false,
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
