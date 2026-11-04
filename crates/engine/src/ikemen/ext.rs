use crate::chardef::Controller;
use crate::expr::{eval, Node};
use crate::fighter::V2;
use crate::matchs::{Event, Match};
use crate::triggers::Ctx;
use super::introspect::BoxKind;

#[derive(Clone, Debug, PartialEq)]
pub struct TextItem {
    pub owner: u32,
    pub id: i32,
    pub text: String,
    pub pos: V2,
    pub vel: V2,
    pub scale: [f32; 2],
    pub color: [u8; 3],
    pub align: i32,
    pub time: i32,
}

fn unquote(s: &str) -> String { s.trim().trim_matches('"').to_string() }

pub fn format_text(fmt: &str, vals: &[f64]) -> String {
    let mut out = String::with_capacity(fmt.len() + 8);
    let mut it = vals.iter();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' { out.push(c); continue; }
        match chars.next() {
            Some('d') | Some('i') => out.push_str(&(it.next().copied().unwrap_or(0.0) as i64).to_string()),
            Some('f') => out.push_str(&format!("{:.6}", it.next().copied().unwrap_or(0.0))),
            Some('%') => out.push('%'),
            Some(o) => { out.push('%'); out.push(o); }
            None => out.push('%'),
        }
    }
    out.replace("\\n", "\n")
}

fn vals(m: &Match, i: usize, sc: &Controller, k: &str) -> Vec<f64> {
    sc.param(k).map(|p| p.exprs.iter().map(|e| m.ev(i, e)).collect()).unwrap_or_default()
}

fn teammate(m: &Match, i: usize) -> Option<usize> {
    let me = &m.fighters[i];
    (0..m.fighters.len()).find(|&k| k != i && !m.fighters[k].is_helper && !m.fighters[k].destroyed && m.fighters[k].side == me.side)
}

fn tag(m: &mut Match, i: usize, sc: &Controller, standby: bool) {
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0);
    if n(m, "self", 1.0) != 0.0 {
        m.fighters[i].ikemen.standby = standby;
        if sc.has("ctrl", 0) { m.fighters[i].ctrl = n(m, "ctrl", 0.0) != 0.0; }
        if sc.has("stateno", 0) { let (s, d) = (n(m, "stateno", 0.0) as i32, m.fighters[i].def.clone()); m.change_state(i, s, d); }
    }
    if sc.has("partnerstateno", 0) {
        if let Some(p) = teammate(m, i) {
            let s = n(m, "partnerstateno", 0.0) as i32;
            m.fighters[p].ikemen.standby = !standby;
            if sc.has("partnerctrl", 0) { m.fighters[p].ctrl = n(m, "partnerctrl", 0.0) != 0.0; }
            let d = m.fighters[p].def.clone();
            m.change_state(p, s, d);
        }
    }
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) -> bool {
    let n = |m: &Match, k: &str, d: f64, x: usize| m.num(i, sc, k, d, x) as f32;
    match sc.kind.as_str() {
        "text" => {
            let text = format_text(&sc.raw("text").map(unquote).unwrap_or_default(), &vals(m, i, sc, "params"));
            let c = [n(m, "color", 255.0, 0), n(m, "color", 255.0, 1), n(m, "color", 255.0, 2)].map(|v| v.clamp(0.0, 255.0) as u8);
            let item = TextItem {
                owner: m.fighters[i].root, id: n(m, "id", -1.0, 0) as i32, text,
                pos: V2 { x: n(m, "pos", 0.0, 0), y: n(m, "pos", 0.0, 1) },
                vel: V2 { x: n(m, "velocity", 0.0, 0), y: n(m, "velocity", 0.0, 1) },
                scale: [n(m, "scale", 1.0, 0), n(m, "scale", 1.0, 1)],
                color: c, align: n(m, "align", 1.0, 0) as i32, time: n(m, "removetime", 1.0, 0) as i32,
            };
            m.texts.push(item);
        }
        "removetext" => {
            let (id, owner) = (n(m, "id", -1.0, 0) as i32, m.fighters[i].root);
            m.texts.retain(|t| t.owner != owner || (id >= 0 && t.id != id));
        }
        "tagin" => tag(m, i, sc, false),
        "tagout" => tag(m, i, sc, true),
        "modifyprojectile" => {
            let (id, owner) = (n(m, "id", -1.0, 0) as i32, m.fighters[i].id);
            let pv = |k: &str| sc.has(k, 0).then(|| V2 { x: n(m, k, 0.0, 0), y: n(m, k, 0.0, 1) });
            let (vel, acc, mul) = (pv("velocity"), pv("accel"), pv("velmul"));
            let rt = sc.has("removetime", 0).then(|| n(m, "removetime", -1.0, 0) as i32);
            let hits = sc.has("projhits", 0).then(|| n(m, "projhits", 1.0, 0) as i32);
            let pri = sc.has("projpriority", 0).then(|| n(m, "projpriority", 1.0, 0) as i32);
            let scale = sc.has("projscale", 0).then(|| [n(m, "projscale", 1.0, 0), n(m, "projscale", 1.0, 1)]);
            for p in m.projectiles.iter_mut().filter(|p| p.owner == owner && (id < 0 || p.id == id)) {
                if let Some(v) = vel { p.vel = V2 { x: v.x * p.facing, y: v.y }; }
                if let Some(v) = acc { p.accel = V2 { x: v.x * p.facing, y: v.y }; }
                if let Some(v) = mul { p.vel_mul = v; }
                if let Some(v) = rt { p.remove_time = v; }
                if let Some(v) = hits { p.hits = v; }
                if let Some(v) = pri { p.priority = v; }
                if let Some(v) = scale { p.scale = v; }
            }
        }
        "modifysnd" => {
            let channel = n(m, "channel", -1.0, 0) as i32;
            let owner = m.fighters[i].id;
            if sc.has("pan", 0) || sc.has("abspan", 0) { crate::sctrl::sound::pan(m, i, sc); }
            if sc.has("volumescale", 0) { let volume = n(m, "volumescale", 100.0, 0) / 100.0; m.events.push(Event::SoundVolume { owner, channel, volume }); }
        }
        _ => return false,
    }
    true
}

pub fn tick(m: &mut Match) {
    for t in m.texts.iter_mut() {
        t.pos.x += t.vel.x;
        t.pos.y += t.vel.y;
        if t.time > 0 { t.time -= 1; }
    }
    m.texts.retain(|t| t.time != 0);
}

pub fn trigger(c: &Ctx, name: &str, args: &[Node]) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    Some(match name {
        "standby" => f.ikemen.standby as i32 as f64,
        "numtext" => c.m.texts.iter().filter(|t| t.owner == f.root).count() as f64,
        "clsnoverlap" => {
            let word = |at: usize| super::introspect::box_word(args.get(at));
            let kinds = (word(0).and_then(BoxKind::from_word), word(2).and_then(BoxKind::from_word).filter(|k| *k != BoxKind::Dummy));
            let (Some(own_kind), Some(other_kind)) = kinds else { return Some(0.0) };
            let id = args.get(1).map(|n| eval(n, c)).unwrap_or(-1.0) as u32;
            let Some(o) = c.m.idx(id) else { return Some(0.0) };
            super::introspect::overlap(&super::introspect::boxes(f, own_kind), &super::introspect::boxes(&c.m.fighters[o], other_kind)) as i32 as f64
        }
        _ => return None,
    })
}
