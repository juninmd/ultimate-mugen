use crate::gfx::*;
use crate::postfx::PostFx;
use crate::stagefx::{Ctx, StageReact};
use crate::stages::{Particle, Stage, StageArt};
use crate::ui::Ui;
use crate::vfx::{FxParams, Vfx};
use std::collections::HashMap;
use std::sync::Arc;
use ume_engine::chardef::{AnimFrame, Blend, CharDef, ImageData};
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;
use ume_engine::palette::{BlendMode, PalParams};

fn draw_transformed_layer_quad(
    texture: &Texture2D,
    pivot: [f32; 2],
    axis: [f32; 2],
    size: [f32; 2],
    angle_deg: f32,
    x_shear: f32,
    color: Color,
) {
    let (sin, cos) = (-angle_deg.to_radians()).sin_cos();
    let corners = [
        [0.0, 0.0],
        [size[0], 0.0],
        [size[0], size[1]],
        [0.0, size[1]],
    ];
    let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let vertices = (0..4)
        .map(|i| {
            let (rx, ry) = (corners[i][0] - axis[0], corners[i][1] - axis[1]);
            let sheared = rx - x_shear * ry;
            Vertex::new(
                pivot[0] + sheared * cos - ry * sin,
                pivot[1] + sheared * sin + ry * cos,
                0.0,
                uvs[i][0],
                uvs[i][1],
                color,
            )
        })
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture.clone()),
    });
}

pub struct Gfx {
    tex: HashMap<u64, Texture2D>,
    indexed: HashMap<u64, IndexedTexture>,
    palettes: HashMap<usize, (Arc<Vec<u8>>, Palette)>,
    pub view_scale: f32,
    pub ox: f32,
    pub oy: f32,
    pub post: PostFx,
    flush_cache: bool,
    /** impact-effects intensity (0.4 reduced · 1 normal · 1.3 intense) */
    pub fx_gain: f32,
}

impl Gfx {
    pub fn new() -> Self {
        Gfx {
            tex: HashMap::new(),
            indexed: HashMap::new(),
            palettes: HashMap::new(),
            view_scale: 3.0,
            ox: 0.0,
            oy: 0.0,
            post: PostFx::new(),
            flush_cache: false,
            fx_gain: 1.0,
        }
    }
    pub fn texture(&mut self, img: &Arc<ImageData>, nearest: bool) -> Texture2D {
        self.tex
            .entry(img.id)
            .or_insert_with(|| {
                let t = Texture2D::from_rgba8(img.w as u16, img.h as u16, &img.rgba);
                t.set_filter(if nearest {
                    FilterMode::Nearest
                } else {
                    FilterMode::Linear
                });
                t
            })
            .clone()
    }
    pub fn palette(&mut self, pal: &Arc<Vec<u8>>) -> Palette {
        self.palettes
            .entry(Arc::as_ptr(pal) as usize)
            .or_insert_with(|| (pal.clone(), Palette::from_rgba(pal)))
            .1
            .clone()
    }
    pub fn sprite_tex(
        &mut self,
        def: &CharDef,
        key: (i32, i32),
        remap: &[((i32, i32), (i32, i32))],
    ) -> Option<SpriteTex> {
        let spr = def.sprites.get(&key)?;
        if let Some(ix) = def.indexed.get(&key) {
            let pal = (!remap.is_empty())
                .then(|| def.remap_palette(key, remap))
                .flatten()
                .or(ix.palette.as_ref());
            if let Some(pal) = pal.filter(|_| ix.w > 0 && ix.h > 0) {
                let palette = self.palette(pal);
                let tex = self
                    .indexed
                    .entry(ix.id)
                    .or_insert_with(|| IndexedTexture::new(ix.w, ix.h, ix.pixels.clone()))
                    .clone();
                return Some(SpriteTex::Indexed(tex, palette));
            }
        }
        let img = spr.image.as_ref()?;
        Some(SpriteTex::Rgba(self.texture(img, spr.scale >= 0.99)))
    }
    pub fn prefetch(&mut self, def: &CharDef, max_occupancy: f32) -> bool {
        let keys: Vec<(i32, i32)> = def.sprites.keys().copied().collect();
        keys.into_iter().all(|k| {
            self.sprite_tex(def, k, &[])
                .is_none_or(|t| t.prefetch(max_occupancy))
        })
    }
    pub fn trim(&mut self, keep: usize) {
        if self.tex.len() + self.indexed.len() > keep {
            self.tex.clear();
            self.indexed.clear();
            self.palettes.clear();
        }
    }
    /// Ask for a texture-cache flush at the start of the next frame (never mid-frame: pending draw batches still reference the textures).
    pub fn trim_next_frame(&mut self) {
        self.flush_cache = true;
    }
    pub fn begin_frame(&mut self) {
        if self.flush_cache {
            self.flush_cache = false;
            self.tex.clear();
            self.indexed.clear();
            self.palettes.clear();
        }
    }
    pub fn fit(&mut self, vw: f32, vh: f32) {
        let (w, h) = (screen_width(), screen_height());
        self.view_scale = (w / vw).min(h / vh);
        self.ox = (w - vw * self.view_scale) / 2.0;
        self.oy = (h - vh * self.view_scale) / 2.0;
    }
}

pub enum SpriteTex {
    Rgba(Texture2D),
    Indexed(IndexedTexture, Palette),
}

impl SpriteTex {
    pub fn draw(&self, x: f32, y: f32, c: Color, p: DrawTextureParams) {
        match self {
            SpriteTex::Rgba(t) => draw_texture_ex(t, x, y, c, p),
            SpriteTex::Indexed(t, pal) => draw_indexed_ex(t, pal, x, y, c, p),
        }
    }
    pub fn prefetch(&self, max_occupancy: f32) -> bool {
        match self {
            SpriteTex::Rgba(t) => t.prefetch(max_occupancy),
            SpriteTex::Indexed(t, _) => t.prefetch(max_occupancy),
        }
    }
}

pub struct DrawOpts<'a> {
    pub alpha: f32,
    pub add: bool,
    pub sub: bool,
    pub flash: f32,
    pub tint: Color,
    pub angle: f32,
    pub vflip: bool,
    pub pal: Option<PalParams>,
    pub remap: &'a [((i32, i32), (i32, i32))],
}
impl<'a> Default for DrawOpts<'a> {
    fn default() -> Self {
        DrawOpts {
            alpha: 1.0,
            add: false,
            sub: false,
            flash: 0.0,
            tint: WHITE,
            angle: 0.0,
            vflip: false,
            pal: None,
            remap: &[],
        }
    }
}

pub struct MatchRenderer {
    pub debug: bool,
    pub particles: Vec<Particle>,
    /** ids of human-controlled fighters (get the ▼ marker) */
    pub human: Vec<u32>,
    pub squash: Vec<[f32; 2]>,
    /** animated + reactive stage layer (see `stagefx`: `fx.react(&[Impulse])`) */
    pub fx: StageReact,
}

impl MatchRenderer {
    pub fn new(stage: &Stage) -> Self {
        let particles = match &stage.art {
            StageArt::Procedural { particles, .. } => particles.clone(),
            _ => vec![],
        };
        MatchRenderer {
            debug: false,
            particles,
            human: vec![],
            squash: vec![],
            fx: StageReact::new(stage.kind),
        }
    }
    fn sx(m: &Match, x: f32) -> f32 {
        x - m.camera.x + m.view_w / 2.0
    }
    fn sy(m: &Match, y: f32) -> f32 {
        y - m.camera.y + m.stage.zoffset
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite(
        g: &mut Gfx,
        m: &Match,
        def: &CharDef,
        fr: &AnimFrame,
        x: f32,
        y: f32,
        facing: f32,
        lscl: f32,
        extra: [f32; 2],
        o: &DrawOpts<'_>,
        shx: f32,
        shy: f32,
    ) {
        let Some(spr) = def.sprites.get(&(fr.group, fr.index)) else {
            return;
        };
        let s = g.view_scale;
        let Some(tex) = g.sprite_tex(def, (fr.group, fr.index), &o.remap) else {
            return;
        };
        let xs = def.c("size.xscale", 1.0) as f32;
        let ys = def.c("size.yscale", 1.0) as f32;
        let k = spr.scale * lscl;
        let (kx, ky) = (k * xs * extra[0] * fr.xscale, k * ys * extra[1] * fr.yscale);
        let mut flip = facing < 0.0;
        if fr.flip_h {
            flip = !flip;
        }
        let vflip = fr.flip_v ^ o.vflip;
        let px = Self::sx(m, x) + fr.x * lscl * xs * facing + shx;
        let py = Self::sy(m, y) + fr.y * lscl * ys + shy;
        let (w, h) = (spr.width * kx, spr.height * ky);
        let left = if flip {
            px - (spr.width - spr.axis_x) * kx
        } else {
            px - spr.axis_x * kx
        };
        let top = if vflip {
            py - (spr.height - spr.axis_y) * ky
        } else {
            py - spr.axis_y * ky
        };
        let mut params = DrawTextureParams {
            dest_size: Some(vec2(w * s, h * s)),
            flip_x: flip,
            flip_y: vflip,
            rotation: -o.angle.to_radians() * facing,
            pivot: Some(vec2(g.ox + px * s, g.oy + py * s)),
            ..Default::default()
        };
        let sub = o.sub || (matches!(fr.blend, Blend::Sub) && !o.add);
        let add = !sub && (o.add || matches!(fr.blend, Blend::Add | Blend::Add1));
        let mut c = o.tint;
        c.a *= o.alpha;
        if let Some((sa, _)) = fr.alpha {
            c.a *= (sa as f32 / 256.0).min(1.0);
        }
        let pal = o.pal.filter(|p| !p.is_identity() && !sub);
        params.pal_fx = pal.map(|p| PalFx {
            add: p.add.map(|v| v / 255.0),
            mul: p.mul.map(|v| v / 256.0),
            color: p.color / 256.0,
            invert: p.invert,
            hue: p.hue,
        });
        if add {
            set_blend(GfxBlend::Add);
        } else if sub {
            set_blend(GfxBlend::Sub);
        }
        if o.flash > 0.0 {
            tex.draw(g.ox + left * s, g.oy + top * s, c, params.clone());
            if add || sub {
                set_blend(GfxBlend::Alpha);
            }
            set_blend(GfxBlend::Add);
            params.pal_fx = None;
            tex.draw(
                g.ox + left * s,
                g.oy + top * s,
                Color::new(1.0, 1.0, 1.0, o.flash),
                params,
            );
            set_blend(GfxBlend::Alpha);
        } else {
            tex.draw(g.ox + left * s, g.oy + top * s, c, params);
            if add || sub {
                set_blend(GfxBlend::Alpha);
            }
        }
    }

    /// Draws the world (stage, reflections, fighters, particles) with the cinematic camera; returns the post-processing parameters.
    pub fn draw(
        &mut self,
        g: &mut Gfx,
        m: &Match,
        stage: &Stage,
        vfx: &mut Vfx,
        ui: &Ui,
    ) -> FxParams {
        g.fit(m.view_w, m.view_h);
        let gain = g.fx_gain;
        clear_background(BLACK);
        let (mut shx, mut shy) = (0.0, 0.0);
        if m.shake_time > 0 {
            if m.shake_vertical() {
                shy = m.shake_offset();
            }
            if m.shake_horizontal() {
                shx = (rand::gen_range(-0.5, 0.5)) * m.shake_ampl.abs() * 0.3;
            }
        }
        let (tx, ty) = vfx.shake(gain);
        shx += tx;
        shy += ty;
        // cinematic camera: punch-in on hits, KO / super zoom about the action
        let (s0, ox0, oy0) = (g.view_scale, g.ox, g.oy);
        self.squash = (0..m.fighters.len())
            .map(|i| {
                if gain > 0.0 {
                    vfx.squash_scale(i)
                } else {
                    [1.0, 1.0]
                }
            })
            .collect();
        let (zoom, focus) = vfx.camera(m, gain);
        let (fvx, fvy) = (
            focus.0 - m.camera.x + m.view_w / 2.0,
            focus.1 - m.camera.y + m.stage.zoffset,
        );
        let (px, py) = (ox0 + fvx * s0, oy0 + fvy * s0);
        g.view_scale = s0 * zoom;
        g.ox = px + (ox0 - px) * zoom;
        g.oy = py + (oy0 - py) * zoom;
        let s = g.view_scale;
        self.fx.begin_frame(m, gain);
        self.draw_stage(g, m, stage, false, shx, shy);
        if m.pause.as_ref().is_some_and(|p| p.darken) {
            draw_rectangle(
                g.ox,
                g.oy,
                m.view_w * s,
                m.view_h * s,
                Color::new(0.0, 0.0, 0.04, 0.62),
            );
        }
        if gain > 0.0 {
            vfx.draw_back(g, m, shx, shy);
        }
        let global_noshadow = m.global_asserts & ume_engine::flags::gsf::GLOBALNOSHADOW != 0;
        // soft contact shadows (3 layered ellipses)
        for f in m.fighters.iter().filter(|_| {
            matches!(
                stage.art,
                StageArt::Procedural { .. } | StageArt::Illustrated { .. }
            )
        }) {
            if f.destroyed
                || (f.is_helper && !f.helper_player)
                || f.has_assert(
                    ume_engine::flags::asf::INVISIBLE | ume_engine::flags::asf::NOSHADOW,
                )
                || global_noshadow
            {
                continue;
            }
            let hgt = (-f.pos.y).max(0.0);
            let r = 22.0 * (1.0 - hgt / 200.0).max(0.4);
            let a = 0.17 * (1.0 - hgt / 150.0).max(0.3);
            for (k, m2) in [(1.0, 1.0), (0.72, 1.0), (0.45, 1.0)].iter().enumerate() {
                let _ = m2;
                draw_ellipse(
                    g.ox + (Self::sx(m, f.pos.x) + shx) * s,
                    g.oy + (Self::sy(m, 0.0) + shy) * s,
                    r * (1.0 + 0.25 * (2 - k) as f32) * s,
                    r * 0.22 * (1.0 + 0.25 * (2 - k) as f32) * s,
                    0.0,
                    Color::new(0.0, 0.0, 0.0, a),
                );
            }
        }
        if let StageArt::Mugen(bg) = &stage.art {
            let (sh, refl) = (&bg.stage.shadow, bg.stage.reflection);
            for f in &m.fighters {
                if f.destroyed
                    || f.has_assert(
                        ume_engine::flags::asf::INVISIBLE | ume_engine::flags::asf::NOSHADOW,
                    )
                    || global_noshadow
                {
                    continue;
                }
                let Some(fr) = f.frame() else { continue };
                if sh.intensity > 0.0 {
                    let tint = Color::new(
                        sh.color[0] / 255.0,
                        sh.color[1] / 255.0,
                        sh.color[2] / 255.0,
                        1.0,
                    );
                    let o = DrawOpts {
                        alpha: sh.intensity / 256.0,
                        tint,
                        vflip: true,
                        ..Default::default()
                    };
                    Self::draw_sprite(
                        g,
                        m,
                        &f.anim_def,
                        fr,
                        f.pos.x,
                        -f.pos.y * sh.yscale,
                        f.facing,
                        f.lscl,
                        [f.draw_scale[0], f.draw_scale[1] * sh.yscale.abs()],
                        &o,
                        shx,
                        shy,
                    );
                }
                if refl > 0.0 {
                    let o = DrawOpts {
                        alpha: refl / 256.0,
                        add: true,
                        vflip: true,
                        ..Default::default()
                    };
                    Self::draw_sprite(
                        g,
                        m,
                        &f.anim_def,
                        fr,
                        f.pos.x,
                        -f.pos.y,
                        f.facing,
                        f.lscl,
                        f.draw_scale,
                        &o,
                        shx,
                        shy,
                    );
                }
            }
        }
        // floor reflections (procedural stages)
        if matches!(
            stage.art,
            StageArt::Procedural { .. } | StageArt::Illustrated { .. }
        ) && gain > 0.0
        {
            for f in &m.fighters {
                if f.destroyed
                    || (f.is_helper && !f.helper_player)
                    || f.has_assert(
                        ume_engine::flags::asf::INVISIBLE | ume_engine::flags::asf::NOSHADOW,
                    )
                    || global_noshadow
                {
                    continue;
                }
                self.draw_reflection(g, m, f, shx, shy);
            }
        }
        // ordered draw list
        enum It {
            F(usize),
            P(usize),
            E(usize),
        }
        let mut items: Vec<(i32, usize, It)> = vec![];
        let mut ord = 0;
        for (i, e) in m.explods.iter().enumerate() {
            if e.under {
                items.push((-100 + e.spr_priority, ord, It::E(i)));
                ord += 1;
            }
        }
        for (i, f) in m.fighters.iter().enumerate() {
            if !f.destroyed {
                items.push((
                    f.spr_priority
                        - if f.has_assert(ume_engine::flags::asf::DRAWUNDER) {
                            1000
                        } else {
                            0
                        },
                    ord,
                    It::F(i),
                ));
                ord += 1;
            }
        }
        for (i, p) in m.projectiles.iter().enumerate() {
            items.push((p.spr_priority, ord, It::P(i)));
            ord += 1;
        }
        for (i, e) in m.explods.iter().enumerate() {
            if !e.under {
                items.push((
                    if e.on_top {
                        1000 + e.spr_priority
                    } else {
                        e.spr_priority
                    },
                    ord,
                    It::E(i),
                ));
                ord += 1;
            }
        }
        items.sort_by_key(|x| (x.0, x.1));
        for (_, _, it) in items {
            match it {
                It::F(i) => self.draw_fighter(g, m, &m.fighters[i], i, shx, shy),
                It::P(i) => {
                    let p = &m.projectiles[i];
                    if let Some(fr) = p.cur.frame() {
                        Self::draw_sprite(
                            g,
                            m,
                            &p.cur.def,
                            fr,
                            p.pos.x,
                            p.pos.y,
                            p.facing,
                            p.lscl,
                            p.scale,
                            &DrawOpts::default(),
                            shx,
                            shy,
                        );
                    }
                }
                It::E(i) => {
                    let e = &m.explods[i];
                    let remap = m
                        .idx(e.owner)
                        .map(|o| m.fighters[o].remap.as_slice())
                        .unwrap_or(&[]);
                    if let Some(fr) = e.cur.frame() {
                        let p = e.world_pos(m.camera);
                        Self::draw_sprite(
                            g,
                            m,
                            &e.cur.def,
                            fr,
                            p.x,
                            p.y,
                            e.facing,
                            e.lscl,
                            e.scale,
                            &DrawOpts {
                                add: e.add,
                                vflip: e.vfacing < 0.0,
                                remap,
                                ..Default::default()
                            },
                            shx,
                            shy,
                        );
                    }
                }
            }
        }
        self.draw_stage(g, m, stage, true, shx, shy);
        self.draw_markers(g, m, shx, shy);
        if gain > 0.0 {
            vfx.draw_world(g, m, shx, shy, ui);
        }
        if self.debug {
            self.draw_debug(g, m);
        }
        // back to the plain view transform for screen-space overlays
        g.view_scale = s0;
        g.ox = ox0;
        g.oy = oy0;
        let s = g.view_scale;
        let focus_screen = (ox0 + fvx * s0, oy0 + fvy * s0);
        if gain > 0.0 {
            vfx.draw_weather(gain);
            vfx.draw_lines(focus_screen, gain);
        }
        if let Some((t, c)) = m.flash.filter(|_| gain > 0.2) {
            draw_rectangle(
                g.ox,
                g.oy,
                m.view_w * s,
                m.view_h * s,
                Color::new(
                    c[0] / 255.0,
                    c[1] / 255.0,
                    c[2] / 255.0,
                    (t as f32 / 10.0).min(0.85),
                ),
            );
        }
        // letterbox bars
        let (w, h) = (screen_width(), screen_height());
        if g.ox > 0.0 {
            draw_rectangle(0.0, 0.0, g.ox, h, BLACK);
            draw_rectangle(w - g.ox, 0.0, g.ox, h, BLACK);
        }
        if g.oy > 0.0 {
            draw_rectangle(0.0, 0.0, w, g.oy, BLACK);
            draw_rectangle(0.0, h - g.oy, w, g.oy, BLACK);
        }
        let bloom = if matches!(stage.art, StageArt::Mugen(_)) {
            0.35
        } else {
            0.5
        };
        let mut p = vfx.params(
            gain.max(0.0),
            (focus_screen.0 / w, focus_screen.1 / h),
            bloom,
        );
        self.fx.grade(&mut p, gain);
        p
    }

    /// Team brawls (more than 2 fighters): a floating life bar over every fighter and a bobbing ▼ over the one you control.
    fn draw_markers(&self, g: &Gfx, m: &Match, shx: f32, shy: f32) {
        let roots: Vec<&Fighter> = m.roots().collect();
        if roots.len() <= 2 {
            return;
        }
        let s = g.view_scale;
        for f in roots {
            if !f.alive || f.destroyed || f.has_assert(ume_engine::flags::asf::INVISIBLE) {
                continue;
            }
            let hgt = f.def.c("size.height", 60.0) as f32 * f.lscl + 14.0;
            let (cx, cy) = (
                g.ox + (Self::sx(m, f.pos.x) + shx) * s,
                g.oy + (Self::sy(m, f.pos.y - hgt) + shy) * s,
            );
            let mine = self.human.contains(&f.id);
            let (w, h) = (
                (if mine { 26.0 } else { 20.0 }) * s,
                (if mine { 3.0 } else { 2.2 }) * s,
            );
            let ratio = (f.life / f.life_max).clamp(0.0, 1.0);
            let team = if f.side == 0 {
                Color::new(0.13, 0.9, 1.0, 1.0)
            } else {
                Color::new(1.0, 0.3, 0.43, 1.0)
            };
            draw_rectangle(
                cx - w / 2.0 - s * 0.6,
                cy - s * 0.6,
                w + s * 1.2,
                h + s * 1.2,
                Color::new(0.03, 0.03, 0.08, 0.85),
            );
            draw_rectangle(
                cx - w / 2.0,
                cy,
                w * ratio,
                h,
                if ratio < 0.25 {
                    Color::new(1.0, 0.48, 0.16, 1.0)
                } else {
                    team
                },
            );
            if mine {
                let bob = (m.game_time as f32 * 0.25).sin() * 1.6 * s;
                let (ty, a) = (cy - 4.0 * s + bob, 4.4 * s);
                draw_triangle(
                    vec2(cx - a, ty - a),
                    vec2(cx + a, ty - a),
                    vec2(cx, ty + a * 0.4),
                    Color::new(1.0, 0.8, 0.2, 1.0),
                );
                draw_triangle(
                    vec2(cx - a * 0.55, ty - a * 0.8),
                    vec2(cx + a * 0.55, ty - a * 0.8),
                    vec2(cx, ty),
                    Color::new(1.0, 0.95, 0.6, 1.0),
                );
            }
        }
    }

    /// Glossy floor reflection: the sprite mirrored about the floor line as a vertex-coloured strip mesh — it fades out with distance
    /// from the feet (per-vertex alpha) and wobbles horizontally on water / wet stages. Strength comes from the stage look.
    fn draw_reflection(&self, g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) {
        let Some(fr) = f.frame() else { return };
        let (gloss, wobble) = self.fx.gloss();
        let hgt = (-f.pos.y).max(0.0);
        let a0 = gloss.max(0.12) * (1.0 - hgt / 170.0).max(0.0);
        if a0 <= 0.01 {
            return;
        }
        let def = &f.anim_def;
        let Some(spr) = def.sprites.get(&(fr.group, fr.index)) else { return };
        let Some(img) = def.rgba_sprite((fr.group, fr.index)) else { return };
        let s = g.view_scale;
        let tex = g.texture(&img, spr.scale >= 0.99);
        let (xs, ys) = (def.c("size.xscale", 1.0) as f32, def.c("size.yscale", 1.0) as f32);
        let k = spr.scale * f.lscl;
        let (kx, ky) = (
            k * xs * f.draw_scale[0] * fr.xscale,
            k * ys * f.draw_scale[1] * fr.yscale,
        );
        let mut flip = f.facing < 0.0;
        if fr.flip_h {
            flip = !flip;
        }
        let px = Self::sx(m, f.pos.x) + fr.x * f.lscl * xs * f.facing + shx;
        let py = Self::sy(m, -f.pos.y) + fr.y * f.lscl * ys + shy;
        let (w, h) = (spr.width * kx, spr.height * ky);
        let left = if flip {
            px - (spr.width - spr.axis_x) * kx
        } else {
            px - spr.axis_x * kx
        };
        // mirror about the floor line (not about the feet): a jumping fighter's image must not hang in mid-air
        let floor = Self::sy(m, 0.0) + shy;
        let bottom = py + (spr.height - spr.axis_y) * ky;
        let top = 2.0 * floor - bottom;
        if bottom > floor + 46.0 {
            return;
        }
        let reach = (floor + 46.0 - top).clamp(1.0, h);
        let rows = 12usize;
        let tint = self.fx.ambient(f.pos.x, m.camera.x);
        let (u0, u1) = if flip { (1.0, 0.0) } else { (0.0, 1.0) };
        let t = self.fx.time();
        let mut vertices = Vec::with_capacity((rows + 1) * 2);
        for j in 0..=rows {
            let q = j as f32 / rows as f32;
            let y = top + q * reach;
            let d = ((y - floor) / 46.0).clamp(0.0, 1.0);
            let a = a0 * (1.0 - d * d * (3.0 - 2.0 * d)) * if y < floor { 0.0 } else { 1.0 };
            let wob = wobble * (t * 4.0 + (y - floor) * 0.5).sin() * (0.5 + d * 1.6);
            let v = 1.0 - q * reach / h;
            let c = Color::new(tint.r * 0.8, tint.g * 0.9, tint.b, a);
            vertices.push(Vertex::new(
                g.ox + (left + wob) * s,
                g.oy + y * s,
                0.0,
                u0,
                v,
                c,
            ));
            vertices.push(Vertex::new(
                g.ox + (left + w + wob) * s,
                g.oy + y * s,
                0.0,
                u1,
                v,
                c,
            ));
        }
        let mut indices = Vec::with_capacity(rows * 6);
        for j in 0..rows {
            let b = (j * 2) as u16;
            indices.extend([b, b + 1, b + 2, b + 1, b + 3, b + 2]);
        }
        draw_mesh(&Mesh {
            vertices,
            indices,
            texture: Some(tex),
        });
    }

    fn draw_fighter(&self, g: &mut Gfx, m: &Match, f: &Fighter, fi: usize, shx: f32, shy: f32) {
        if f.has_assert(ume_engine::flags::asf::INVISIBLE) {
            return;
        }
        if let Some(ai) = &f.after_image {
            let shown = ai.params.shown(ai.trail.len(), ai.length);
            for (k, &i) in shown.iter().enumerate().rev() {
                let t = &ai.trail[i];
                let fr = AnimFrame {
                    group: t.group,
                    index: t.index,
                    x: t.fx,
                    y: t.fy,
                    time: 1,
                    flip_h: t.flip_h,
                    flip_v: false,
                    blend: Blend::None,
                    alpha: None,
                    xscale: 1.0,
                    yscale: 1.0,
                    angle: 0.0,
                    clsn1: vec![],
                    clsn2: vec![],
                };
                let tr = ai.params.trans;
                let o = DrawOpts {
                    alpha: tr.src / 256.0,
                    add: matches!(tr.mode, BlendMode::Add | BlendMode::Add1),
                    sub: tr.mode == BlendMode::Sub,
                    pal: Some(ai.params.image_params(k)),
                    remap: &f.remap,
                    ..Default::default()
                };
                Self::draw_sprite(
                    g,
                    m,
                    &f.anim_def,
                    &fr,
                    t.x,
                    t.y,
                    t.facing,
                    f.lscl,
                    f.draw_scale,
                    &o,
                    shx,
                    shy,
                );
            }
        }
        let Some(fr) = f.frame() else { return };
        let mut frame = fr.clone();
        if let Some((_, to, _t)) = f
            .sprite_remap
            .iter()
            .find(|(from, _, t)| *t != 0 && *from == (frame.group, frame.index))
        {
            frame.group = to.0;
            frame.index = to.1;
        }
        let mut extra = f.draw_scale;
        let mut angle_extra = 0.0;
        if let Some((scale, angle, _, t)) = f.sprite_transform {
            if t != 0 {
                extra = [extra[0] * scale, extra[1] * scale];
                angle_extra = angle;
            }
        }
        let fr = &frame;
        let mut x = f.pos.x + f.draw_offset.x * f.facing;
        let hit = f.hit_shake > 0 && f.move_type == 'H';
        if hit {
            x += if f.hit_shake % 2 == 1 { 1.5 } else { -1.5 };
        }
        let mut o = DrawOpts {
            angle: if f.angle_draw { f.angle } else { 0.0 },
            tint: self.fx.ambient(f.pos.x, m.camera.x),
            ..Default::default()
        };
        if hit && f.hit_shake % 4 < 2 && !f.ghv.guarded && g.fx_gain > 0.2 {
            o.flash = 0.55;
        }
        o.pal = Some(m.fighter_pal(fi));
        o.remap = &f.remap;
        if let Some(t) = &f.trans {
            match t.mode {
                BlendMode::Add | BlendMode::Add1 => {
                    o.add = true;
                    o.alpha = t.src / 256.0;
                }
                BlendMode::Sub => {
                    o.sub = true;
                    o.alpha = t.src / 256.0;
                }
                BlendMode::AddAlpha => {
                    o.alpha = t.src / 256.0;
                }
                BlendMode::None => {}
            }
        }
        // awakened form: warm tint + pulsing halo in the form colour
        if (f.form > 0 || f.def.boss) && f.alive && g.fx_gain > 0.0 {
            let c = crate::vfx::Vfx::form_color(f);
            let amb = if f.form > 0 { 1.0 } else { 0.55 };
            let k = 0.3 * amb;
            o.tint = Color::new(
                1.0 + (c[0] - 1.0) * k,
                1.0 + (c[1] - 1.0) * k,
                1.0 + (c[2] - 1.0) * k,
                1.0,
            );
            let pulse = (0.38 + 0.14 * (m.game_time as f32 * 0.3).sin()) * amb;
            for (dx, dy) in [(-1.8, 0.0), (1.8, 0.0), (0.0, -1.8), (0.0, 1.8)] {
                Self::draw_sprite(
                    g,
                    m,
                    &f.anim_def,
                    fr,
                    x + dx,
                    f.pos.y + dy,
                    f.facing,
                    f.lscl,
                    extra,
                    &DrawOpts {
                        angle: angle_extra,
                        add: true,
                        alpha: pulse,
                        tint: Color::new(c[0], c[1], c[2], 1.0),
                        ..Default::default()
                    },
                    shx,
                    shy,
                );
            }
        }
        // the fallen fighter loses its colour once the round is decided (the match-deciding defeat even more)
        if !f.alive && m.round_state >= 3 {
            let k = if f.state_no == 5193 { 0.52 } else { 0.76 };
            o.tint = Color::new(k, k, k * 1.06, 1.0);
        }
        // owner of an active super: pulsing golden outline glow (sprite re-drawn additively around itself)
        if m.pause
            .as_ref()
            .is_some_and(|p| p.is_super && (p.by == f.id || p.by == f.root))
        {
            let pulse = 0.55 + 0.35 * (m.game_time as f32 * 0.35).sin();
            for (dx, dy) in [
                (-1.6, 0.0),
                (1.6, 0.0),
                (0.0, -1.6),
                (0.0, 1.6),
                (-1.2, -1.2),
                (1.2, 1.2),
                (-1.2, 1.2),
                (1.2, -1.2),
            ] {
                Self::draw_sprite(
                    g,
                    m,
                    &f.anim_def,
                    fr,
                    x + dx,
                    f.pos.y + dy,
                    f.facing,
                    f.lscl,
                    extra,
                    &DrawOpts {
                        angle: angle_extra,
                        add: true,
                        alpha: pulse * 0.28,
                        tint: Color::new(1.0, 0.8, 0.3, 1.0),
                        ..Default::default()
                    },
                    shx,
                    shy,
                );
            }
        }
        let sq = self.squash.get(fi).copied().unwrap_or([1.0, 1.0]);
        let ds = [extra[0] * sq[0], extra[1] * sq[1]];
        o.angle += angle_extra;
        Self::draw_sprite(
            g,
            m,
            &f.anim_def,
            fr,
            x,
            f.pos.y + f.draw_offset.y,
            f.facing,
            f.lscl,
            ds,
            &o,
            shx,
            shy,
        );
    }

    fn draw_stage(&mut self, g: &mut Gfx, m: &Match, st: &Stage, front: bool, shx: f32, shy: f32) {
        if m.global_asserts
            & if front {
                ume_engine::flags::gsf::NOFG
            } else {
                ume_engine::flags::gsf::NOBG
            }
            != 0
        {
            return;
        }
        let s = g.view_scale;
        match &st.art {
            StageArt::Procedural { back, tint, .. } => {
                let animated = self.fx.active();
                let cx = Ctx::new(g, m, back, shx, shy);
                let draw_layer = |g: &mut Gfx, l: &crate::stages::Layer| {
                    if l.dynamic {
                        return;
                    }
                    // zoomed-out field (big teams): layers scale about the ground line so the backdrop still fills the wider view
                    let z = m.zoom;
                    let x = m.view_w / 2.0 - l.w * z / 2.0 - m.camera.x * l.delta * z + shx;
                    let mut y =
                        m.stage.zoffset + (l.y + l.trim - m.camera.y * l.dy - 205.0) * z + shy;
                    let mut h = (l.h - l.trim) * z;
                    // the sky (first layer) stretches up to the top edge
                    if std::ptr::eq(l, &back[0]) && y > 0.0 {
                        h += y;
                        y = 0.0;
                    }
                    let t = g.texture(&l.image, false);
                    draw_texture_ex(
                        &t,
                        g.ox + x * s,
                        g.oy + y * s,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(l.w * z * s, h * s)),
                            ..Default::default()
                        },
                    );
                };
                if !front {
                    for (i, l) in back.iter().enumerate().filter(|(_, l)| !l.front) {
                        draw_layer(g, l);
                        if animated {
                            self.fx.draw_layer(i, &cx, g, m);
                        }
                    }
                } else {
                    let w = m.view_w + 40.0;
                    for p in self.particles.iter_mut() {
                        if m.pause.is_none() {
                            p.x += p.vx;
                            p.y += p.vy;
                            if p.y > m.view_h + 5.0 {
                                p.y = -5.0;
                            }
                            if p.y < -10.0 {
                                p.y = m.view_h + 5.0;
                            }
                        }
                        let px = ((p.x - m.camera.x * 0.9) % w + w) % w - 20.0;
                        draw_circle(
                            g.ox + px * s,
                            g.oy + p.y * s,
                            p.s * s,
                            Color::from_rgba(p.c[0], p.c[1], p.c[2], (p.a * 255.0) as u8),
                        );
                    }
                    for l in back.iter().filter(|l| l.front) {
                        draw_layer(g, l);
                    }
                    if animated {
                        self.fx.draw_front(&cx, g);
                    }
                    if let Some(t) = tint {
                        draw_rectangle(
                            g.ox,
                            g.oy,
                            m.view_w * s,
                            m.view_h * s,
                            Color::new(t[0], t[1], t[2], t[3]),
                        );
                    }
                }
            }
            StageArt::Illustrated { image } => {
                if front {
                    return;
                }
                let ground = m.stage.zoffset - m.camera.y;
                let top_ground = m.stage.zoffset - m.stage.bound_high.min(0.0);
                let bottom_ground = m.stage.zoffset - m.stage.bound_low.max(0.0);
                let h = (top_ground / 0.85)
                    .max((m.view_h - bottom_ground) / 0.15)
                    .max(m.view_h);
                let w = (h * image.w as f32 / image.h as f32).max(m.view_w);
                let x =
                    ((m.view_w - w) * 0.5 - m.camera.x * 0.12 * m.zoom).clamp(m.view_w - w, 0.0);
                let y = ground - h * 0.85;
                let texture = g.texture(image, true);
                draw_texture_ex(
                    &texture,
                    g.ox + (x + shx) * s,
                    g.oy + (y + shy) * s,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(w * s, h * s)),
                        ..Default::default()
                    },
                );
            }
            StageArt::Mugen(bg) => {
                let scl = bg.scl;
                let scl_y = bg.scl_y;
                let (cx, cy) = (m.camera.x / scl, m.camera.y / scl_y);
                let state = m.bg.as_ref();
                for (k, l) in bg.stage.layers.iter().enumerate() {
                    if (l.layer == 1) != front {
                        continue;
                    }
                    if state.is_some_and(|st| !st.visible(k)) {
                        continue;
                    }
                    let key = match state {
                        Some(st) => st.sprite(k),
                        None => l.sprite,
                    };
                    let (Some(img), Some(meta)) = (bg.images.get(&key), bg.stage.sprites.get(&key))
                    else {
                        continue;
                    };
                    let t = g.texture(img, true);
                    let off = match state {
                        Some(st) => st.offset(k),
                        None => {
                            let gt = m.game_time as f32;
                            [l.velocity[0] * gt, l.velocity[1] * gt]
                        }
                    };
                    let layer_scale = l.scale_at([cx, cy]);
                    let (lx, ly) = (scl * layer_scale[0], scl_y * layer_scale[1]);
                    let sxp = (l.start[0] + off[0] - cx * l.delta[0]) * scl + m.view_w / 2.0 + shx;
                    let syp = (l.start[1] + off[1] - cy * l.delta[1]) * scl_y
                        + (m.view_h - 240.0) / 2.0
                        + shy;
                    let (w, h) = (meta.w as f32 * lx, meta.h as f32 * ly);
                    let (axis_dx, axis_dy) = (meta.axis_x as f32 * lx, meta.axis_y as f32 * ly);
                    let (x0, y0) = (sxp - axis_dx, syp - axis_dy);
                    let (stx, sty) = (w + l.tile_spacing[0] * scl, h + l.tile_spacing[1] * scl_y);
                    let xs: Vec<f32> = match l.tile[0] {
                        1 => {
                            let st = x0 - (x0 / stx).ceil() * stx;
                            let mut v = vec![];
                            let mut x = st;
                            while x < m.view_w {
                                v.push(x);
                                x += stx;
                            }
                            v
                        }
                        n if n > 1 => (0..n).map(|i| x0 + i as f32 * stx).collect(),
                        _ => vec![x0],
                    };
                    let ys: Vec<f32> = match l.tile[1] {
                        1 => {
                            let st = y0 - (y0 / sty).ceil() * sty;
                            let mut v = vec![];
                            let mut y = st;
                            while y < m.view_h {
                                v.push(y);
                                y += sty;
                            }
                            v
                        }
                        n if n > 1 => (0..n).map(|i| y0 + i as f32 * sty).collect(),
                        _ => vec![y0],
                    };
                    let add = l.trans.starts_with("add") && l.trans != "addalpha";
                    let sub = l.trans == "sub";
                    let alpha = if l.trans == "addalpha" {
                        (l.alpha[0] / 256.0).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    if add {
                        set_blend(GfxBlend::Add);
                    } else if sub {
                        set_blend(GfxBlend::Sub);
                    }
                    let clip = l.clip_window();
                    if let Some(win) = clip {
                        let scroll = l.window_scroll();
                        let wx = (win[0] - cx * scroll[0]) * scl + m.view_w / 2.0 + shx;
                        let wy = (win[1] - cy * scroll[1]) * scl_y + (m.view_h - 240.0) / 2.0 + shy;
                        let (ww, wh) = (
                            (win[2] - win[0] + 1.0) * scl,
                            (win[3] - win[1] + 1.0) * scl_y,
                        );
                        set_scissor(Some((
                            (g.ox + wx * s) as i32,
                            (g.oy + wy * s) as i32,
                            (ww * s).max(0.0) as i32,
                            (wh * s).max(0.0) as i32,
                        )));
                    }
                    let tint = Color::new(1.0, 1.0, 1.0, alpha);
                    let transformed = l.angle != 0.0 || l.x_shear != 0.0;
                    for &x in &xs {
                        for &y in &ys {
                            if transformed {
                                draw_transformed_layer_quad(
                                    &t,
                                    [g.ox + (x + axis_dx) * s, g.oy + (y + axis_dy) * s],
                                    [axis_dx * s, axis_dy * s],
                                    [w * s, h * s],
                                    l.angle,
                                    l.x_shear,
                                    tint,
                                );
                            } else {
                                draw_texture_ex(
                                    &t,
                                    g.ox + x * s,
                                    g.oy + y * s,
                                    tint,
                                    DrawTextureParams {
                                        dest_size: Some(vec2(w * s + 0.5, h * s + 0.5)),
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                    }
                    if clip.is_some() {
                        set_scissor(None);
                    }
                    if add || sub {
                        set_blend(GfxBlend::Alpha);
                    }
                }
            }
        }
    }

    fn draw_debug(&self, g: &Gfx, m: &Match) {
        let s = g.view_scale;
        let bx = |b: &[f32; 4], c: Color| {
            draw_rectangle_lines(
                g.ox + Self::sx(m, b[0]) * s,
                g.oy + Self::sy(m, b[1]) * s,
                (b[2] - b[0]) * s,
                (b[3] - b[1]) * s,
                2.0,
                c,
            )
        };
        for f in &m.fighters {
            for b in f.boxes(2) {
                bx(&b, Color::new(0.2, 0.6, 1.0, 1.0));
            }
            for b in f.boxes(1) {
                bx(&b, RED);
            }
            draw_text(
                &format!(
                    "{} {}{} a{} t{}",
                    f.state_no, f.state_type, f.move_type, f.anim_no, f.state_time
                ),
                g.ox + (Self::sx(m, f.pos.x) - 20.0) * s,
                g.oy + (Self::sy(m, f.pos.y) + 10.0) * s,
                4.0 * s,
                YELLOW,
            );
        }
        for p in &m.projectiles {
            for b in Match::proj_boxes(p, 1) {
                bx(&b, ORANGE);
            }
        }
    }
}
