use crate::chardef::AnimFrame;
use crate::diagnostics::{note_unsupported, Category};
use crate::expr::{eval, Node};
use crate::fighter::{Fighter, V2};
use crate::input::{button_bit, k, rel_dirs, HIST};
use crate::matchs::{Explod, Projectile};
use crate::palette::PalFx;
use crate::triggers::Ctx;

type Box4 = [f32; 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxKind {
    Clsn1,
    Clsn2,
    Size,
    Dummy,
}

impl BoxKind {
    pub fn from_word(word: &str) -> Option<BoxKind> {
        Some(match word { "clsn1" => BoxKind::Clsn1, "clsn2" => BoxKind::Clsn2, "size" => BoxKind::Size, "dummy" => BoxKind::Dummy, _ => return None })
    }
}

pub fn box_word(node: Option<&Node>) -> Option<&str> {
    match node? {
        Node::Trig { name, args, raw: None, comp: None } if args.is_empty() => Some(name.as_str()),
        _ => None,
    }
}

pub fn boxes(f: &Fighter, kind: BoxKind) -> Vec<Box4> {
    match kind {
        BoxKind::Clsn1 => f.boxes(1),
        BoxKind::Clsn2 => f.boxes(2),
        BoxKind::Dummy => vec![],
        BoxKind::Size => {
            let (back, front, height) = (f.c("size.ground.back", 15.0) * f.lscl, f.c("size.ground.front", 16.0) * f.lscl, f.c("size.height", 60.0) * f.lscl);
            let (x1, x2) = if f.facing >= 0.0 { (f.pos.x - back, f.pos.x + front) } else { (f.pos.x - front, f.pos.x + back) };
            vec![[x1, f.pos.y - height, x2, f.pos.y]]
        }
    }
}

pub fn overlap(a: &[Box4], b: &[Box4]) -> bool {
    a.iter().any(|x| b.iter().any(|y| x[0].min(x[2]) <= y[0].max(y[2]) && y[0].min(y[2]) <= x[0].max(x[2]) && x[1].min(x[3]) <= y[1].max(y[3]) && y[1].min(y[3]) <= x[1].max(x[3])))
}

fn unknown_variable(name: &str, raw: &str) -> Option<f64> {
    note_unsupported(Category::Trigger, &format!("{name}({raw})"));
    Some(0.0)
}

fn undefined() -> Option<f64> { Some(f64::NAN) }

fn flag(v: bool) -> f64 { v as i32 as f64 }

fn arg(c: &Ctx, args: &[Node], at: usize) -> f64 { args.get(at).map(|n| eval(n, c)).unwrap_or(0.0) }

fn owner_player_no(c: &Ctx, owner: u32) -> f64 { c.m.idx(owner).map(|o| c.m.fighters[o].player_no as f64).unwrap_or(0.0) }

fn camera_relative(c: &Ctx, world: V2, lscl: f32) -> (f64, f64) {
    (((world.x - c.m.camera.x) / lscl) as f64, (world.y / lscl) as f64)
}

fn anim_time(cur: &crate::matchs::AnimCursor) -> f64 {
    match cur.a() { None => 0.0, Some(a) if a.total_time < 0 => -1.0, Some(a) => (cur.elapsed - a.total_time) as f64 }
}

type ExplodGet = fn(&Explod, &Ctx) -> f64;

fn explod_var(name: &str) -> Option<ExplodGet> {
    let get: ExplodGet = match name {
        "id" => |e, _| e.id as f64,
        "time" => |e, _| e.time as f64,
        "removetime" => |e, _| e.remove_time as f64,
        "bindtime" => |e, _| e.bind_time as f64,
        "bindid" => |e, _| e.bind_to.map(|b| b as f64).unwrap_or(-1.0),
        "anim" => |e, _| e.cur.anim as f64,
        "animelem" => |e, _| (e.cur.elem + 1) as f64,
        "animelemtime" => |e, _| e.cur.elem_time as f64,
        "animtime" => |e, _| anim_time(&e.cur),
        "animplayerno" | "spriteplayerno" => |e, c| owner_player_no(c, e.owner),
        "facing" => |e, _| e.facing as f64,
        "ignorehitpause" => |e, _| flag(e.ignore_hit_pause),
        "layerno" => |e, _| if e.under { -1.0 } else if e.on_top { 1.0 } else { 0.0 },
        "pausemovetime" => |e, _| e.pause_move as f64,
        "supermovetime" => |e, _| e.super_move as f64,
        "sprpriority" => |e, _| e.spr_priority as f64,
        "pos.x" => |e, c| camera_relative(c, e.world_pos(c.m.camera), e.lscl).0,
        "pos.y" => |e, c| camera_relative(c, e.world_pos(c.m.camera), e.lscl).1,
        "pos.z" | "vel.z" | "accel.z" => |_, _| 0.0,
        "vel.x" => |e, _| (e.vel.x / e.lscl) as f64,
        "vel.y" => |e, _| (e.vel.y / e.lscl) as f64,
        "accel.x" => |e, _| (e.accel.x / e.lscl) as f64,
        "accel.y" => |e, _| (e.accel.y / e.lscl) as f64,
        "scale.x" => |e, _| e.scale[0] as f64,
        "scale.y" => |e, _| e.scale[1] as f64,
        _ => return None,
    };
    Some(get)
}

type ProjGet = fn(&Projectile, &Ctx) -> f64;

fn proj_var(name: &str) -> Option<ProjGet> {
    let get: ProjGet = match name {
        "projid" => |p, _| p.id as f64,
        "time" => |p, _| p.time as f64,
        "projremovetime" => |p, _| p.remove_time as f64,
        "projhits" => |p, _| p.hits as f64,
        "projmisstime" => |p, _| p.miss_time as f64,
        "projpriority" => |p, _| p.priority as f64,
        "projsprpriority" => |p, _| p.spr_priority as f64,
        "projedgebound" => |p, _| (p.edge_bound / p.lscl) as f64,
        "projstagebound" => |p, _| (p.stage_bound / p.lscl) as f64,
        "lowbound" => |p, _| (p.height_bound[0] / p.lscl) as f64,
        "highbound" => |p, _| (p.height_bound[1] / p.lscl) as f64,
        "projhitanim" => |p, _| p.hit_anim as f64,
        "projremanim" => |p, _| p.rem_anim as f64,
        "projcancelanim" => |p, _| p.cancel_anim as f64,
        "projremove" => |p, _| flag(p.remove),
        "anim" => |p, _| p.cur.anim as f64,
        "animelem" => |p, _| (p.cur.elem + 1) as f64,
        "facing" => |p, _| p.facing as f64,
        "teamside" => |p, _| (p.side + 1) as f64,
        "pausemovetime" => |p, _| p.pause_move as f64,
        "supermovetime" => |p, _| p.super_move as f64,
        "pos.x" => |p, c| camera_relative(c, p.pos, p.lscl).0,
        "pos.y" => |p, c| camera_relative(c, p.pos, p.lscl).1,
        "pos.z" | "vel.z" | "accel.z" | "remvelocity.z" => |_, _| 0.0,
        "vel.x" => |p, _| (p.vel.x / p.lscl) as f64,
        "vel.y" => |p, _| (p.vel.y / p.lscl) as f64,
        "accel.x" => |p, _| (p.accel.x / p.lscl) as f64,
        "accel.y" => |p, _| (p.accel.y / p.lscl) as f64,
        "velmul.x" => |p, _| p.vel_mul.x as f64,
        "velmul.y" => |p, _| p.vel_mul.y as f64,
        "remvelocity.x" => |p, _| (p.rem_vel.x / p.lscl) as f64,
        "remvelocity.y" => |p, _| (p.rem_vel.y / p.lscl) as f64,
        "scale.x" => |p, _| p.scale[0] as f64,
        "scale.y" => |p, _| p.scale[1] as f64,
        _ => return None,
    };
    Some(get)
}

fn palfx_var(c: &Ctx, raw: &str) -> Option<f64> {
    let (scope, field) = match raw.split_once('.') {
        Some(("bg", rest)) => ('b', rest),
        Some(("all", rest)) => ('a', rest),
        _ => ('s', raw),
    };
    type Get = fn(&PalFx) -> f64;
    let get: Get = match field {
        "time" => |p| if p.time > 0 { (p.time - p.elapsed).max(0) as f64 } else { p.time as f64 },
        "add.r" => |p| p.add[0] as f64,
        "add.g" => |p| p.add[1] as f64,
        "add.b" => |p| p.add[2] as f64,
        "mul.r" => |p| p.mul[0] as f64,
        "mul.g" => |p| p.mul[1] as f64,
        "mul.b" => |p| p.mul[2] as f64,
        "color" => |p| p.color as f64,
        "invertall" => |p| flag(p.invert_all),
        _ => return unknown_variable("palfxvar", raw),
    };
    let fx = match scope { 'b' => c.m.bg_palfx.as_ref(), 'a' => c.m.all_palfx.as_ref(), _ => c.m.fighters[c.i].palfx.as_ref() };
    Some(fx.map(get).unwrap_or_else(|| get(&PalFx::default())))
}

fn clsn_var(c: &Ctx, args: &[Node], raw: &str) -> Option<f64> {
    if !matches!(raw, "back" | "front" | "top" | "bottom") { return unknown_variable("clsnvar", raw); }
    let Some(kind) = box_word(args.first()).and_then(BoxKind::from_word) else { return undefined() };
    let f = &c.m.fighters[c.i];
    let list = boxes(f, kind);
    let Some(b) = list.get(arg(c, args, 1).max(0.0) as usize) else { return undefined() };
    let (lo, hi) = (b[0].min(b[2]) - f.pos.x, b[0].max(b[2]) - f.pos.x);
    let (back, front) = if f.facing >= 0.0 { (lo, hi) } else { (-hi, -lo) };
    let v = match raw { "back" => back, "front" => front, "top" => b[1].min(b[3]) - f.pos.y, _ => b[1].max(b[3]) - f.pos.y };
    Some((v / f.lscl) as f64)
}

fn helper_var(c: &Ctx, raw: &str) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    let value = match raw {
        "id" => f.helper_id as f64,
        "helpertype" => flag(f.helper_player),
        "keyctrl" => flag(f.key_ctrl),
        "ownpal" => flag(f.own_pal),
        "preserve" => flag(f.preserve),
        _ => return unknown_variable("helpervar", raw),
    };
    if f.is_helper { Some(value) } else { undefined() }
}

fn frame_sprite<'a>(f: &'a Fighter, fr: &AnimFrame) -> Option<&'a crate::chardef::Sprite> { f.anim_def.sprites.get(&(fr.group, fr.index)) }

fn sprite_var(c: &Ctx, raw: &str) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    if !matches!(raw, "group" | "image" | "width" | "height" | "xoffset" | "yoffset") { return unknown_variable("spritevar", raw); }
    let Some(fr) = f.frame() else { return undefined() };
    let sprite = frame_sprite(f, fr);
    Some(match raw {
        "group" => fr.group as f64,
        "image" => fr.index as f64,
        "width" => sprite.map(|s| s.width as f64).unwrap_or(0.0),
        "height" => sprite.map(|s| s.height as f64).unwrap_or(0.0),
        "xoffset" => sprite.map(|s| s.axis_x as f64).unwrap_or(0.0),
        _ => sprite.map(|s| s.axis_y as f64).unwrap_or(0.0),
    })
}

fn anim_elem_var(c: &Ctx, raw: &str) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    if !matches!(raw, "group" | "image" | "time" | "hflip" | "vflip" | "xoffset" | "yoffset" | "xscale" | "yscale" | "angle" | "alphasource" | "alphadest" | "numclsn1" | "numclsn2") {
        return unknown_variable("animelemvar", raw);
    }
    let Some(fr) = f.frame() else { return undefined() };
    let alpha = fr.alpha.unwrap_or((256, 0));
    Some(match raw {
        "group" => fr.group as f64,
        "image" => fr.index as f64,
        "time" => fr.time as f64,
        "hflip" => flag(fr.flip_h),
        "vflip" => flag(fr.flip_v),
        "xoffset" => fr.x as f64,
        "yoffset" => fr.y as f64,
        "xscale" => fr.xscale as f64,
        "yscale" => fr.yscale as f64,
        "angle" => fr.angle as f64,
        "alphasource" => alpha.0 as f64,
        "alphadest" => alpha.1 as f64,
        "numclsn1" => fr.clsn1.len() as f64,
        _ => fr.clsn2.len() as f64,
    })
}

fn key_down(bits: u32, facing: i32, key: &str) -> Option<bool> {
    let relative = rel_dirs(bits, facing);
    Some(match key {
        "F" => relative & 8 != 0,
        "B" => relative & 4 != 0,
        "U" => bits & k::U != 0,
        "D" => bits & k::D != 0,
        "L" => bits & k::L != 0,
        "R" => bits & k::R != 0,
        "N" => bits == 0,
        other => bits & button_bit(other)? != 0,
    })
}

fn input_time(c: &Ctx, raw: &str) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    if key_down(0, 1, raw).is_none() { return unknown_variable("inputtime", raw); }
    let mut held = 0;
    let mut t = f.history.tick;
    while t >= 0 && held < HIST as i64 - 1 && key_down(f.history.at(t), f.history.face_at(t), raw) == Some(true) { held += 1; t -= 1; }
    Some(held as f64)
}

fn root_group<'a>(c: &'a Ctx) -> impl Iterator<Item = usize> + 'a {
    let root = c.m.fighters[c.i].root;
    (0..c.m.fighters.len()).filter(move |&k| c.m.fighters[k].root == root && !c.m.fighters[k].destroyed)
}

fn proj_clsn_overlap(c: &Ctx, args: &[Node]) -> f64 {
    let f = &c.m.fighters[c.i];
    let Some(kind) = box_word(args.get(2)).and_then(BoxKind::from_word).filter(|k| *k != BoxKind::Dummy) else { return 0.0 };
    let Some(p) = c.m.projectiles.iter().filter(|p| p.owner == f.id && !p.remove).nth(arg(c, args, 0).max(0.0) as usize) else { return 0.0 };
    let Some(target) = c.m.idx(arg(c, args, 1) as u32) else { return 0.0 };
    let Some(fr) = p.cur.frame() else { return 0.0 };
    let (sx, sy) = (p.scale[0] * p.lscl, p.scale[1] * p.lscl);
    let own: Vec<Box4> = fr.clsn1.iter().map(|b| {
        let (x1, x2) = if p.facing < 0.0 { (-b[2] * sx, -b[0] * sx) } else { (b[0] * sx, b[2] * sx) };
        [p.pos.x + x1, p.pos.y + b[1] * sy, p.pos.x + x2, p.pos.y + b[3] * sy]
    }).collect();
    flag(overlap(&own, &boxes(&c.m.fighters[target], kind)))
}

pub fn trigger(c: &Ctx, name: &str, args: &[Node], raw: Option<&str>) -> Option<f64> {
    let f = &c.m.fighters[c.i];
    let raw_name = raw.unwrap_or("");
    Some(match name {
        "explodvar" => {
            let Some(get) = explod_var(raw_name) else { return unknown_variable(name, raw_name) };
            let id = arg(c, args, 0) as i32;
            let found = c.m.explods.iter().filter(|e| e.owner == f.id && (id < 0 || e.id == id)).nth(arg(c, args, 1).max(0.0) as usize);
            return Some(found.map(|e| get(e, c)).unwrap_or(f64::NAN));
        }
        "projvar" => {
            let Some(get) = proj_var(raw_name) else { return unknown_variable(name, raw_name) };
            let id = arg(c, args, 0) as i32;
            let found = c.m.projectiles.iter().filter(|p| p.owner == f.id && !p.remove && (id < 0 || p.id == id)).nth(arg(c, args, 1).max(0.0) as usize);
            return Some(found.map(|p| get(p, c)).unwrap_or(f64::NAN));
        }
        "palfxvar" => return palfx_var(c, raw_name),
        "clsnvar" => return clsn_var(c, args, raw_name),
        "helpervar" => return helper_var(c, raw_name),
        "spritevar" => return sprite_var(c, raw_name),
        "animelemvar" => return anim_elem_var(c, raw_name),
        "inputtime" => return input_time(c, raw.unwrap_or("")),
        "projclsnoverlap" => proj_clsn_overlap(c, args),
        "animlength" => f.anim().map(|a| a.total_time as f64).unwrap_or(0.0),
        "animplayerno" | "spriteplayerno" => c.m.fighters.iter().find(|o| std::sync::Arc::ptr_eq(&o.def, &f.anim_def)).map(|o| o.player_no as f64).unwrap_or(f.player_no as f64),
        "ikemenversion" => match raw_name { "major" => 1.0, "minor" => 0.0, _ => return unknown_variable(name, raw_name) },
        "index" => root_group(c).position(|k| k == c.i).unwrap_or(0) as f64,
        "runorder" => (c.i + 1) as f64,
        "helperindexexist" => {
            let wanted = arg(c, args, 0);
            flag(wanted >= 0.0 && c.m.fighters.iter().filter(|h| h.is_helper && h.root == f.root && !h.destroyed).nth(wanted as usize).is_some())
        }
        _ => return None,
    })
}
