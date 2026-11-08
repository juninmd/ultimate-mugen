use crate::expr::{eval, Node};
use crate::palette::BlendMode;
use crate::triggers::Ctx;

fn parts(raw: Option<&str>) -> (&str, &str) {
    raw.and_then(|r| r.split_once(','))
        .map(|(a, b)| (a.trim(), b.trim()))
        .unwrap_or(("", raw.unwrap_or("").trim()))
}

fn stage_bg_var(c: &Ctx, raw: Option<&str>) -> f64 {
    let Some(bg) = c.m.bg.as_ref() else {
        return 0.0;
    };
    let (id, key) = parts(raw);
    let Ok(id) = id.parse::<i32>() else {
        return 0.0;
    };
    let Some(k) = bg.stage.layers.iter().position(|l| l.id == id) else {
        return 0.0;
    };
    let layer = &bg.stage.layers[k];
    let offset = bg.offset(k);
    let (group, index) = bg.sprite(k);
    let state = &bg.layers[k];
    (match key {
        "pos.x" => layer.start[0] + offset[0],
        "pos.y" => layer.start[1] + offset[1],
        "start.x" => layer.start[0],
        "start.y" => layer.start[1],
        "delta.x" => layer.delta[0],
        "delta.y" => layer.delta[1],
        "velocity.x" | "vel.x" => state.vel[0],
        "velocity.y" | "vel.y" => state.vel[1],
        "tile.x" => layer.tile[0] as f32,
        "tile.y" => layer.tile[1] as f32,
        "layerno" => layer.layer as f32,
        "id" => layer.id as f32,
        "visible" => bg.visible(k) as i32 as f32,
        "enabled" | "enable" => state.enabled as i32 as f32,
        "sprite.group" | "spriteno.group" => group as f32,
        "sprite.index" | "spriteno.index" => index as f32,
        _ => 0.0,
    }) as f64
}

pub fn trigger(
    c: &Ctx,
    name: &str,
    args: &[Node],
    comp: Option<char>,
    raw: Option<&str>,
) -> Option<f64> {
    let m = c.m;
    let f = &m.fighters[c.i];
    let l = f.lscl as f64;
    let arg0 = || args.first().map(|n| eval(n, c)).unwrap_or(0.0);
    let ground = m.ground_y();
    let top = m.effective_bound_high() - ground;
    let bottom = m.stage.bound_low + m.view_h - ground;
    let height = f.c("size.height", 60.0) * f.lscl;
    let stage_left = m.stage.bound_left - m.view_w / 2.0;
    let stage_right = m.stage.bound_right + m.view_w / 2.0;
    Some(match name {
        "screenwidth" => m.view_w as f64 / l,
        "screenheight" => m.view_h as f64 / l,
        "topbounddist" => (f.pos.y - top) as f64 / l,
        "topboundbodydist" => (f.pos.y - top - height) as f64 / l,
        "botbounddist" | "botboundbodydist" => (bottom - f.pos.y) as f64 / l,
        "stagefrontedgedist" => {
            (if f.facing > 0.0 {
                stage_right - f.pos.x
            } else {
                f.pos.x - stage_left
            }) as f64
                / l
        }
        "stagebackedgedist" => {
            (if f.facing > 0.0 {
                f.pos.x - stage_left
            } else {
                stage_right - f.pos.x
            }) as f64
                / l
        }
        "numstagebg" => m.bg.as_ref().map_or(0, |bg| {
            bg.stage
                .layers
                .iter()
                .filter(|layer| args.is_empty() || layer.id == arg0() as i32)
                .count()
        }) as f64,
        "stagebgvar" => stage_bg_var(c, raw),
        "stageconst" => {
            m.bg.as_ref()
                .and_then(|bg| bg.stage.constants.get(raw.unwrap_or("").trim()))
                .copied()
                .unwrap_or(0.0) as f64
        }
        "const1080p" => arg0() * f.def.local_w as f64 / 1920.0,
        "localcoord" => {
            if comp == Some('y') {
                f.def.local_h as f64
            } else {
                f.def.local_w as f64
            }
        }
        "angle" => f.angle as f64,
        "xangle" | "yangle" | "xshear" => 0.0,
        "scale" => f.draw_scale[if comp == Some('y') { 1 } else { 0 }] as f64,
        "alpha" => match &f.trans {
            Some(t) if t.mode != BlendMode::None => {
                if comp == Some('y') {
                    t.dst as f64
                } else {
                    t.src as f64
                }
            }
            _ => {
                if comp == Some('y') {
                    0.0
                } else {
                    256.0
                }
            }
        },
        "layerno" => 0.0,
        "sprpriority" => f.spr_priority as f64,
        _ => return None,
    })
}
