use crate::air::{parse_air, Animation};
use crate::fnt::{decode_fnt, Font};
use crate::ini::{find_section, nums, parse_ini, split_top, unquote, Section};
use crate::sff::{decode_sff, DecodedSff};
use crate::snd::decode_snd_checked;
use crate::stage::BgLayer;
use crate::vfs::{norm_path, resolve, text, Vfs};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element {
    pub anim: Option<i32>,
    pub spr: Option<(i32, i32)>,
    pub font: Option<[i32; 3]>,
    pub text: Option<String>,
    pub offset: [f32; 2],
    pub scale: [f32; 2],
    pub facing: i32,
    pub vfacing: i32,
    pub layerno: i32,
    pub displaytime: i32,
    pub window: Option<[f32; 4]>,
}

impl Element {
    pub fn is_visible(&self) -> bool {
        self.anim.is_some() || self.spr.is_some() || self.text.is_some()
    }
}

fn key(prefix: &str, k: &str) -> String {
    if prefix.is_empty() {
        k.to_string()
    } else {
        format!("{prefix}.{k}")
    }
}

pub fn element(sec: &Section, prefix: &str) -> Element {
    let get = |k: &str| sec.get(&key(prefix, k));
    let n = |k: &str, d: &[f32]| nums(get(k), d);
    let anim = get("anim").map(|_| n("anim", &[-1.0])[0] as i32).filter(|&a| a >= 0);
    let spr = get("spr").map(|_| n("spr", &[-1.0, 0.0])).filter(|v| v[0] >= 0.0).map(|v| (v[0] as i32, v[1] as i32));
    let font = get("font").map(|_| {
        let v = n("font", &[-1.0, 0.0, 0.0]);
        [v[0] as i32, v.get(1).copied().unwrap_or(0.0) as i32, v.get(2).copied().unwrap_or(0.0) as i32]
    });
    let scale = n("scale", &[1.0, 1.0]);
    let offset = n("offset", &[0.0, 0.0]);
    let window = get("window").map(|_| n("window", &[0.0, 0.0, 0.0, 0.0])).filter(|v| v.len() >= 4).map(|v| [v[0], v[1], v[2], v[3]]);
    Element {
        anim,
        spr,
        font,
        text: get("text").map(unquote),
        offset: [offset[0], offset.get(1).copied().unwrap_or(0.0)],
        scale: [scale[0], scale.get(1).copied().unwrap_or(scale[0])],
        facing: if n("facing", &[1.0])[0] < 0.0 { -1 } else { 1 },
        vfacing: if n("vfacing", &[1.0])[0] < 0.0 { -1 } else { 1 },
        layerno: n("layerno", &[0.0])[0] as i32,
        displaytime: n("displaytime", &[-1.0])[0] as i32,
        window,
    }
}

pub fn pos(sec: Option<&Section>, k: &str) -> [f32; 2] {
    let v = nums(sec.and_then(|s| s.get(k)), &[0.0, 0.0]);
    [v[0], v.get(1).copied().unwrap_or(0.0)]
}

pub fn int(sec: Option<&Section>, k: &str, d: i32) -> i32 {
    nums(sec.and_then(|s| s.get(k)), &[d as f32])[0] as i32
}

pub fn string(sec: Option<&Section>, k: &str) -> Option<String> {
    sec.and_then(|s| s.get(k)).map(unquote).filter(|s| !s.is_empty())
}

#[derive(Clone, Debug, Default)]
pub struct Background {
    pub clear_color: Option<[u8; 3]>,
    pub layers: Vec<BgLayer>,
}

pub fn background(secs: &[Section], anims: &HashMap<i32, Animation>, name: &str) -> Background {
    let name = name.to_ascii_lowercase();
    let def = secs.iter().find(|s| s.lname == format!("{name}def"));
    let clear_color = def.and_then(|s| s.get("bgclearcolor")).map(|v| {
        let c = nums(Some(v), &[0.0, 0.0, 0.0]);
        [c[0] as u8, c.get(1).copied().unwrap_or(0.0) as u8, c.get(2).copied().unwrap_or(0.0) as u8]
    });
    let is_layer = |s: &&Section| s.lname == name || s.lname.strip_prefix(name.as_str()).is_some_and(|r| r.starts_with(char::is_whitespace));
    let layers = secs
        .iter()
        .filter(is_layer)
        .map(|s| {
            let n = |k: &str, d: &[f32]| nums(s.get(k), d);
            let two = |k: &str, d: f32| {
                let v = n(k, &[d, d]);
                [v[0], v.get(1).copied().unwrap_or(v[0])]
            };
            let kind = s.get("type").unwrap_or("normal").to_ascii_lowercase();
            let sp = n("spriteno", &[0.0, 0.0]);
            let tile = n("tile", &[0.0, 0.0]);
            BgLayer {
                sprite: (sp[0] as i32, sp.get(1).copied().unwrap_or(0.0) as i32),
                start: two("start", 0.0),
                delta: two("delta", 1.0),
                tile: [tile[0] as i32, tile.get(1).copied().unwrap_or(0.0) as i32],
                tile_spacing: two("tilespacing", 0.0),
                velocity: two("velocity", 0.0),
                layer: n("layerno", &[0.0])[0] as i32,
                trans: s.get("trans").unwrap_or("none").to_ascii_lowercase(),
                mask: n("mask", &[0.0])[0] != 0.0,
                anim: (kind == "anim").then(|| anims.get(&(n("actionno", &[0.0])[0] as i32)).cloned()).flatten(),
                kind,
                id: n("id", &[0.0])[0] as i32,
                position_link: false,
                window: None,
                sin_x: [0.0; 3],
                sin_y: [0.0; 3],
                alpha: [256.0, 0.0],
            }
        })
        .collect();
    Background { clear_color, layers }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Default)]
pub struct SystemDef {
    pub localcoord: [f32; 2],
    pub spr: Option<String>,
    pub snd: Option<String>,
    pub logo_storyboard: Option<String>,
    pub intro_storyboard: Option<String>,
    pub select: Option<String>,
    pub fight: Option<String>,
    pub fonts: BTreeMap<i32, String>,
    pub music: BTreeMap<String, Bgm>,
    pub menu: Vec<MenuItem>,
    pub sections: Vec<Section>,
    pub anims: HashMap<i32, Animation>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bgm {
    pub file: String,
    pub volume: i32,
    pub loop_start: i64,
    pub loop_end: i64,
    pub looping: bool,
}

const MENU_ITEMS: [&str; 13] = [
    "arcade", "versus", "teamarcade", "teamversus", "teamcoop", "survival", "survivalcoop", "training", "watch", "netplayversus", "options", "exit", "storymode",
];

fn localcoord(secs: &[Section]) -> [f32; 2] {
    let info = find_section(secs, "info");
    let v = nums(info.and_then(|s| s.get("localcoord")), &[320.0, 240.0]);
    let w = if v[0] > 0.0 { v[0] } else { 320.0 };
    [w, v.get(1).copied().filter(|&h| h > 0.0).unwrap_or(w * 0.75)]
}

fn fonts(files: Option<&Section>) -> BTreeMap<i32, String> {
    files
        .map(|s| {
            s.entries
                .iter()
                .filter_map(|(k, v)| k.strip_prefix("font").and_then(|n| n.parse().ok()).map(|n| (n, unquote(v))))
                .filter(|(_, v)| !v.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_system(src: &str) -> SystemDef {
    let secs = parse_ini(src);
    let files = find_section(&secs, "files");
    let music_sec = find_section(&secs, "music");
    let mut music = BTreeMap::new();
    if let Some(m) = music_sec {
        for (k, v) in &m.entries {
            if let Some(name) = k.strip_suffix(".bgm") {
                let g = |s: &str, d: i64| nums(m.get(&format!("{name}.bgm.{s}")), &[d as f32])[0] as i64;
                music.insert(
                    name.to_string(),
                    Bgm { file: unquote(v), volume: g("volume", 100) as i32, loop_start: g("loopstart", 0), loop_end: g("loopend", 0), looping: g("loop", 1) != 0 },
                );
            }
        }
    }
    let title = find_section(&secs, "title info");
    let menu = MENU_ITEMS
        .iter()
        .filter_map(|id| {
            let label = string(title, &format!("menu.itemname.{id}"));
            match (title, label) {
                (None, _) => Some(MenuItem { id: id.to_string(), label: id.to_ascii_uppercase() }),
                (Some(_), Some(label)) => Some(MenuItem { id: id.to_string(), label }),
                _ => None,
            }
        })
        .collect();
    SystemDef {
        localcoord: localcoord(&secs),
        spr: string(files, "spr"),
        snd: string(files, "snd"),
        logo_storyboard: string(files, "logo.storyboard"),
        intro_storyboard: string(files, "intro.storyboard"),
        select: string(files, "select"),
        fight: string(files, "fight"),
        fonts: fonts(files),
        music,
        menu,
        anims: parse_air(src),
        sections: secs,
    }
}

impl SystemDef {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }

    pub fn background(&self, name: &str) -> Background {
        background(&self.sections, &self.anims, name)
    }

    pub fn select_grid(&self) -> SelectGrid {
        let s = self.section("select info");
        let size = pos(s, "cell.size");
        let spacing = nums(s.and_then(|s| s.get("cell.spacing")), &[0.0, 0.0]);
        SelectGrid {
            rows: int(s, "rows", 2).max(1),
            columns: int(s, "columns", 5).max(1),
            wrapping: int(s, "wrapping", 0) != 0,
            pos: pos(s, "pos"),
            cell_size: if size == [0.0, 0.0] { [27.0, 27.0] } else { size },
            cell_spacing: [spacing[0], spacing.get(1).copied().unwrap_or(spacing[0])],
            show_empty_boxes: int(s, "showemptyboxes", 0) != 0,
            move_over_empty_boxes: int(s, "moveoveremptyboxes", 0) != 0,
            p1_start: pos(s, "p1.cursor.startcell"),
            p2_start: pos(s, "p2.cursor.startcell"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectGrid {
    pub rows: i32,
    pub columns: i32,
    pub wrapping: bool,
    pub pos: [f32; 2],
    pub cell_size: [f32; 2],
    pub cell_spacing: [f32; 2],
    pub show_empty_boxes: bool,
    pub move_over_empty_boxes: bool,
    pub p1_start: [f32; 2],
    pub p2_start: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectSlot {
    Char { path: String, stage: Option<String>, params: BTreeMap<String, String> },
    Random,
    Empty,
}

#[derive(Clone, Debug, Default)]
pub struct SelectDef {
    pub slots: Vec<SelectSlot>,
    pub extra_stages: Vec<String>,
    pub options: BTreeMap<String, String>,
}

impl SelectDef {
    pub fn chars(&self) -> impl Iterator<Item = &str> {
        self.slots.iter().filter_map(|s| match s {
            SelectSlot::Char { path, .. } => Some(path.as_str()),
            _ => None,
        })
    }
}

pub fn parse_select(src: &str) -> SelectDef {
    let secs = parse_ini(src);
    let mut out = SelectDef::default();
    if let Some(chars) = find_section(&secs, "characters") {
        for line in &chars.lines {
            let parts = split_top(line);
            let first = parts.first().map(|p| unquote(p)).unwrap_or_default();
            let lower = first.to_ascii_lowercase();
            let slot = match lower.as_str() {
                "randomselect" => SelectSlot::Random,
                "" | "empty" | "blank" => SelectSlot::Empty,
                _ => {
                    let mut stage = None;
                    let mut params = BTreeMap::new();
                    for p in parts.iter().skip(1) {
                        match p.split_once('=') {
                            Some((k, v)) => {
                                params.insert(k.trim().to_ascii_lowercase(), unquote(v.trim()));
                            }
                            None if stage.is_none() && !p.trim().is_empty() => stage = Some(unquote(p.trim())),
                            None => {}
                        }
                    }
                    SelectSlot::Char { path: first, stage, params }
                }
            };
            out.slots.push(slot);
        }
    }
    if let Some(st) = find_section(&secs, "extrastages") {
        out.extra_stages = st.lines.iter().map(|l| unquote(split_top(l).first().map(|s| s.as_str()).unwrap_or(""))).filter(|s| !s.is_empty()).collect();
    }
    if let Some(o) = find_section(&secs, "options") {
        out.options = o.entries.iter().map(|(k, v)| (k.clone(), unquote(v))).collect();
    }
    out
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bar {
    pub pos: [f32; 2],
    pub range_x: [f32; 2],
    pub bg0: Element,
    pub bg1: Element,
    pub mid: Element,
    pub front: Element,
    pub counter: Element,
}

#[derive(Clone, Debug, Default)]
pub struct FightDef {
    pub localcoord: [f32; 2],
    pub sff: Option<String>,
    pub snd: Option<String>,
    pub fightfx_sff: Option<String>,
    pub fightfx_air: Option<String>,
    pub common_snd: Option<String>,
    pub fonts: BTreeMap<i32, String>,
    pub sections: Vec<Section>,
    pub anims: HashMap<i32, Animation>,
}

pub fn parse_fight(src: &str) -> FightDef {
    let secs = parse_ini(src);
    let files = find_section(&secs, "files");
    FightDef {
        localcoord: localcoord(&secs),
        sff: string(files, "sff"),
        snd: string(files, "snd"),
        fightfx_sff: string(files, "fightfx.sff"),
        fightfx_air: string(files, "fightfx.air"),
        common_snd: string(files, "common.snd"),
        fonts: fonts(files),
        anims: parse_air(src),
        sections: secs,
    }
}

impl FightDef {
    pub fn section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }

    pub fn element(&self, section: &str, prefix: &str) -> Element {
        self.section(section).map(|s| element(s, prefix)).unwrap_or_default()
    }

    pub fn bar(&self, section: &str, player: usize) -> Bar {
        let s = self.section(section);
        let p = format!("p{player}");
        let e = |k: &str| s.map(|s| element(s, &format!("{p}.{k}"))).unwrap_or_default();
        Bar {
            pos: pos(s, &format!("{p}.pos")),
            range_x: pos(s, &format!("{p}.range.x")),
            bg0: e("bg0"),
            bg1: e("bg1"),
            mid: e("mid"),
            front: e("front"),
            counter: e("counter"),
        }
    }

    pub fn life_bar(&self, player: usize) -> Bar {
        self.bar("lifebar", player)
    }

    pub fn power_bar(&self, player: usize) -> Bar {
        self.bar("powerbar", player)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneLayer {
    pub element: Element,
    pub start_time: i32,
    pub end_time: Option<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub end_time: i32,
    pub fade_in: i32,
    pub fade_out: i32,
    pub clear_color: Option<[u8; 3]>,
    pub layer_pos: [f32; 2],
    pub layers: Vec<SceneLayer>,
    pub bg: Option<String>,
    pub bgm: Option<String>,
    pub bgm_loop: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Storyboard {
    pub localcoord: [f32; 2],
    pub spr: Option<String>,
    pub start_scene: usize,
    pub scenes: Vec<Scene>,
    pub sections: Vec<Section>,
    pub anims: HashMap<i32, Animation>,
}

impl Storyboard {
    pub fn total_ticks(&self) -> i64 {
        self.scenes.iter().skip(self.start_scene).map(|s| s.end_time.max(0) as i64).sum()
    }

    pub fn background(&self, name: &str) -> Background {
        background(&self.sections, &self.anims, name)
    }
}

pub fn parse_storyboard(src: &str) -> Storyboard {
    let secs = parse_ini(src);
    let def = find_section(&secs, "scenedef");
    let scenes = secs
        .iter()
        .filter(|s| s.lname == "scene" || s.lname.strip_prefix("scene").is_some_and(|r| r.starts_with(char::is_whitespace)))
        .map(|s| {
            let sec = Some(s);
            let clear = s.get("clearcolor").map(|v| {
                let c = nums(Some(v), &[0.0, 0.0, 0.0]);
                [c[0] as u8, c.get(1).copied().unwrap_or(0.0) as u8, c.get(2).copied().unwrap_or(0.0) as u8]
            });
            let layer_ids: std::collections::BTreeSet<i32> = s
                .entries
                .iter()
                .filter_map(|(k, _)| k.strip_prefix("layer").and_then(|r| r.split('.').next()).and_then(|n| n.parse().ok()))
                .collect();
            let layers = layer_ids
                .into_iter()
                .map(|n| SceneLayer {
                    element: element(s, &format!("layer{n}")),
                    start_time: int(sec, &format!("layer{n}.starttime"), 0),
                    end_time: s.get(&format!("layer{n}.endtime")).map(|_| int(sec, &format!("layer{n}.endtime"), 0)),
                })
                .collect();
            Scene {
                end_time: int(sec, "end.time", 0),
                fade_in: int(sec, "fadein.time", 0),
                fade_out: int(sec, "fadeout.time", 0),
                clear_color: clear,
                layer_pos: pos(sec, "layerall.pos"),
                layers,
                bg: string(sec, "bg.name"),
                bgm: string(sec, "bgm"),
                bgm_loop: int(sec, "bgm.loop", 0) != 0,
            }
        })
        .collect();
    Storyboard {
        localcoord: localcoord(&secs),
        spr: string(def, "spr"),
        start_scene: int(def, "startscene", 0).max(0) as usize,
        scenes,
        anims: parse_air(src),
        sections: secs,
    }
}

#[derive(Clone, Debug, Default)]
pub struct Screenpack {
    pub dir: String,
    pub system: SystemDef,
    pub select: SelectDef,
    pub fight: FightDef,
    pub sprites: Option<DecodedSff>,
    pub fight_sprites: Option<DecodedSff>,
    pub sounds: HashMap<(i32, i32), Vec<u8>>,
    pub fight_sounds: HashMap<(i32, i32), Vec<u8>>,
    pub fonts: BTreeMap<i32, Font>,
    pub fight_fonts: BTreeMap<i32, Font>,
    pub logo: Option<Storyboard>,
    pub intro: Option<Storyboard>,
    pub warnings: Vec<String>,
}

fn dir_of(p: &str) -> &str {
    p.rfind('/').map(|i| &p[..i]).unwrap_or("")
}

fn find<'a>(vfs: &'a Vfs, base: &str, name: &str) -> Option<&'a [u8]> {
    resolve(vfs, base, name).or_else(|| resolve(vfs, "data", name)).or_else(|| resolve(vfs, "font", name)).or_else(|| resolve(vfs, "", name))
}

fn load_fonts(vfs: &Vfs, base: &str, names: &BTreeMap<i32, String>, warnings: &mut Vec<String>) -> BTreeMap<i32, Font> {
    let mut out = BTreeMap::new();
    for (&n, name) in names {
        match find(vfs, base, name).map(decode_fnt) {
            Some(Ok(f)) => {
                out.insert(n, f);
            }
            Some(Err(e)) => warnings.push(format!("font{n} {name}: {e}")),
            None => warnings.push(format!("font{n} {name}: not found")),
        }
    }
    out
}

fn load_sff(vfs: &Vfs, base: &str, name: Option<&String>, warnings: &mut Vec<String>) -> Option<DecodedSff> {
    let name = name?;
    match find(vfs, base, name).map(decode_sff) {
        Some(Ok(s)) => Some(s),
        Some(Err(e)) => {
            warnings.push(format!("{name}: {e}"));
            None
        }
        None => {
            warnings.push(format!("{name}: not found"));
            None
        }
    }
}

fn load_snd(vfs: &Vfs, base: &str, name: Option<&String>, warnings: &mut Vec<String>) -> HashMap<(i32, i32), Vec<u8>> {
    let Some(name) = name else { return HashMap::new() };
    match find(vfs, base, name).map(decode_snd_checked) {
        Some(Ok(s)) => s,
        Some(Err(e)) => {
            warnings.push(format!("{name}: {e}"));
            HashMap::new()
        }
        None => {
            warnings.push(format!("{name}: not found"));
            HashMap::new()
        }
    }
}

fn load_storyboard(vfs: &Vfs, base: &str, name: Option<&String>, warnings: &mut Vec<String>) -> Option<Storyboard> {
    let name = name?;
    match find(vfs, base, name) {
        Some(b) => Some(parse_storyboard(&text(b))),
        None => {
            warnings.push(format!("{name}: not found"));
            None
        }
    }
}

pub fn load_screenpack(vfs: &Vfs, system_path: &str) -> Result<Screenpack, String> {
    let path = norm_path(system_path);
    let src = vfs.get(&path).ok_or_else(|| format!("screenpack: {path} not found"))?;
    let dir = dir_of(&path).to_string();
    let system = parse_system(&text(src));
    let mut warnings = Vec::new();
    let select = match system.select.as_ref().and_then(|n| find(vfs, &dir, n)) {
        Some(b) => parse_select(&text(b)),
        None => {
            warnings.push(format!("select.def {:?}: not found", system.select));
            SelectDef::default()
        }
    };
    let fight_path = system.fight.as_ref().and_then(|n| {
        let candidates = [format!("{dir}/{n}"), format!("data/{n}"), n.clone()];
        candidates.into_iter().map(|c| norm_path(&c)).find(|c| vfs.contains_key(c))
    });
    let (fight, fight_dir) = match &fight_path {
        Some(p) => (parse_fight(&text(&vfs[p])), dir_of(p).to_string()),
        None => {
            warnings.push(format!("fight.def {:?}: not found", system.fight));
            (FightDef::default(), dir.clone())
        }
    };
    let sprites = load_sff(vfs, &dir, system.spr.as_ref(), &mut warnings);
    let sounds = load_snd(vfs, &dir, system.snd.as_ref(), &mut warnings);
    let fonts = load_fonts(vfs, &dir, &system.fonts, &mut warnings);
    let fight_sprites = load_sff(vfs, &fight_dir, fight.sff.as_ref(), &mut warnings);
    let fight_sounds = load_snd(vfs, &fight_dir, fight.snd.as_ref(), &mut warnings);
    let fight_fonts = load_fonts(vfs, &fight_dir, &fight.fonts, &mut warnings);
    let logo = load_storyboard(vfs, &dir, system.logo_storyboard.as_ref(), &mut warnings);
    let intro = load_storyboard(vfs, &dir, system.intro_storyboard.as_ref(), &mut warnings);
    Ok(Screenpack { dir, system, select, fight, sprites, fight_sprites, sounds, fight_sounds, fonts, fight_fonts, logo, intro, warnings })
}
