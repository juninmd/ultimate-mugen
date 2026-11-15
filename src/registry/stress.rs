use super::*;

const SYLLABLES: [&str; 24] = [
    "ka", "ri", "zen", "mo", "tal", "vor", "shi", "ra", "dun", "el", "gar", "yu", "ix", "no",
    "bel", "tor", "sa", "qu", "fen", "lo", "dra", "mi", "kor", "ve",
];
const AUTHORS: [&str; 12] = [
    "Elecbyte",
    "Warusaki3",
    "Kong",
    "Vyn",
    "Ristar87",
    "Infinite",
    "Jmorphman",
    "Phantom.of.the.Server",
    "Cybaster",
    "OrochiKOF",
    "Balthazar",
    "PotS",
];
const SERIES: [&str; 8] = [
    "Street Brawl",
    "Neo Legends",
    "Shonen Clash",
    "Retro Kings",
    "Mecha Zone",
    "Dark Saga",
    "Sky Fist",
    "Original",
];
const PLACES: [&str; 16] = [
    "Harbor",
    "Temple",
    "Rooftop",
    "Arena",
    "Forest",
    "Desert",
    "Station",
    "Palace",
    "Canyon",
    "Market",
    "Volcano",
    "Glacier",
    "Lab",
    "Bridge",
    "Shrine",
    "Colosseum",
];
const MOODS: [&str; 8] = [
    "Night", "Sunset", "Storm", "Dawn", "Rain", "Snow", "Neon", "Ruins",
];

fn mix(n: usize) -> u64 {
    let mut x = n as u64 ^ 0x9e37_79b9_7f4a_7c15;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn fighter_name(n: usize) -> String {
    let h = mix(n);
    let parts = 2 + h % 2;
    let mut name: String = (0..parts)
        .map(|k| SYLLABLES[((h >> (k * 5)) % SYLLABLES.len() as u64) as usize])
        .collect();
    if let Some(first) = name.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    format!("{name} {n:04}")
}

pub fn stress_count() -> usize {
    #[cfg(target_arch = "wasm32")]
    let raw = crate::query_params()
        .into_iter()
        .find(|(k, _)| k == "stress")
        .map(|(_, v)| v);
    #[cfg(not(target_arch = "wasm32"))]
    let raw = std::env::var("UME_STRESS").ok();
    raw.and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .min(5000)
}

impl CharEntry {
    fn stress_clone(&self, n: usize) -> CharEntry {
        let h = mix(n);
        CharEntry {
            key: format!("stress:{n}"),
            id: format!("stress:{n}"),
            name: fighter_name(n),
            series: SERIES[(h % SERIES.len() as u64) as usize].to_string(),
            author: AUTHORS[((h >> 8) % AUTHORS.len() as u64) as usize].to_string(),
            color: [
                (h >> 16) as u8 | 0x40,
                (h >> 24) as u8 | 0x40,
                (h >> 32) as u8 | 0x40,
            ],
            description: self.description.clone(),
            mugen: self.mugen,
            boss: false,
            src: match &self.src {
                Src::Native(nc) => Src::Native(nc.clone()),
                Src::Mugen { vfs, def } => Src::Mugen {
                    vfs: vfs.clone(),
                    def: def.clone(),
                },
            },
            cache: RefCell::new(HashMap::new()),
            preview: RefCell::new(HashMap::new()),
            last_use: Cell::new(0),
            portrait: RefCell::new(None),
            broken: Cell::new(false),
            errs: RefCell::new(HashMap::new()),
            jobs: Default::default(),
            sprite_revision: Cell::new(0),
        }
    }
}

impl StageEntry {
    fn stress_clone(&self, n: usize) -> StageEntry {
        let h = mix(n + 7919);
        StageEntry {
            key: format!("stress-stage:{n}"),
            id: format!("stress-stage:{n}"),
            name: format!(
                "{} {} {n:04}",
                PLACES[(h % PLACES.len() as u64) as usize],
                MOODS[((h >> 6) % MOODS.len() as u64) as usize]
            ),
            thumb: self.thumb,
            mugen: self.mugen,
            src: match &self.src {
                StageSrc::Builtin(i) => StageSrc::Builtin(*i),
                StageSrc::Mugen { vfs, def } => StageSrc::Mugen {
                    vfs: vfs.clone(),
                    def: def.clone(),
                },
            },
            preview: RefCell::new(Preview::Unknown),
            stage: RefCell::new(None),
        }
    }
}

impl Registry {
    pub fn add_stress_roster(&mut self) {
        let count = stress_count();
        if count == 0 {
            return;
        }
        let fighters: Vec<usize> = (0..self.chars.len())
            .filter(|&i| !self.chars[i].boss)
            .collect();
        if !fighters.is_empty() {
            let extra: Vec<CharEntry> = (0..count)
                .map(|n| self.chars[fighters[n % fighters.len()]].stress_clone(n))
                .collect();
            self.chars.extend(extra);
        }
        if !self.stages.is_empty() {
            let extra: Vec<StageEntry> = (0..count)
                .map(|n| self.stages[n % self.stages.len()].stress_clone(n))
                .collect();
            self.stages.extend(extra);
        }
    }
}
