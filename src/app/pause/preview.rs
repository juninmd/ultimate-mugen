use super::draw_chrome::{INK, MUTED};
use super::labels::cost_label;
use super::layout::{Layout, Rect};
use super::skin::{clip, fade, glass, pill, unclip};
use crate::app::*;
use crate::ui::hex;
use ume_engine::chardef::{AnimFrame, CharDef, MoveInfo};
use ume_engine::move_preview::{frame_at, preview_anim};

fn extent(def: &CharDef, fr: &AnimFrame) -> Option<[f32; 4]> {
    let spr = def.sprites.get(&(fr.group, fr.index))?;
    let (kx, ky) = (spr.scale * def.scale_x * fr.xscale, spr.scale * def.scale_y * fr.yscale);
    let (w, h) = (spr.width * kx, spr.height * ky);
    let left = if fr.flip_h { -(spr.width - spr.axis_x) * kx } else { -spr.axis_x * kx };
    let x0 = fr.x * def.scale_x + left;
    let y0 = fr.y * def.scale_y - spr.axis_y * ky;
    Some([x0, y0, x0 + w, y0 + h])
}

fn union(def: &CharDef, frames: &[AnimFrame]) -> Option<[f32; 4]> {
    frames.iter().filter_map(|f| extent(def, f)).reduce(|a, b| [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])])
}

impl App {
    pub(super) fn pause_move_preview(&mut self, lay: &Layout, def: &CharDef, mv: Option<&MoveInfo>, a: f32) {
        let card = lay.preview;
        let f = lay.font;
        glass(&self.ui, card, 14.0, [0.1, 0.2, 0.5], a);
        let ui = &self.ui;
        let heading = fade(hex(MUTED), a);
        ui.text(crate::i18n::tr3("PREVIA", "PREVIEW", "VISTA PREVIA"), card.x + 18.0, card.y + 26.0 * f, 14.0 * f, heading, Align::Left, true);
        let Some(mv) = mv else { return };
        let idx = self.pause.move_cur;
        if self.pause.preview.0 != idx {
            self.pause.preview = (idx, 0.0);
        }
        let stage = Rect::new(card.x + 12.0, card.y + 40.0 * f, card.w - 24.0, card.h - 40.0 * f - 78.0 * f);
        pill(ui, stage, 10.0, fade(Color::new(0.0, 0.0, 0.0, 0.34), a), fade(Color::new(0.0, 0.0, 0.0, 0.18), a), Some(fade(Color::new(1.0, 1.0, 1.0, 0.14), a)));
        let anim = preview_anim(def, mv);
        match anim {
            Some(anim) => {
                let ticks = (self.pause.preview.1 * 60.0) as u32;
                let fr = &anim.frames[frame_at(anim, ticks)];
                self.draw_preview_frame(def, &anim.frames, fr, stage, a);
            }
            None => {
                ui.text(crate::i18n::tr3("Sem previa", "No preview", "Sin vista previa"), stage.x + stage.w / 2.0, stage.y + stage.h / 2.0, 16.0 * f, heading, Align::Center, false);
            }
        }
        let ui = &self.ui;
        let name = mv.name.as_str();
        ui.text(name, card.x + 18.0, card.bottom() - 40.0 * f, 18.0 * f, fade(hex(INK), a), Align::Left, false);
        let known = anim.is_some();
        let Some(cost) = cost_label(mv).or_else(|| known.then(|| crate::i18n::tr3("SEM CUSTO", "FREE", "GRATIS").to_string())) else { return };
        let tw = ui.measure(&cost, 15.0 * f, true) + 22.0;
        let pr = Rect::new(card.x + 18.0, card.bottom() - 32.0 * f, tw, 24.0 * f);
        let (ink, wash) = super::glide::cost_colors(&cost);
        pill(ui, pr, 12.0, fade(wash, a), fade(Color::new(0.0, 0.0, 0.0, 0.35), a), Some(fade(ink, 0.8 * a)));
        ui.text(&cost, pr.x + tw / 2.0, pr.y + 17.0 * f, 15.0 * f, fade(ink, a), Align::Center, true);
    }

    fn draw_preview_frame(&mut self, def: &CharDef, frames: &[AnimFrame], fr: &AnimFrame, stage: Rect, a: f32) {
        let Some(b) = union(def, frames) else { return };
        let Some(spr) = def.sprites.get(&(fr.group, fr.index)) else { return };
        let (bw, bh) = ((b[2] - b[0]).max(1.0), (b[3] - b[1]).max(1.0));
        let pad = 12.0;
        let fit = ((stage.w - pad * 2.0) / bw).min((stage.h - pad * 2.0) / bh).min(1.6);
        let (cx, cy) = (stage.x + stage.w / 2.0 - (b[0] + b[2]) / 2.0 * fit, stage.y + stage.h / 2.0 - (b[1] + b[3]) / 2.0 * fit);
        let Some(tex) = self.g.sprite_tex_for(def, (fr.group, fr.index), spr, &[]) else { return };
        let s = self.ui.s;
        let (kx, ky) = (spr.scale * def.scale_x * fr.xscale * fit, spr.scale * def.scale_y * fr.yscale * fit);
        let left = if fr.flip_h { -(spr.width - spr.axis_x) * kx } else { -spr.axis_x * kx };
        let x = cx + fr.x * def.scale_x * fit + left;
        let y = cy + fr.y * def.scale_y * fit - spr.axis_y * ky;
        let params = DrawTextureParams {
            dest_size: Some(vec2(spr.width * kx * s, spr.height * ky * s)),
            flip_x: fr.flip_h,
            flip_y: fr.flip_v,
            ..Default::default()
        };
        clip(&self.ui, stage);
        tex.draw(self.ui.x(x), self.ui.y(y), Color::new(1.0, 1.0, 1.0, a), params);
        unclip();
    }
}

impl Layout {
    pub fn with_preview(&self) -> Layout {
        let w = if self.compact { 300.0 } else { 290.0 };
        let gap = 14.0;
        let list = Rect::new(self.list.x, self.list.y, self.list.w - w - gap, self.list.h);
        let preview = Rect::new(list.right() + gap, self.list.y, w, self.list.h);
        Layout { list, preview, ..self.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SCALES: [f32; 5] = [0.4, 0.625, 0.73, 1.0, 2.0];

    #[test]
    fn preview_card_sits_beside_the_list_without_overlap() {
        for s in SCALES {
            let l = Layout::new(s).with_preview();
            assert!(!l.list.overlaps(&l.preview) && l.preview.right() <= l.panel.right(), "{s}");
            assert!(l.preview.w >= 280.0 && l.preview.h == l.list.h, "{s}");
        }
    }
}
