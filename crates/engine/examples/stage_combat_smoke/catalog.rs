#[derive(Clone, Debug)]
pub struct StageSpec {
    pub dir: &'static str,
    pub def: &'static str,
    pub name: &'static str,
    pub origin: &'static str,
    pub repo: &'static str,
}

pub fn all_stages() -> Vec<StageSpec> {
    vec![
        // Chrono Trigger Collection (jbahamon/ctm-stages)
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "ZenanBridge.def", name: "Zenan Bridge", origin: "Chrono Trigger (Middle Ages)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "MagusCastle.def", name: "Magus' Castle", origin: "Chrono Trigger (600 A.D.)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "MagusRoom.def", name: "Magus' Room", origin: "Chrono Trigger (Inner Sanctum)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "Courtroom.def", name: "Court Room", origin: "Chrono Trigger (Guardia Castle)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "GuardiaPrison.def", name: "Guardia Prison", origin: "Chrono Trigger (Dungeon Towers)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "InnerLavos.def", name: "Inner Lavos", origin: "Chrono Trigger (Core of Lavos)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "GenoDome.def", name: "Geno Dome", origin: "Chrono Trigger (2300 A.D. Robot Lab)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "OceanPalaceElevator.def", name: "Ocean Palace Elevator", origin: "Chrono Trigger (12000 B.C.)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "TyrannoLair.def", name: "Tyranno Lair", origin: "Chrono Trigger (65M B.C. Fortress)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "BlackOmenElevator.def", name: "Black Omen Elevator", origin: "Chrono Trigger (Floating Citadel)", repo: "jbahamon/ctm-stages" },
        StageSpec { dir: "/tmp/stage_sources/ctm-stages", def: "Blackbird.def", name: "Blackbird Wing", origin: "Chrono Trigger (Dalton's Battleship)", repo: "jbahamon/ctm-stages" },

        // Mountainside Temple Collection (CableDorado2/Ikemen-Plus-Ultra)
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Suave's Corridor.def", name: "Suave's Corridor", origin: "Mountainside Temple (Grand Hall)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Destroyed Temple.def", name: "Destroyed Temple", origin: "Mountainside Temple (Ruins)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Clone Laboratory.def", name: "Clone Laboratory", origin: "Mountainside Temple (Bio Facility)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Clone Laboratory 2.def", name: "Clone Laboratory 2", origin: "Mountainside Temple (Deep Lab)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Dark Corridor.def", name: "Dark Corridor", origin: "Mountainside Temple (Shadow Corridor)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Entrance Pathway.def", name: "Entrance Pathway", origin: "Mountainside Temple (Stone Walkway)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Hidden Path.def", name: "Hidden Path Day", origin: "Mountainside Temple (Nature Trail)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Hidden Path Night.def", name: "Hidden Path Night", origin: "Mountainside Temple (Moonlit Forest)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Hidden Room.def", name: "Hidden Room", origin: "Mountainside Temple (Secret Chamber)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Lobby.def", name: "Temple Lobby", origin: "Mountainside Temple (Central Atrium)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Lobby Night.def", name: "Temple Lobby Night", origin: "Mountainside Temple (Night Atrium)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Lobby 2 Night.def", name: "Temple Lobby Upper Night", origin: "Mountainside Temple (Mezzanine)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Outside.def", name: "Outside Courtyard", origin: "Mountainside Temple (Mountain Vista)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Roof Night.def", name: "Pagoda Roof Night", origin: "Mountainside Temple (High Pagoda)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Secret Hallway.def", name: "Secret Hallway", origin: "Mountainside Temple (Hidden Passage)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Stairs - 1st Floor.def", name: "Grand Staircase 1F", origin: "Mountainside Temple (Lower Stairs)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Stairs - 2nd Floor.def", name: "Grand Staircase 2F", origin: "Mountainside Temple (Mid Stairs)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Stairs - 3rd Floor.def", name: "Grand Staircase 3F", origin: "Mountainside Temple (Upper Stairs)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Winter.def", name: "Temple Winter Garden", origin: "Mountainside Temple (Snow Zen)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Temple Entrance Afternoon.def", name: "Temple Entrance Afternoon", origin: "Mountainside Temple (Sunlit Gate)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Temple Entrance Dusk.def", name: "Temple Entrance Dusk", origin: "Mountainside Temple (Sunset Gate)", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple", def: "Temple Entrance Night.def", name: "Temple Entrance Night", origin: "Mountainside Temple (Midnight Gate)", repo: "CableDorado2/Ikemen-Plus-Ultra" },

        // Arcade & Global Collection (CableDorado2/Ikemen-Plus-Ultra/stages/Others)
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "China.def", name: "Chinatown Street", origin: "Arcade Fighting Classic", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "Mountain Valley.def", name: "Mountain Valley", origin: "Scenic Nature Vista", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "Sakura.def", name: "Sakura Blossom Garden", origin: "Japanese Cherry Blossom", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "Space Rainbow.def", name: "Cosmic Space Rainbow", origin: "Retro Psychedelic Galaxy", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "Sunday Afternoon.def", name: "Sunday Afternoon Park", origin: "City Park Peaceful Noon", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "The Red Dragon's Lair.def", name: "The Red Dragon's Lair", origin: "Dragon Volcanic Cavern", repo: "CableDorado2/Ikemen-Plus-Ultra" },
        StageSpec { dir: "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others", def: "Training Room.def", name: "Cyber Training Dojo", origin: "Simulation Combat Matrix", repo: "CableDorado2/Ikemen-Plus-Ultra" },

        // REBROT Community Stages (davifelipef/rebrot-fighter)
        StageSpec { dir: "/tmp/stage_sources/rebrot-fighter/stages", def: "nersograd.def", name: "Nersogrado Metro", origin: "REBROT Fighter (Cyber City)", repo: "davifelipef/rebrot-fighter" },
        StageSpec { dir: "/tmp/stage_sources/rebrot-fighter/stages", def: "dark_forest.def", name: "Floresta Sombria", origin: "REBROT Fighter (Dark Woods)", repo: "davifelipef/rebrot-fighter" },
        StageSpec { dir: "/tmp/stage_sources/rebrot-fighter/stages", def: "anarchy_ruins.def", name: "Ruínas da Anarquia", origin: "REBROT Fighter (Ruins)", repo: "davifelipef/rebrot-fighter" },
    ]
}
