//! MUGEN stage .def loader (data part of `loadMugenStage` in
//! `src/mugen/loader.ts`; no rendering). All values are in the stage's local
//! units: callers scale by `320 / local_w`.

use crate::air::{parse_air, Animation};
use crate::ini::{find_section, nums, parse_ini, unquote, Section};
use crate::sff::{decode_sff, sprite_to_rgba};
use crate::vfs::{norm_path, resolve, text, Vfs};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BgLayer {
    /// Lowercase `type` (`normal`, `anim`, `parallax` (draw as normal), ...).
    pub kind: String,
    pub sprite: (i32, i32),
    pub start: [f32; 2],
    pub delta: [f32; 2],
    pub tile: [i32; 2],
    pub tile_spacing: [f32; 2],
    pub velocity: [f32; 2],
    /// 0 = back, 1 = front.
    pub layer: i32,
    /// Lowercase `trans` (`none`, `add`, `add1`, `sub`, `addalpha`, ...).
    pub trans: String,
    pub mask: bool,
    /// For `type = anim`: the `actionno` animation from the same .def.
    pub anim: Option<Animation>,
    pub id: i32,
    pub position_link: bool,
    pub window: Option<[f32; 4]>,
    pub sin_x: [f32; 3],
    pub sin_y: [f32; 3],
    pub alpha: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub enum BgCtrlKind { Null, Visible(bool), Enable(bool), VelSet(Option<f32>, Option<f32>), VelAdd(Option<f32>, Option<f32>), PosSet(Option<f32>, Option<f32>), PosAdd(Option<f32>, Option<f32>), Anim(i32), SinX([f32; 3]), SinY([f32; 3]) }

#[derive(Clone, Debug)]
pub struct BgCtrl { pub kind: BgCtrlKind, pub start: i32, pub end: i32, pub loop_time: i32, pub ids: Vec<i32> }

#[derive(Clone, Debug)]
pub struct BgCtrlDef { pub loop_time: i32, pub ids: Vec<i32>, pub ctrls: Vec<BgCtrl> }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StageMusic { pub file: String, pub volume: f32, pub loop_start: i64, pub loop_end: i64 }

#[derive(Clone, Debug, PartialEq)]
pub struct StageShadow { pub intensity: f32, pub color: [f32; 3], pub yscale: f32, pub fade_range: [f32; 2] }

#[derive(Clone)]
pub struct StageImage {
    pub w: u32,
    pub h: u32,
    pub axis_x: i32,
    pub axis_y: i32,
    /// RGBA `w * h * 4` (index 0 transparent).
    pub rgba: Vec<u8>,
}

impl std::fmt::Debug for StageImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StageImage")
            .field("w", &self.w)
            .field("h", &self.h)
            .field("axis_x", &self.axis_x)
            .field("axis_y", &self.axis_y)
            .field("rgba_len", &self.rgba.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct MugenStage {
    pub name: String,
    /// `[StageInfo] localcoord` width (default 320).
    pub local_w: f32,
    /// `[StageInfo] localcoord` height (default 240). Cenarios 16:9 (1280x720)
    /// usam 720 aqui: o eixo vertical escala por `240/local_h` para o cenario
    /// preencher a tela em vez de sobrar faixa preta embaixo.
    pub local_h: f32,
    pub bound_left: f32,
    pub bound_right: f32,
    pub bound_high: f32,
    pub bound_low: f32,
    pub p1x: f32,
    pub p2x: f32,
    pub zoffset: f32,
    pub vertical_follow: f32,
    pub tension: f32,
    pub layers: Vec<BgLayer>,
    pub sprites: HashMap<(i32, i32), StageImage>,
    pub anims: HashMap<i32, Animation>,
    pub ctrl_defs: Vec<BgCtrlDef>,
    pub music: Option<StageMusic>,
    pub shadow: StageShadow,
    pub reflection: f32,
    pub start_x: f32,
    pub start_y: f32,
    pub zoom_out: f32,
    pub zoom_in: f32,
    pub info_name: String,
    pub display_name: String,
    pub author: String,
    pub mugen_version: String,
    pub round_time: f32,
    pub auto_turn: bool,
}

fn bgctrl_kind(s: &Section) -> Option<BgCtrlKind> {
    let f = |k: &str| s.get(k).and_then(|v| v.split(',').next()).and_then(|v| v.trim().parse::<f32>().ok());
    let v = || nums(s.get("value"), &[0.0])[0];
    let tri = || { let v = nums(s.get("value"), &[0.0, 0.0, 0.0]); [v[0], v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0)] };
    Some(match s.get("type")?.trim().to_ascii_lowercase().as_str() {
        "null" => BgCtrlKind::Null,
        "visible" => BgCtrlKind::Visible(v() != 0.0),
        "enable" | "enabled" => BgCtrlKind::Enable(v() != 0.0),
        "velset" => BgCtrlKind::VelSet(f("x"), f("y")),
        "veladd" => BgCtrlKind::VelAdd(f("x"), f("y")),
        "posset" => BgCtrlKind::PosSet(f("x"), f("y")),
        "posadd" => BgCtrlKind::PosAdd(f("x"), f("y")),
        "anim" => BgCtrlKind::Anim(v() as i32),
        "sinx" => BgCtrlKind::SinX(tri()),
        "siny" => BgCtrlKind::SinY(tri()),
        _ => return None,
    })
}

fn ids(s: &Section) -> Vec<i32> { s.get("ctrlid").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default() }

fn parse_bgctrls(secs: &[Section]) -> Vec<BgCtrlDef> {
    let mut defs: Vec<BgCtrlDef> = vec![];
    for s in secs {
        if s.lname.starts_with("bgctrldef") {
            defs.push(BgCtrlDef { loop_time: nums(s.get("looptime"), &[-1.0])[0] as i32, ids: ids(s), ctrls: vec![] });
        } else if s.lname.starts_with("bgctrl") {
            let (Some(def), Some(kind)) = (defs.last_mut(), bgctrl_kind(s)) else { continue };
            let t = nums(s.get("time"), &[0.0]);
            let start = t[0] as i32;
            let end = t.get(1).map(|v| *v as i32).unwrap_or(start);
            let loop_time = t.get(2).map(|v| *v as i32).unwrap_or(-1);
            let own = ids(s);
            def.ctrls.push(BgCtrl { kind, start, end, loop_time, ids: if own.is_empty() { def.ids.clone() } else { own } });
        }
    }
    defs
}

fn pair3(v: Vec<f32>) -> [f32; 3] { [v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0)] }

fn pair(v: Vec<f32>, def: [f32; 2]) -> [f32; 2] {
    [v.first().copied().unwrap_or(def[0]), v.get(1).copied().unwrap_or(def[1])]
}

pub fn load_stage(vfs: &Vfs, def_path: &str) -> Result<MugenStage, String> {
    let def_path = norm_path(def_path);
    let base_dir = def_path.rfind('/').map(|i| &def_path[..i]).unwrap_or("");
    let t = text(vfs.get(&def_path).ok_or_else(|| format!("stage: {def_path} not found"))?);
    let secs = parse_ini(&t);
    let get = |s: &str, k: &str| find_section(&secs, s).and_then(|s| s.get(k));
    let g = |s: &str, k: &str, d: f32| nums(get(s, k), &[d])[0];

    let name = {
        let n = unquote(get("info", "displayname").or_else(|| get("info", "name")).unwrap_or(""));
        if n.is_empty() {
            def_path.clone()
        } else {
            n
        }
    };
    let lc = nums(get("stageinfo", "localcoord"), &[320.0, 240.0]);
    let local_w = if lc[0] == 0.0 { 320.0 } else { lc[0] };
    let local_h = lc.get(1).copied().filter(|h| *h > 0.0).unwrap_or(240.0);
    let zoffset = g("stageinfo", "zoffset", 200.0);

    let mut sprites = HashMap::new();
    let bgdef = secs.iter().find(|s| s.lname == "bgdef");
    if let Some(spr) = bgdef.and_then(|s| s.get("spr")) {
        let buf = resolve(vfs, base_dir, spr).or_else(|| resolve(vfs, "stages", spr));
        if let Some(buf) = buf {
            let sff = decode_sff(buf).map_err(|e| format!("stage {def_path}: {e}"))?;
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<_> = sff.sprites.iter().filter(|s| seen.insert((s.group, s.index))).collect();
            let rgba = crate::par::map(&unique, |s| sprite_to_rgba(s, None));
            for (s, rgba) in unique.into_iter().zip(rgba) {
                sprites.insert((s.group, s.index), StageImage { w: s.width, h: s.height, axis_x: s.axis_x, axis_y: s.axis_y, rgba });
            }
        }
    }

    let anims = parse_air(&t);
    let is_bg = |s: &Section| {
        s.lname == "bg" || (s.lname.starts_with("bg") && s.lname[2..].starts_with(|c: char| c.is_whitespace()))
    };
    let mut layers = Vec::new();
    for s in secs.iter().filter(|s| is_bg(s)) {
        let kind = s.get("type").unwrap_or("normal").to_lowercase();
        let n = |k: &str, d: &[f32]| nums(s.get(k), d);
        let sp = pair(n("spriteno", &[0.0, 0.0]), [0.0, 0.0]);
        let delta_v = n("delta", &[1.0, 1.0]);
        let delta = [delta_v[0], delta_v.get(1).copied().unwrap_or(delta_v[0])];
        let tile = pair(n("tile", &[0.0, 0.0]), [0.0, 0.0]);
        let anim = if kind == "anim" { anims.get(&(n("actionno", &[0.0])[0] as i32)).cloned() } else { None };
        let win = if s.get("window").is_some() { Some(n("window", &[0.0, 0.0, 0.0, 0.0])) } else if s.get("maskwindow").is_some() { Some(n("maskwindow", &[0.0, 0.0, 0.0, 0.0])) } else { None };
        let three = |k: &str| { let v = n(k, &[0.0, 0.0, 0.0]); [v[0], v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0)] };
        let alpha = n("alpha", &[256.0, 0.0]);
        layers.push(BgLayer {
            sprite: (sp[0] as i32, sp[1] as i32),
            start: pair(n("start", &[0.0, 0.0]), [0.0, 0.0]),
            delta,
            tile: [tile[0] as i32, tile[1] as i32],
            tile_spacing: pair(n("tilespacing", &[0.0, 0.0]), [0.0, 0.0]),
            velocity: pair(n("velocity", &[0.0, 0.0]), [0.0, 0.0]),
            layer: n("layerno", &[0.0])[0] as i32,
            trans: s.get("trans").unwrap_or("none").to_lowercase(),
            mask: n("mask", &[0.0])[0] != 0.0,
            anim,
            id: n("id", &[0.0])[0] as i32,
            position_link: n("positionlink", &[0.0])[0] != 0.0,
            window: win.filter(|w| w.len() >= 4).map(|w| [w[0], w[1], w[2], w[3]]),
            sin_x: three("sin.x"),
            sin_y: three("sin.y"),
            alpha: [alpha[0], alpha.get(1).copied().unwrap_or(0.0)],
            kind,
        });
    }

    Ok(MugenStage {
        name,
        local_w,
        local_h,
        bound_left: g("camera", "boundleft", -150.0),
        bound_right: g("camera", "boundright", 150.0),
        bound_high: g("camera", "boundhigh", -25.0),
        bound_low: g("camera", "boundlow", 0.0),
        p1x: g("playerinfo", "p1startx", -70.0),
        p2x: g("playerinfo", "p2startx", 70.0),
        zoffset,
        vertical_follow: g("camera", "verticalfollow", 0.2),
        tension: g("camera", "tension", 50.0),
        layers,
        sprites,
        ctrl_defs: parse_bgctrls(&secs),
        music: get("music", "bgmusic").map(unquote).filter(|f| !f.is_empty()).map(|file| StageMusic {
            file,
            volume: g("music", "bgvolume", 0.0),
            loop_start: g("music", "bgmloopstart", 0.0) as i64,
            loop_end: g("music", "bgmloopend", 0.0) as i64,
        }),
        shadow: StageShadow {
            intensity: g("shadow", "intensity", 128.0).clamp(0.0, 256.0),
            color: pair3(nums(get("shadow", "color"), &[0.0, 0.0, 0.0])),
            yscale: g("shadow", "yscale", 0.4),
            fade_range: pair(nums(get("shadow", "fade.range"), &[0.0, 0.0]), [0.0, 0.0]),
        },
        reflection: g("reflection", "intensity", 0.0).clamp(0.0, 256.0),
        start_x: g("camera", "startx", 0.0),
        start_y: g("camera", "starty", 0.0),
        zoom_out: g("camera", "zoomout", 1.0),
        zoom_in: g("camera", "zoomin", 1.0),
        info_name: unquote(get("info", "name").unwrap_or("")),
        display_name: unquote(get("info", "displayname").unwrap_or("")),
        author: unquote(get("info", "author").unwrap_or("")),
        mugen_version: unquote(get("info", "mugenversion").unwrap_or("")),
        round_time: g("stageinfo", "roundtime", -1.0),
        auto_turn: g("playerinfo", "autoturn", 1.0) != 0.0,
        anims,
    })
}
