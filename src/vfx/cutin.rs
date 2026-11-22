use super::Vfx;
use crate::gfx::*;
use crate::poly;
use crate::render::{Gfx, SpriteTex};
use crate::ui::Ui;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

fn fighter_portrait(g: &mut Gfx, f: &Fighter) -> Option<(SpriteTex, Color)> {
    if let Some(tex) = g
        .sprite_tex(&f.def, (9000, 1), &f.remap)
        .or_else(|| g.sprite_tex(&f.def, (9000, 0), &f.remap))
    {
        return Some((tex, WHITE));
    }
    f.def.portrait_pick().map(|(sp, img, _full)| {
        (SpriteTex::Rgba(g.texture(&img, sp.scale >= 0.99)), WHITE)
    })
}

impl Vfx {
    pub fn draw_cutin(&self, ui: &Ui, g: &mut Gfx, m: &Match) {
        let t = self.cinema.cutin_t;
        if t <= 0.01 {
            return;
        }
        let Some(f) = m
            .fighters
            .iter()
            .find(|f| f.id == self.cinema.cutin_owner || f.root == self.cinema.cutin_owner)
        else {
            return;
        };
        let col = self.cinema.cutin_col;
        let dur = 0.92;
        let p = (1.0 - (t / dur)).clamp(0.0, 1.0);
        let env = if p < 0.14 {
            p / 0.14
        } else if p > 0.78 {
            ((1.0 - p) / 0.22).max(0.0)
        } else {
            1.0
        };
        let slide = if p < 0.16 {
            let u = 1.0 - p / 0.16;
            1.0 - u * u * u
        } else {
            1.0
        };
        let right = self.cinema.cutin_side != 0;
        let band_h = 104.0 * env;
        let cy = 192.0 - band_h * 0.5;
        let skew = if right { -44.0 } else { 44.0 };
        let w = 1360.0 * slide;
        let x0 = if right { 1280.0 - w } else { -40.0 };
        let to_scr = |pts: Vec<Vec2>| -> Vec<Vec2> {
            pts.into_iter()
                .map(|pt| vec2(ui.x(pt.x), ui.y(pt.y)))
                .collect()
        };
        let band = to_scr(poly::slanted(x0, cy, w, band_h, skew));
        let dark = Color::new(
            col[0] * 0.14,
            col[1] * 0.12,
            col[2] * 0.2 + 0.06,
            0.92 * env,
        );
        let bright = Color::new(col[0], col[1], col[2], 0.95 * env);
        let core = Color::new(1.0, 0.96, 0.85, 0.85 * env);
        if right {
            poly::grad_h(&band, bright, dark);
        } else {
            poly::grad_h(&band, dark, bright);
        }
        poly::outline(
            &band,
            (2.2 * ui.s).max(1.5),
            Color::new(col[0], col[1], col[2], 0.9 * env),
        );

        for i in 0..6 {
            let y_off = cy + 12.0 + i as f32 * (band_h * 0.14);
            let streak_w = 180.0 + (i as f32 * 73.0) % 240.0;
            let sx =
                ((self.clock * (1800.0 + i as f32 * 260.0) + i as f32 * 210.0) % 1500.0) - 180.0;
            let bar = to_scr(poly::slanted(sx, y_off, streak_w, 3.0 * env, skew * 0.15));
            poly::solid(&bar, Color::new(1.0, 1.0, 1.0, 0.24 * env));
        }

        if let Some((tex, tint)) = fighter_portrait(g, f) {
            let pw = 112.0 * env;
            let px = if right {
                1280.0 - 260.0 * slide
            } else {
                260.0 * slide - pw
            };
            tex.draw(
                ui.x(px),
                ui.y(cy + 4.0),
                Color::new(tint.r, tint.g, tint.b, env),
                DrawTextureParams {
                    dest_size: Some(vec2(pw * ui.s, (band_h - 8.0).max(1.0) * ui.s)),
                    flip_x: right,
                    ..Default::default()
                },
            );
        }

        let tx = if right {
            760.0 - 90.0 * (1.0 - slide)
        } else {
            320.0 + 90.0 * (1.0 - slide)
        };
        let align = if right {
            crate::ui::Align::Right
        } else {
            crate::ui::Align::Left
        };
        ui.text_bold(
            &f.def.display_name.to_uppercase(),
            tx,
            cy + band_h * 0.56,
            34.0 * env.max(0.5),
            core,
            Color::new(0.05, 0.02, 0.08, env),
            align,
        );
        ui.text(
            crate::i18n::tr3("ARTE SUPREMA", "SUPER ART", "ARTE SUPREMA"),
            tx,
            cy + band_h * 0.84,
            15.0 * env.max(0.5),
            Color::new(1.0, 1.0, 1.0, 0.88 * env),
            align,
            false,
        );
    }
}
