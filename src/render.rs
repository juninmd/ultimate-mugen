//! Match renderer (macroquad): stages, fighters, effects, debug boxes.
use crate::postfx::PostFx;
use crate::stages::{Particle, Stage, StageArt};
use crate::ui::Ui;
use crate::vfx::{FxParams, Vfx};
use macroquad::prelude::*;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};
use std::collections::HashMap;
use std::sync::Arc;
use ume_engine::chardef::{AnimFrame, Blend, CharDef, ImageData};
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

const VS: &str = "#version 100
attribute vec3 position; attribute vec2 texcoord; attribute vec4 color0;
varying lowp vec2 uv; varying lowp vec4 color;
uniform mat4 Model; uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1); color = color0 / 255.0; uv = texcoord; }";
const FS: &str = "#version 100
varying lowp vec4 color; varying lowp vec2 uv; uniform sampler2D Texture;
void main() { gl_FragColor = color * texture2D(Texture, uv); }";

pub struct Gfx { tex: HashMap<u64, Texture2D>, pub add: Material, pub view_scale: f32, pub ox: f32, pub oy: f32, pub post: PostFx, /** impact-effects intensity (0.4 reduced · 1 normal · 1.3 intense) */ pub fx_gain: f32 }

impl Gfx {
    pub fn new() -> Self {
        let add = load_material(ShaderSource::Glsl { vertex: VS, fragment: FS }, MaterialParams {
            pipeline_params: PipelineParams { color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)), ..Default::default() },
            ..Default::default()
        }).unwrap();
        Gfx { tex: HashMap::new(), add, view_scale: 3.0, ox: 0.0, oy: 0.0, post: PostFx::new(), fx_gain: 1.0 }
    }
    pub fn texture(&mut self, img: &Arc<ImageData>, nearest: bool) -> Texture2D {
        self.tex.entry(img.id).or_insert_with(|| {
            let t = Texture2D::from_rgba8(img.w as u16, img.h as u16, &img.rgba);
            t.set_filter(if nearest { FilterMode::Nearest } else { FilterMode::Linear });
            t
        }).clone()
    }
    pub fn trim(&mut self, keep: usize) { if self.tex.len() > keep { self.tex.clear(); } }
    pub fn fit(&mut self, vw: f32, vh: f32) {
        let (w, h) = (screen_width(), screen_height());
        self.view_scale = (w / vw).min(h / vh);
        self.ox = (w - vw * self.view_scale) / 2.0;
        self.oy = (h - vh * self.view_scale) / 2.0;
    }
}

pub struct DrawOpts { pub alpha: f32, pub add: bool, pub flash: f32, pub tint: Color, pub angle: f32, pub vflip: bool }
impl Default for DrawOpts { fn default() -> Self { DrawOpts { alpha: 1.0, add: false, flash: 0.0, tint: WHITE, angle: 0.0, vflip: false } } }

pub struct MatchRenderer { pub debug: bool, pub particles: Vec<Particle>, /** ids of human-controlled fighters (get the ▼ marker) */ pub human: Vec<u32> }

impl MatchRenderer {
    pub fn new(stage: &Stage) -> Self {
        let particles = match &stage.art { StageArt::Procedural { particles, .. } => particles.clone(), _ => vec![] };
        MatchRenderer { debug: false, particles, human: vec![] }
    }
    fn sx(m: &Match, x: f32) -> f32 { x - m.camera.x + m.view_w / 2.0 }
    fn sy(m: &Match, y: f32) -> f32 { y - m.camera.y + m.stage.zoffset }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite(g: &mut Gfx, m: &Match, def: &CharDef, fr: &AnimFrame, x: f32, y: f32, facing: f32, lscl: f32, extra: [f32; 2], o: &DrawOpts, shx: f32, shy: f32) {
        let Some(spr) = def.sprites.get(&(fr.group, fr.index)) else { return };
        let Some(img) = spr.image.as_ref() else { return };
        let s = g.view_scale;
        let tex = g.texture(img, spr.scale >= 0.99);
        let xs = def.c("size.xscale", 1.0) as f32; let ys = def.c("size.yscale", 1.0) as f32;
        let k = spr.scale * lscl;
        let (kx, ky) = (k * xs * extra[0] * fr.xscale, k * ys * extra[1] * fr.yscale);
        let mut flip = facing < 0.0;
        if fr.flip_h { flip = !flip; }
        let vflip = fr.flip_v ^ o.vflip;
        let px = Self::sx(m, x) + fr.x * lscl * xs * facing + shx;
        let py = Self::sy(m, y) + fr.y * lscl * ys + shy;
        let (w, h) = (spr.width * kx, spr.height * ky);
        let left = if flip { px - (spr.width - spr.axis_x) * kx } else { px - spr.axis_x * kx };
        let top = if vflip { py - (spr.height - spr.axis_y) * ky } else { py - spr.axis_y * ky };
        let params = DrawTextureParams { dest_size: Some(vec2(w * s, h * s)), flip_x: flip, flip_y: vflip, rotation: -o.angle.to_radians() * facing, pivot: Some(vec2(g.ox + px * s, g.oy + py * s)), ..Default::default() };
        let add = o.add || matches!(fr.blend, Blend::Add | Blend::Add1);
        let mut c = o.tint; c.a *= o.alpha;
        if let Some((sa, _)) = fr.alpha { c.a *= (sa as f32 / 256.0).min(1.0); }
        if add { gl_use_material(&g.add); }
        draw_texture_ex(&tex, g.ox + left * s, g.oy + top * s, c, params.clone());
        if add { gl_use_default_material(); }
        if o.flash > 0.0 {
            gl_use_material(&g.add);
            draw_texture_ex(&tex, g.ox + left * s, g.oy + top * s, Color::new(1.0, 1.0, 1.0, o.flash), params);
            gl_use_default_material();
        }
    }

    /// Draws the world (stage, reflections, fighters, particles) with the cinematic camera; returns the post-processing parameters.
    pub fn draw(&mut self, g: &mut Gfx, m: &Match, stage: &Stage, vfx: &mut Vfx, ui: &Ui) -> FxParams {
        g.fit(m.view_w, m.view_h);
        let gain = g.fx_gain;
        clear_background(BLACK);
        let (mut shx, mut shy) = (0.0, 0.0);
        if m.shake_time > 0 { shy = if m.shake_time % 2 == 1 { 1.0 } else { -1.0 } * m.shake_ampl * 0.5; shx = (rand::gen_range(-0.5, 0.5)) * m.shake_ampl * 0.3; }
        let (tx, ty) = vfx.shake(gain);
        shx += tx; shy += ty;
        // cinematic camera: punch-in on hits, KO / super zoom about the action
        let (s0, ox0, oy0) = (g.view_scale, g.ox, g.oy);
        let (zoom, focus) = vfx.camera(m, gain);
        let (fvx, fvy) = (focus.0 - m.camera.x + m.view_w / 2.0, focus.1 - m.camera.y + m.stage.zoffset);
        let (px, py) = (ox0 + fvx * s0, oy0 + fvy * s0);
        g.view_scale = s0 * zoom; g.ox = px + (ox0 - px) * zoom; g.oy = py + (oy0 - py) * zoom;
        let s = g.view_scale;
        self.draw_stage(g, m, stage, false, shx, shy);
        if m.pause.as_ref().is_some_and(|p| p.darken) { draw_rectangle(g.ox, g.oy, m.view_w * s, m.view_h * s, Color::new(0.0, 0.0, 0.04, 0.62)); }
        if gain > 0.0 { vfx.draw_back(g, m, shx, shy); }
        // soft contact shadows (3 layered ellipses)
        for f in &m.fighters {
            if f.destroyed || (f.is_helper && !f.helper_player) || f.has_assert(ume_engine::fighter::asf::INVISIBLE) { continue; }
            let hgt = (-f.pos.y).max(0.0);
            let r = 22.0 * (1.0 - hgt / 200.0).max(0.4);
            let a = 0.17 * (1.0 - hgt / 150.0).max(0.3);
            for (k, m2) in [(1.0, 1.0), (0.72, 1.0), (0.45, 1.0)].iter().enumerate() {
                let _ = m2;
                draw_ellipse(g.ox + (Self::sx(m, f.pos.x) + shx) * s, g.oy + (Self::sy(m, 0.0) + shy) * s, r * (1.0 + 0.25 * (2 - k) as f32) * s, r * 0.22 * (1.0 + 0.25 * (2 - k) as f32) * s, 0.0, Color::new(0.0, 0.0, 0.0, a));
            }
        }
        // floor reflections (procedural stages)
        if matches!(stage.art, StageArt::Procedural { .. }) && gain > 0.0 {
            for f in &m.fighters {
                if f.destroyed || (f.is_helper && !f.helper_player) || f.has_assert(ume_engine::fighter::asf::INVISIBLE) { continue; }
                self.draw_reflection(g, m, f, shx, shy);
            }
        }
        // ordered draw list
        enum It { F(usize), P(usize), E(usize) }
        let mut items: Vec<(i32, usize, It)> = vec![];
        let mut ord = 0;
        for (i, e) in m.explods.iter().enumerate() { if e.under { items.push((-100 + e.spr_priority, ord, It::E(i))); ord += 1; } }
        for (i, f) in m.fighters.iter().enumerate() { if !f.destroyed { items.push((f.spr_priority, ord, It::F(i))); ord += 1; } }
        for (i, p) in m.projectiles.iter().enumerate() { items.push((p.spr_priority, ord, It::P(i))); ord += 1; }
        for (i, e) in m.explods.iter().enumerate() { if !e.under { items.push((if e.on_top { 1000 + e.spr_priority } else { e.spr_priority }, ord, It::E(i))); ord += 1; } }
        items.sort_by_key(|x| (x.0, x.1));
        for (_, _, it) in items {
            match it {
                It::F(i) => self.draw_fighter(g, m, &m.fighters[i], shx, shy),
                It::P(i) => { let p = &m.projectiles[i]; if let Some(fr) = p.cur.frame() { Self::draw_sprite(g, m, &p.cur.def, fr, p.pos.x, p.pos.y, p.facing, p.lscl, p.scale, &DrawOpts::default(), shx, shy); } }
                It::E(i) => { let e = &m.explods[i]; if let Some(fr) = e.cur.frame() { Self::draw_sprite(g, m, &e.cur.def, fr, e.pos.x, e.pos.y, e.facing, e.lscl, e.scale, &DrawOpts { add: e.add, vflip: e.vfacing < 0.0, ..Default::default() }, shx, shy); } }
            }
        }
        self.draw_stage(g, m, stage, true, shx, shy);
        self.draw_markers(g, m, shx, shy);
        if gain > 0.0 { vfx.draw_world(g, m, shx, shy, ui); }
        if self.debug { self.draw_debug(g, m); }
        // back to the plain view transform for screen-space overlays
        g.view_scale = s0; g.ox = ox0; g.oy = oy0;
        let s = g.view_scale;
        let focus_screen = (ox0 + fvx * s0, oy0 + fvy * s0);
        if gain > 0.0 { vfx.draw_weather(gain); vfx.draw_lines(focus_screen, gain); }
        if let Some((t, c)) = m.flash.filter(|_| gain > 0.2) { draw_rectangle(g.ox, g.oy, m.view_w * s, m.view_h * s, Color::new(c[0] / 255.0, c[1] / 255.0, c[2] / 255.0, (t as f32 / 10.0).min(0.85))); }
        // letterbox bars
        let (w, h) = (screen_width(), screen_height());
        if g.ox > 0.0 { draw_rectangle(0.0, 0.0, g.ox, h, BLACK); draw_rectangle(w - g.ox, 0.0, g.ox, h, BLACK); }
        if g.oy > 0.0 { draw_rectangle(0.0, 0.0, w, g.oy, BLACK); draw_rectangle(0.0, h - g.oy, w, g.oy, BLACK); }
        let bloom = if matches!(stage.art, StageArt::Mugen(_)) { 0.35 } else { 0.5 };
        vfx.params(gain.max(0.0), (focus_screen.0 / w, focus_screen.1 / h), bloom)
    }

    /// Team brawls (more than 2 fighters): a floating life bar over every fighter and a bobbing ▼ over the one you control.
    fn draw_markers(&self, g: &Gfx, m: &Match, shx: f32, shy: f32) {
        let roots: Vec<&Fighter> = m.roots().collect();
        if roots.len() <= 2 { return; }
        let s = g.view_scale;
        for f in roots {
            if !f.alive || f.destroyed || f.has_assert(ume_engine::fighter::asf::INVISIBLE) { continue; }
            let hgt = f.def.c("size.height", 60.0) as f32 * f.lscl + 14.0;
            let (cx, cy) = (g.ox + (Self::sx(m, f.pos.x) + shx) * s, g.oy + (Self::sy(m, f.pos.y - hgt) + shy) * s);
            let mine = self.human.contains(&f.id);
            let (w, h) = ((if mine { 26.0 } else { 20.0 }) * s, (if mine { 3.0 } else { 2.2 }) * s);
            let ratio = (f.life / f.life_max).clamp(0.0, 1.0);
            let team = if f.side == 0 { Color::new(0.13, 0.9, 1.0, 1.0) } else { Color::new(1.0, 0.3, 0.43, 1.0) };
            draw_rectangle(cx - w / 2.0 - s * 0.6, cy - s * 0.6, w + s * 1.2, h + s * 1.2, Color::new(0.03, 0.03, 0.08, 0.85));
            draw_rectangle(cx - w / 2.0, cy, w * ratio, h, if ratio < 0.25 { Color::new(1.0, 0.48, 0.16, 1.0) } else { team });
            if mine {
                let bob = (m.game_time as f32 * 0.25).sin() * 1.6 * s;
                let (ty, a) = (cy - 4.0 * s + bob, 4.4 * s);
                draw_triangle(vec2(cx - a, ty - a), vec2(cx + a, ty - a), vec2(cx, ty + a * 0.4), Color::new(1.0, 0.8, 0.2, 1.0));
                draw_triangle(vec2(cx - a * 0.55, ty - a * 0.8), vec2(cx + a * 0.55, ty - a * 0.8), vec2(cx, ty), Color::new(1.0, 0.95, 0.6, 1.0));
            }
        }
    }

    fn draw_reflection(&self, g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) {
        let Some(fr) = f.frame() else { return };
        let hgt = (-f.pos.y).max(0.0);
        let a = 0.2 * (1.0 - hgt / 160.0).max(0.0);
        if a <= 0.01 { return; }
        let o = DrawOpts { alpha: a, vflip: true, tint: Color::new(0.75, 0.85, 1.0, 1.0), ..Default::default() };
        Self::draw_sprite(g, m, &f.anim_def, fr, f.pos.x, -f.pos.y, f.facing, f.lscl, f.draw_scale, &o, shx, shy);
    }

    fn draw_fighter(&self, g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) {
        if f.has_assert(ume_engine::fighter::asf::INVISIBLE) { return; }
        if let Some(ai) = &f.after_image {
            let n = ai.trail.len();
            for (i, t) in ai.trail.iter().enumerate().rev().step_by(2) {
                if i == 0 { continue; }
                let fr = AnimFrame { group: t.group, index: t.index, x: t.fx, y: t.fy, time: 1, flip_h: t.flip_h, flip_v: false, blend: Blend::None, alpha: None, xscale: 1.0, yscale: 1.0, angle: 0.0, clsn1: vec![], clsn2: vec![] };
                Self::draw_sprite(g, m, &f.anim_def, &fr, t.x, t.y, t.facing, f.lscl, f.draw_scale, &DrawOpts { alpha: 0.4 * (1.0 - i as f32 / n as f32), add: true, tint: Color::new(0.4, 0.6, 1.0, 1.0), ..Default::default() }, shx, shy);
            }
        }
        let Some(fr) = f.frame() else { return };
        let mut x = f.pos.x;
        let hit = f.hit_shake > 0 && f.move_type == 'H';
        if hit { x += if f.hit_shake % 2 == 1 { 1.5 } else { -1.5 }; }
        let mut o = DrawOpts { angle: if f.angle_draw { f.angle } else { 0.0 }, ..Default::default() };
        if hit && f.hit_shake % 4 < 2 && !f.ghv.guarded && g.fx_gain > 0.2 { o.flash = 0.55; }
        if let Some(p) = &f.palfx {
            if p.add.iter().sum::<f32>() > 200.0 { o.flash = 0.6; } else if p.add[0] > p.add[2] + 40.0 { o.tint = Color::new(1.0, 0.7, 0.6, 1.0); } else if p.add[2] > p.add[0] + 40.0 { o.tint = Color::new(0.6, 0.8, 1.0, 1.0); }
        }
        if let Some((mode, a, _)) = &f.trans { if mode.starts_with("add") { o.add = true; o.alpha = a / 256.0; } }
        // owner of an active super: pulsing golden outline glow (sprite re-drawn additively around itself)
        if m.pause.as_ref().is_some_and(|p| p.is_super && (p.by == f.id || p.by == f.root)) {
            let pulse = 0.55 + 0.35 * (m.game_time as f32 * 0.35).sin();
            for (dx, dy) in [(-1.6, 0.0), (1.6, 0.0), (0.0, -1.6), (0.0, 1.6), (-1.2, -1.2), (1.2, 1.2), (-1.2, 1.2), (1.2, -1.2)] {
                Self::draw_sprite(g, m, &f.anim_def, fr, x + dx, f.pos.y + dy, f.facing, f.lscl, f.draw_scale, &DrawOpts { add: true, alpha: pulse * 0.28, tint: Color::new(1.0, 0.8, 0.3, 1.0), ..Default::default() }, shx, shy);
            }
        }
        Self::draw_sprite(g, m, &f.anim_def, fr, x, f.pos.y, f.facing, f.lscl, f.draw_scale, &o, shx, shy);
    }

    fn draw_stage(&mut self, g: &mut Gfx, m: &Match, st: &Stage, front: bool, shx: f32, shy: f32) {
        let s = g.view_scale;
        match &st.art {
            StageArt::Procedural { back, tint, .. } => {
                if !front {
                    for l in back {
                        let x = m.view_w / 2.0 - l.w / 2.0 - m.camera.x * l.delta + shx;
                        let y = l.y - m.camera.y * l.dy + (m.stage.zoffset - 205.0) + shy;
                        let t = g.texture(&l.image, false);
                        draw_texture_ex(&t, g.ox + x * s, g.oy + y * s, WHITE, DrawTextureParams { dest_size: Some(vec2(l.w * s, l.h * s)), ..Default::default() });
                    }
                } else {
                    let w = m.view_w + 40.0;
                    for p in self.particles.iter_mut() {
                        if m.pause.is_none() { p.x += p.vx; p.y += p.vy; if p.y > m.view_h + 5.0 { p.y = -5.0; } if p.y < -10.0 { p.y = m.view_h + 5.0; } }
                        let px = ((p.x - m.camera.x * 0.9) % w + w) % w - 20.0;
                        draw_circle(g.ox + px * s, g.oy + p.y * s, p.s * s, Color::from_rgba(p.c[0], p.c[1], p.c[2], (p.a * 255.0) as u8));
                    }
                    if let Some(t) = tint { draw_rectangle(g.ox, g.oy, m.view_w * s, m.view_h * s, Color::new(t[0], t[1], t[2], t[3])); }
                }
            }
            StageArt::Mugen(bg) => {
                let scl = bg.scl;
                let (cx, cy) = (m.camera.x / scl, m.camera.y / scl);
                for l in &bg.stage.layers {
                    if (l.layer == 1) != front { continue; }
                    let key = if let Some(a) = &l.anim {
                        let tot = a.total_time.max(1) as i64;
                        let tm = (m.game_time % tot) as i32;
                        let mut acc = 0; let mut idx = 0;
                        for (i, f) in a.frames.iter().enumerate() { idx = i; if f.time < 0 || tm < acc + f.time { break; } acc += f.time; }
                        a.frames.get(idx).map(|f| (f.group, f.index)).unwrap_or(l.sprite)
                    } else { l.sprite };
                    let (Some(img), Some(meta)) = (bg.images.get(&key), bg.stage.sprites.get(&key)) else { continue };
                    let t = g.texture(img, true);
                    let gt = m.game_time as f32;
                    let sxp = (l.start[0] + l.velocity[0] * gt - cx * l.delta[0]) * scl + m.view_w / 2.0 + shx;
                    let syp = (l.start[1] + l.velocity[1] * gt - cy * l.delta[1]) * scl + (m.view_h - 240.0) / 2.0 + shy;
                    let (w, h) = (meta.w as f32 * scl, meta.h as f32 * scl);
                    let (x0, y0) = (sxp - meta.axis_x as f32 * scl, syp - meta.axis_y as f32 * scl);
                    let (stx, sty) = (w + l.tile_spacing[0] * scl, h + l.tile_spacing[1] * scl);
                    let xs: Vec<f32> = match l.tile[0] { 1 => { let st = x0 - (x0 / stx).ceil() * stx; let mut v = vec![]; let mut x = st; while x < m.view_w { v.push(x); x += stx; } v } n if n > 1 => (0..n).map(|i| x0 + i as f32 * stx).collect(), _ => vec![x0] };
                    let ys: Vec<f32> = match l.tile[1] { 1 => { let st = y0 - (y0 / sty).ceil() * sty; let mut v = vec![]; let mut y = st; while y < m.view_h { v.push(y); y += sty; } v } n if n > 1 => (0..n).map(|i| y0 + i as f32 * sty).collect(), _ => vec![y0] };
                    let add = l.trans.starts_with("add");
                    if add { gl_use_material(&g.add); }
                    for &x in &xs { for &y in &ys { draw_texture_ex(&t, g.ox + x * s, g.oy + y * s, WHITE, DrawTextureParams { dest_size: Some(vec2(w * s + 0.5, h * s + 0.5)), ..Default::default() }); } }
                    if add { gl_use_default_material(); }
                }
            }
        }
    }

    fn draw_debug(&self, g: &Gfx, m: &Match) {
        let s = g.view_scale;
        let bx = |b: &[f32; 4], c: Color| draw_rectangle_lines(g.ox + Self::sx(m, b[0]) * s, g.oy + Self::sy(m, b[1]) * s, (b[2] - b[0]) * s, (b[3] - b[1]) * s, 2.0, c);
        for f in &m.fighters {
            for b in f.boxes(2) { bx(&b, Color::new(0.2, 0.6, 1.0, 1.0)); }
            for b in f.boxes(1) { bx(&b, RED); }
            draw_text(&format!("{} {}{} a{} t{}", f.state_no, f.state_type, f.move_type, f.anim_no, f.state_time), g.ox + (Self::sx(m, f.pos.x) - 20.0) * s, g.oy + (Self::sy(m, f.pos.y) + 10.0) * s, 4.0 * s, YELLOW);
        }
        for p in &m.projectiles { for b in Match::proj_boxes(p, 1) { bx(&b, ORANGE); } }
    }
}
