pub struct StageDef {
    pub arena: &'static str,
    pub waves: &'static [&'static [&'static str]],
    pub boss: &'static str,
    pub alt_boss: &'static str,
    pub accent: [f32; 3],
    pub lane_far: f32,
    pub lane_near: f32,
}

pub const SEGMENT_WORLD: f32 = 640.0;
pub const FIGHT_HALF_RANGE: f32 = 100.0;
pub const VILLAIN_BASE_LIFE_RATIO: f32 = 0.10;

pub fn villain_life_ratio(id: &str) -> f32 {
    match id.strip_prefix("umf:").unwrap_or(id) {
        "npc_razor" | "npc_phantom" => 0.09,
        "npc_spitfire" => 0.10,
        "npc_brute" => 0.11,
        "npc_colossus" => 0.12,
        _ => VILLAIN_BASE_LIFE_RATIO,
    }
}

pub fn fights_in(def: &StageDef) -> usize {
    def.waves.len() + 1
}

pub fn strip_span(def: &StageDef) -> f32 {
    SEGMENT_WORLD * fights_in(def) as f32
}

pub fn fight_center(def: &StageDef, fight: usize) -> f32 {
    let last = fights_in(def) - 1;
    (fight.min(last) as f32 - last as f32 / 2.0) * SEGMENT_WORLD
}

pub const STAGES: [StageDef; 5] = [
    StageDef {
        arena: "brazil",
        waves: &[
            &["npc_brute", "npc_brute"],
            &["npc_razor", "npc_brute"],
            &["npc_razor", "npc_razor", "npc_brute"],
            &["npc_razor", "npc_brute", "npc_spitfire"],
        ],
        boss: "bruno",
        alt_boss: "daichi",
        accent: [1.0, 0.72, 0.2],
        lane_far: -26.0,
        lane_near: 8.0,
    },
    StageDef {
        arena: "neon",
        waves: &[
            &["npc_razor", "npc_spitfire"],
            &["npc_brute", "npc_spitfire", "npc_razor"],
            &["npc_phantom", "npc_razor"],
            &["npc_brute", "npc_brute", "npc_spitfire"],
        ],
        boss: "marlow",
        alt_boss: "kaname",
        accent: [0.4, 0.9, 1.0],
        lane_far: -22.0,
        lane_near: 8.0,
    },
    StageDef {
        arena: "bamboo",
        waves: &[
            &["npc_phantom", "npc_phantom"],
            &["npc_spitfire", "npc_brute", "npc_phantom"],
            &["npc_colossus"],
            &["npc_razor", "npc_phantom", "npc_spitfire"],
        ],
        boss: "lian",
        alt_boss: "shion",
        accent: [0.45, 1.0, 0.6],
        lane_far: -20.0,
        lane_near: 8.0,
    },
    StageDef {
        arena: "volcano",
        waves: &[
            &["npc_colossus", "npc_spitfire"],
            &["npc_phantom", "npc_razor", "npc_brute"],
            &["npc_colossus", "npc_phantom"],
            &["npc_brute", "npc_colossus", "npc_razor"],
        ],
        boss: "garan",
        alt_boss: "haruki",
        accent: [1.0, 0.4, 0.25],
        lane_far: -20.0,
        lane_near: 8.0,
    },
    StageDef {
        arena: "dojo",
        waves: &[
            &["npc_brute", "npc_razor", "npc_spitfire"],
            &["npc_colossus", "npc_phantom", "npc_razor"],
            &["npc_colossus", "npc_brute", "npc_spitfire", "npc_phantom"],
            &["npc_phantom", "npc_phantom", "npc_razor", "npc_spitfire"],
        ],
        boss: "valdric",
        alt_boss: "kaito",
        accent: [0.85, 0.4, 1.0],
        lane_far: -16.0,
        lane_near: 8.0,
    },
];
