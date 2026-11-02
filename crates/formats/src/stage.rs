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
}

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
}

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
    let zoffset = g("stageinfo", "zoffset", 200.0);

    let mut sprites = HashMap::new();
    let bgdef = secs.iter().find(|s| s.lname == "bgdef");
    if let Some(spr) = bgdef.and_then(|s| s.get("spr")) {
        let buf = resolve(vfs, base_dir, spr).or_else(|| resolve(vfs, "stages", spr));
        if let Some(buf) = buf {
            let sff = decode_sff(buf).map_err(|e| format!("stage {def_path}: {e}"))?;
            for s in &sff.sprites {
                sprites.entry((s.group, s.index)).or_insert_with(|| StageImage {
                    w: s.width,
                    h: s.height,
                    axis_x: s.axis_x,
                    axis_y: s.axis_y,
                    rgba: sprite_to_rgba(s, None),
                });
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
            kind,
        });
    }

    Ok(MugenStage {
        name,
        local_w,
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
    })
}
