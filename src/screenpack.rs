//! Screenpack support: load `system.def` / `select.def` / `fight.def` from a M.U.G.E.N pack and expose the
//! pieces the native UI can honour (menu labels, roster order, storyboard text and lifebar geometry).
use std::path::{Path, PathBuf};
use ume_formats::screenpack::{load_screenpack, Bar, Screenpack, Storyboard};
use ume_formats::vfs::{add_dir, Vfs};

pub struct ScreenpackUi {
    pub name: String,
    pub dir: PathBuf,
    pub localcoord: [f32; 2],
    pub menu: Vec<(String, String)>,
    pub life: [Option<Bar>; 2],
    pub power: [Option<Bar>; 2],
    pub characters: Vec<String>,
    pub intro: Option<Storyboard>,
    pub logo: Option<Storyboard>,
    pub warnings: Vec<String>,
}

impl ScreenpackUi {
    fn from_pack(root: &Path, pack: Screenpack) -> ScreenpackUi {
        let menu = pack.system.menu.iter().map(|m| (m.id.to_ascii_lowercase(), m.label.clone())).collect();
        let characters = pack.select.chars().map(|c| c.to_ascii_lowercase()).collect();
        let mut life = [None, None];
        let mut power = [None, None];
        for side in 0..2 {
            let bar = pack.fight.life_bar(side + 1);
            if bar.range_x[0] != 0.0 || bar.range_x[1] != 0.0 || bar.pos != [0.0, 0.0] { life[side] = Some(bar); }
            let bar = pack.fight.power_bar(side + 1);
            if bar.range_x[0] != 0.0 || bar.range_x[1] != 0.0 || bar.pos != [0.0, 0.0] { power[side] = Some(bar); }
        }
        ScreenpackUi { name: root.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "screenpack".into()), dir: root.to_path_buf(), localcoord: pack.fight.localcoord, menu, life, power, characters, intro: pack.intro, logo: pack.logo, warnings: pack.warnings }
    }

    pub fn menu_label(&self, id: &str) -> Option<&str> {
        self.menu.iter().find(|(k, _)| k == id).map(|(_, v)| v.as_str()).filter(|v| !v.trim().is_empty())
    }

    /// Roster order from `select.def` mapped to lowercase character folder names.
    pub fn roster(&self) -> &[String] { &self.characters }

    /// Position of a fighter row in the screenpack's `select.def`, if it lists it.
    pub fn roster_pos(&self, folder: &str) -> Option<usize> {
        let folder = folder.to_ascii_lowercase();
        self.characters.iter().position(|c| c == &folder || c.ends_with(&format!("/{folder}")) || c.contains(&folder))
    }

    /// (x offset, width) of the life/power bar in the engine's 426x240 HUD space, from `fight.def`.
    pub fn bar_span(&self, side: usize, power: bool, localcoord: [f32; 2]) -> Option<(f32, f32)> {
        let bar = if power { self.power.get(side).and_then(|b| b.as_ref()) } else { self.life.get(side).and_then(|b| b.as_ref()) }?;
        let kx = 426.0 / if localcoord[0] > 1.0 { localcoord[0] } else { 320.0 };
        let width = (bar.range_x[1] - bar.range_x[0]).abs() * kx;
        if width < 8.0 { return None; }
        let x = bar.pos[0] * kx;
        Some((x, width))
    }

    /// Index of the storyboard scene playing at `tick`, if any.
    pub fn scene_at(&self, tick: i64) -> Option<usize> {
        let sb = self.intro.as_ref()?;
        let mut acc = 0i64;
        for (i, scene) in sb.scenes.iter().enumerate().skip(sb.start_scene) {
            acc += scene.end_time.max(1) as i64;
            if tick < acc { return Some(i); }
        }
        None
    }

    /// First text layer of the intro scene at `tick` (used by the arcade opening when a pack provides one).
    pub fn intro_text(&self, tick: i64) -> Option<String> {
        let sb = self.intro.as_ref()?;
        let scene = sb.scenes.get(self.scene_at(tick)?)?;
        scene.layers.iter().find_map(|l| l.element.text.clone()).filter(|s| !s.trim().is_empty())
    }

    pub fn intro_total(&self) -> i64 { self.intro.as_ref().map(|s| s.total_ticks()).unwrap_or(0) }
}

/// Load a screenpack from a folder that contains `system.def` or `data/system.def`.
pub fn load_from(root: &Path) -> Result<ScreenpackUi, String> {
    let mut vfs: Vfs = Vfs::new();
    add_dir(&mut vfs, root, "").map_err(|e| format!("screenpack: {e}"))?;
    let candidates = ["data/system.def", "system.def"];
    let path = candidates.iter().find(|c| vfs.contains_key(**c)).ok_or("screenpack: system.def not found")?;
    let pack = load_screenpack(&vfs, path)?;
    Ok(ScreenpackUi::from_pack(root, pack))
}

/// Probe the usual places (next to the executable and inside `mugen/`), first readable pack wins.
pub fn probe() -> Option<ScreenpackUi> {
    let mut roots: Vec<PathBuf> = vec![];
    if let Ok(p) = std::env::var("UME_SCREENPACK") { roots.push(PathBuf::from(p)); }
    let base = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
    roots.push(base.join("mugen").join("data"));
    roots.push(base.join("mugen"));
    let packs = base.join("mugen").join("screenpacks");
    if let Ok(read) = std::fs::read_dir(&packs) {
        for e in read.flatten() { roots.push(e.path()); roots.push(e.path().join("data")); }
    }
    for root in roots {
        if let Ok(ui) = load_from(&root) {
            if ui.warnings.len() <= 8 { return Some(ui); }
            eprintln!("screenpack: {} ignored ({} warnings)", root.display(), ui.warnings.len());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("ume-screenpack-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(p.join("data")).unwrap();
        p
    }

    const SYSTEM: &str = "[Info]\nlocalcoord = 320,240\n[Files]\nselect = select.def\nfight = fight.def\n[Title Info]\nmenu.itemname.arcade = START\nmenu.itemname.survival = SURVIVE\nmenu.itemname.exit = LEAVE\n";
    const SELECT: &str = "[Characters]\nkfm\nkfm, stage=dojo\nrandomselect\n";
    const FIGHT: &str = "[Files]\n[Lifebar]\np1.pos = 10, 20\np1.range.x = 0, 100\np2.pos = 310, 20\np2.range.x = -100, 0\n[Powerbar]\np1.pos = 10, 30\np1.range.x = 0, 50\n";

    fn pack(tag: &str) -> PathBuf {
        let root = scratch(tag);
        fs::write(root.join("data/system.def"), SYSTEM).unwrap();
        fs::write(root.join("data/select.def"), SELECT).unwrap();
        fs::write(root.join("data/fight.def"), FIGHT).unwrap();
        root
    }

    #[test]
    fn loads_menu_labels_roster_and_lifebars() {
        let ui = load_from(&pack("basic")).unwrap();
        assert_eq!(ui.menu_label("arcade"), Some("START"));
        assert_eq!(ui.menu_label("survival"), Some("SURVIVE"));
        assert_eq!(ui.menu_label("versus"), None, "missing labels stay absent so the native text is used");
        assert_eq!(ui.roster(), ["kfm", "kfm"]);
        assert_eq!(ui.roster_pos("kfm"), Some(0));
        assert_eq!(ui.roster_pos("nope"), None);
        let (x, width) = ui.bar_span(0, false, [320.0, 240.0]).unwrap();
        assert!((width - 133.125).abs() < 0.5, "100 px in 320-space becomes ~133 px in the 426 HUD space: {width}");
        assert!(x > 13.0 && x < 14.0);
        assert!(ui.bar_span(1, false, [320.0, 240.0]).is_some());
        assert!(ui.power[0].is_some());
    }

    #[test]
    fn a_folder_without_system_def_is_rejected() {
        let root = scratch("empty");
        assert!(load_from(&root).is_err());
    }

    #[test]
    fn storyboard_scenes_advance_by_time() {
        let root = pack("story");
        let story = "[SceneDef]\nstartscene = 0\n[Scene 0]\nend.time = 60\nlayer0.text = HELLO\nlayer0.offset = 10,10\n[Scene 1]\nend.time = 120\nlayer0.text = FIGHT\n";
        fs::write(root.join("data/intro.def"), story).unwrap();
        let system = SYSTEM.replace("[Files]\nselect = select.def\nfight = fight.def", "[Files]\nselect = select.def\nfight = fight.def\nintro.storyboard = intro.def");
        fs::write(root.join("data/system.def"), system).unwrap();
        let ui = load_from(&root).unwrap();
        assert_eq!(ui.intro_text(10).as_deref(), Some("HELLO"));
        assert_eq!(ui.intro_text(90).as_deref(), Some("FIGHT"));
        assert_eq!(ui.intro_text(500), None, "after the last scene the storyboard is done");
        assert_eq!(ui.intro_total(), 180);
    }
}
