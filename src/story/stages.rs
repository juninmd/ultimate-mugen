pub struct StageDef {
    pub arena: &'static str,
    pub waves: &'static [&'static [&'static str]],
    pub boss: &'static str,
    pub alt_boss: &'static str,
    pub accent: [f32; 3],
}

pub const STAGES: [StageDef; 5] = [
    StageDef {
        arena: "brazil",
        waves: &[
            &["npc_brute", "npc_brute"],
            &["npc_razor", "npc_brute"],
            &["npc_razor", "npc_razor", "npc_brute"],
        ],
        boss: "bruno",
        alt_boss: "daichi",
        accent: [1.0, 0.72, 0.2],
    },
    StageDef {
        arena: "neon",
        waves: &[
            &["npc_razor", "npc_spitfire"],
            &["npc_brute", "npc_spitfire", "npc_razor"],
            &["npc_phantom", "npc_razor"],
        ],
        boss: "marlow",
        alt_boss: "kaname",
        accent: [0.4, 0.9, 1.0],
    },
    StageDef {
        arena: "bamboo",
        waves: &[
            &["npc_phantom", "npc_phantom"],
            &["npc_spitfire", "npc_brute", "npc_phantom"],
            &["npc_colossus"],
        ],
        boss: "lian",
        alt_boss: "shion",
        accent: [0.45, 1.0, 0.6],
    },
    StageDef {
        arena: "volcano",
        waves: &[
            &["npc_colossus", "npc_spitfire"],
            &["npc_phantom", "npc_razor", "npc_brute"],
            &["npc_colossus", "npc_phantom"],
        ],
        boss: "garan",
        alt_boss: "haruki",
        accent: [1.0, 0.4, 0.25],
    },
    StageDef {
        arena: "dojo",
        waves: &[
            &["npc_brute", "npc_razor", "npc_spitfire"],
            &["npc_colossus", "npc_phantom", "npc_razor"],
            &["npc_colossus", "npc_brute", "npc_spitfire", "npc_phantom"],
        ],
        boss: "valdric",
        alt_boss: "kaito",
        accent: [0.85, 0.4, 1.0],
    },
];
