use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct StageSpec {
    pub subpath: &'static str,
    pub def: &'static str,
    pub name: &'static str,
    pub origin: &'static str,
    pub repo: &'static str,
}

impl StageSpec {
    pub fn resolved_dir(&self) -> PathBuf {
        if let Ok(env_dir) = std::env::var("UME_STAGES_DIR") {
            let p = Path::new(&env_dir).join(self.subpath);
            if p.exists() { return p; }
        }
        let mugen_dir = Path::new("mugen/stages").join(self.subpath);
        if mugen_dir.exists() { return mugen_dir; }
        let tmp_dir = Path::new("/tmp/stage_sources").join(self.subpath);
        if tmp_dir.exists() { return tmp_dir; }
        mugen_dir
    }
}

const fn s(subpath: &'static str, def: &'static str, name: &'static str, origin: &'static str, repo: &'static str) -> StageSpec {
    StageSpec { subpath, def, name, origin, repo }
}

#[rustfmt::skip]
pub fn all_stages() -> Vec<StageSpec> {
    vec![
        // Chrono Trigger Collection (jbahamon/ctm-stages)
        s("ctm-stages", "ZenanBridge.def", "Zenan Bridge", "Chrono Trigger (Middle Ages)", "jbahamon/ctm-stages"),
        s("ctm-stages", "MagusCastle.def", "Magus' Castle", "Chrono Trigger (600 A.D.)", "jbahamon/ctm-stages"),
        s("ctm-stages", "MagusRoom.def", "Magus' Room", "Chrono Trigger (Inner Sanctum)", "jbahamon/ctm-stages"),
        s("ctm-stages", "Courtroom.def", "Court Room", "Chrono Trigger (Guardia Castle)", "jbahamon/ctm-stages"),
        s("ctm-stages", "GuardiaPrison.def", "Guardia Prison", "Chrono Trigger (Dungeon Towers)", "jbahamon/ctm-stages"),
        s("ctm-stages", "InnerLavos.def", "Inner Lavos", "Chrono Trigger (Core of Lavos)", "jbahamon/ctm-stages"),
        s("ctm-stages", "GenoDome.def", "Geno Dome", "Chrono Trigger (2300 A.D. Robot Lab)", "jbahamon/ctm-stages"),
        s("ctm-stages", "OceanPalaceElevator.def", "Ocean Palace Elevator", "Chrono Trigger (12000 B.C.)", "jbahamon/ctm-stages"),
        s("ctm-stages", "TyrannoLair.def", "Tyranno Lair", "Chrono Trigger (65M B.C. Fortress)", "jbahamon/ctm-stages"),
        s("ctm-stages", "BlackOmenElevator.def", "Black Omen Elevator", "Chrono Trigger (Floating Citadel)", "jbahamon/ctm-stages"),
        s("ctm-stages", "Blackbird.def", "Blackbird Wing", "Chrono Trigger (Dalton's Battleship)", "jbahamon/ctm-stages"),

        // Mountainside Temple Collection (CableDorado2/Ikemen-Plus-Ultra)
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Suave's Corridor.def", "Suave's Corridor", "Mountainside Temple (Grand Hall)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Destroyed Temple.def", "Destroyed Temple", "Mountainside Temple (Ruins)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Clone Laboratory.def", "Clone Laboratory", "Mountainside Temple (Bio Facility)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Clone Laboratory 2.def", "Clone Laboratory 2", "Mountainside Temple (Deep Lab)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Dark Corridor.def", "Dark Corridor", "Mountainside Temple (Shadow Corridor)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Entrance Pathway.def", "Entrance Pathway", "Mountainside Temple (Stone Walkway)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Hidden Path.def", "Hidden Path Day", "Mountainside Temple (Nature Trail)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Hidden Path Night.def", "Hidden Path Night", "Mountainside Temple (Moonlit Forest)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Hidden Room.def", "Hidden Room", "Mountainside Temple (Secret Chamber)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Lobby.def", "Temple Lobby", "Mountainside Temple (Central Atrium)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Lobby Night.def", "Temple Lobby Night", "Mountainside Temple (Night Atrium)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Lobby 2 Night.def", "Temple Lobby Upper Night", "Mountainside Temple (Mezzanine)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Outside.def", "Outside Courtyard", "Mountainside Temple (Mountain Vista)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Roof Night.def", "Pagoda Roof Night", "Mountainside Temple (High Pagoda)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Secret Hallway.def", "Secret Hallway", "Mountainside Temple (Hidden Passage)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Stairs - 1st Floor.def", "Grand Staircase 1F", "Mountainside Temple (Lower Stairs)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Stairs - 2nd Floor.def", "Grand Staircase 2F", "Mountainside Temple (Mid Stairs)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Stairs - 3rd Floor.def", "Grand Staircase 3F", "Mountainside Temple (Upper Stairs)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Winter.def", "Temple Winter Garden", "Mountainside Temple (Snow Zen)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Temple Entrance Afternoon.def", "Temple Entrance Afternoon", "Mountainside Temple (Sunlit Gate)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Temple Entrance Dusk.def", "Temple Entrance Dusk", "Mountainside Temple (Sunset Gate)", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Mountainside Temple", "Temple Entrance Night.def", "Temple Entrance Night", "Mountainside Temple (Midnight Gate)", "CableDorado2/Ikemen-Plus-Ultra"),

        // Arcade & Global Collection (CableDorado2/Ikemen-Plus-Ultra/stages/Others)
        s("Ikemen-Plus-Ultra/stages/Others", "China.def", "Chinatown Street", "Arcade Fighting Classic", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Others", "Mountain Valley.def", "Mountain Valley", "Scenic Nature Vista", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Others", "Sakura.def", "Sakura Blossom Garden", "Japanese Cherry Blossom", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Others", "Space Rainbow.def", "Cosmic Space Rainbow", "Retro Psychedelic Galaxy", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Others", "Sunday Afternoon.def", "Sunday Afternoon Park", "City Park Peaceful Noon", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Others", "The Red Dragon's Lair.def", "The Red Dragon's Lair", "Dragon Volcanic Cavern", "CableDorado2/Ikemen-Plus-Ultra"),
        s("Ikemen-Plus-Ultra/stages/Others", "Training Room.def", "Cyber Training Dojo", "Simulation Combat Matrix", "CableDorado2/Ikemen-Plus-Ultra"),

        // REBROT Community Stages (davifelipef/rebrot-fighter)
        s("rebrot-fighter/stages", "nersograd.def", "Nersogrado Metro", "REBROT Fighter (Cyber City)", "davifelipef/rebrot-fighter"),
        s("rebrot-fighter/stages", "dark_forest.def", "Floresta Sombria", "REBROT Fighter (Dark Woods)", "davifelipef/rebrot-fighter"),
        s("rebrot-fighter/stages", "anarchy_ruins.def", "Ruínas da Anarquia", "REBROT Fighter (Ruins)", "davifelipef/rebrot-fighter"),
    ]
}
