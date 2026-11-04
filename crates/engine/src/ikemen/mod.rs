use crate::chardef::{CharDef, Controller};
use crate::expr::Node;
use crate::matchs::Match;
use crate::triggers::Ctx;

mod ext;
pub use ext::{format_text, tick, TextItem};

#[derive(Clone, Debug, Default)]
pub struct IkemenState {
    pub map: Vec<(String, f32)>,
    pub red_life: f32,
    pub dizzy_points: f32,
    pub dizzy_max: f32,
    pub guard_points: f32,
    pub guard_max: f32,
    pub dizzy: bool,
    pub guard_break: bool,
    pub standby: bool,
    pub score: f32,
}

impl IkemenState {
    pub fn new(def: &CharDef) -> Self {
        let life = def.c("data.life", 1000.0) as f32;
        let dizzy_max = def.c("data.dizzypoints", life as f64) as f32;
        let guard_max = def.c("data.guardpoints", life as f64) as f32;
        IkemenState { map: vec![], red_life: life, dizzy_points: dizzy_max, dizzy_max, guard_points: guard_max, guard_max, dizzy: false, guard_break: false, standby: false, score: 0.0 }
    }

    pub fn get(&self, key: &str) -> f32 {
        self.map.iter().find(|(k, _)| k == key).map(|(_, v)| *v).unwrap_or(0.0)
    }

    pub fn set(&mut self, key: &str, v: f32, add: bool) {
        match self.map.iter_mut().find(|(k, _)| k == key) {
            Some((_, x)) => *x = if add { *x + v } else { v },
            None => self.map.push((key.to_string(), v)),
        }
    }
}

fn map_key(sc: &Controller) -> Option<String> {
    let raw = sc.raw("map")?;
    let k = raw.trim().trim_matches('"').trim().to_ascii_lowercase();
    (!k.is_empty()).then_some(k)
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) -> bool {
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0) as f32;
    let kind = sc.kind.as_str();
    match kind {
        "mapset" | "mapadd" => {
            let Some(key) = map_key(sc) else { return true };
            let v = n(m, "value", 0.0);
            let add = kind == "mapadd" || n(m, "type", 0.0) as i32 == 1;
            m.fighters[i].ikemen.set(&key, v, add);
        }
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
        "map" => name_arg(args).map(|k| s.get(&k) as f64).unwrap_or(0.0),
        "redlife" => s.red_life as f64,
        "score" => c.m.idx(f.root).map_or(s.score, |r| c.m.fighters[r].ikemen.score) as f64,
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
