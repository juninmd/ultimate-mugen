use crate::chardef::{CharDef, Controller};
use crate::expr::Node;
use crate::matchs::Match;
use crate::triggers::Ctx;

mod ext;
pub mod life;
mod maps;
pub use ext::{format_text, tick, TextItem};
pub use maps::{MapStore, MAP_MAX_KEYS, MAP_MAX_KEY_LEN};

#[derive(Clone, Debug, Default)]
pub struct IkemenState {
    pub map: MapStore,
    pub red_life: f32,
    pub dizzy_points: f32,
    pub dizzy_max: f32,
    pub guard_points: f32,
    pub guard_max: f32,
    pub dizzy: bool,
    pub guard_break: bool,
    pub stun_by_hit: bool,
    pub standby: bool,
}

impl IkemenState {
    pub fn new(def: &CharDef) -> Self {
        let life = def.c("data.life", 1000.0) as f32;
        let dizzy_max = def.c("data.dizzypoints", life as f64) as f32;
        let guard_max = def.c("data.guardpoints", life as f64) as f32;
        IkemenState { map: MapStore::default(), red_life: life, dizzy_points: dizzy_max, dizzy_max, guard_points: guard_max, guard_max, dizzy: false, guard_break: false, stun_by_hit: false, standby: false }
    }
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) -> bool {
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0) as f32;
    let kind = sc.kind.as_str();
    if maps::run(m, i, sc) { return true; }
    match kind {
        "redlifeset" | "redlifeadd" => {
            let v = n(m, "value", 0.0);
            let f = &mut m.fighters[i];
            let dv = if kind == "redlifeadd" && n_absolute(sc) && v < 0.0 { v } else if kind == "redlifeadd" && v < 0.0 { v / f.defence_mul } else { v };
            let base = if kind == "redlifeadd" { f.ikemen.red_life } else { 0.0 };
            f.ikemen.red_life = (base + dv).clamp(f.life.max(0.0), f.life_max);
        }
        "dizzypointsset" | "dizzypointsadd" => {
            let v = n(m, "value", 0.0);
            let s = &mut m.fighters[i].ikemen;
            let base = if kind == "dizzypointsadd" { s.dizzy_points } else { 0.0 };
            s.dizzy_points = (base + v).clamp(0.0, s.dizzy_max);
        }
        "guardpointsset" | "guardpointsadd" => {
            let v = n(m, "value", 0.0);
            let s = &mut m.fighters[i].ikemen;
            let base = if kind == "guardpointsadd" { s.guard_points } else { 0.0 };
            s.guard_points = (base + v).clamp(0.0, s.guard_max);
        }
        "dizzyset" => { let v = n(m, "value", 1.0) != 0.0; m.fighters[i].ikemen.dizzy = v; }
        "guardbreakset" => { let v = n(m, "value", 1.0) != 0.0; m.fighters[i].ikemen.guard_break = v; }
        _ => return ext::run(m, i, sc),
    }
    true
}

fn n_absolute(sc: &Controller) -> bool { sc.raw("absolute").is_some_and(|v| v.trim() != "0") }

fn name_arg(args: &[Node]) -> Option<String> {
    match args.first()? {
        Node::Trig { name, args, raw: None, .. } if args.is_empty() => Some(name.clone()),
        Node::Trig { name, raw: Some(s), .. } if name == "__str" => Some(s.to_ascii_lowercase()),
        _ => None,
    }
}

pub fn trigger(c: &Ctx, name: &str, args: &[Node]) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    let s = &f.ikemen;
    let b = |x: bool| x as i32 as f64;
    Some(match name {
        "map" => name_arg(args).map(|k| s.map.get(&k) as f64).unwrap_or(0.0),
        "teammap" => name_arg(args).map(|k| maps::team_value(c.m, c.i, &k) as f64).unwrap_or(0.0),
        "redlife" => s.red_life as f64,
        "dizzy" => b(s.dizzy),
        "dizzypoints" => s.dizzy_points as f64,
        "dizzypointsmax" => s.dizzy_max as f64,
        "guardbreak" => b(s.guard_break),
        "guardpoints" => s.guard_points as f64,
        "guardpointsmax" => s.guard_max as f64,
        _ => return ext::trigger(c, name, args),
    })
}

#[cfg(test)]
mod tests;
