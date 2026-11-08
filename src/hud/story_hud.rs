use crate::gfx::*;
use crate::render::Gfx;
use crate::ui::{hex, Ui};
use ume_engine::fighter::Fighter;
use ume_engine::matchs::BASE_W;

const PANEL_HEIGHT: f32 = 21.0;
const MARGIN: f32 = 6.0;

pub(super) fn draw_boss_bar(ui: &Ui, g: &Gfx, boss: &Fighter) {
    let s = g.view_scale;
    let x = BASE_W / 2.0 + 24.0;
    let width = BASE_W / 2.0 - 24.0 - MARGIN;
    let y = 5.0;
    let at = |px: f32, py: f32| vec2(g.ox + px * s, g.oy + py * s);
    let rect = |px: f32, py: f32, w: f32, h: f32, c: Color| {
        let p = at(px, py);
        draw_rectangle(p.x, p.y, w * s, h * s, c);
    };
    let label = |t: &str, px: f32, py: f32, size: f32, c: Color, right: bool| {
        let font_size = (size * s).max(9.0) as u16;
        let dim = measure_text(t, Some(&ui.display), font_size, 1.0);
        let p = at(px, py);
        let left = if right { p.x - dim.width } else { p.x };
        draw_text_ex(t, left + s * 0.5, p.y + s * 0.5, TextParams { font: Some(&ui.display), font_size, color: Color::new(0.05, 0.0, 0.08, 0.9), ..Default::default() });
        draw_text_ex(t, left, p.y, TextParams { font: Some(&ui.display), font_size, color: c, ..Default::default() });
    };
    rect(x - 2.0, y - 2.0, width + 4.0, PANEL_HEIGHT + 4.0, Color::new(0.04, 0.0, 0.06, 0.82));
    rect(x - 2.0, y - 2.0, width + 4.0, 1.2, hex("#ff4d6d"));
    label(&crate::i18n::tr("VILÃO DA FASE", "STAGE VILLAIN").to_string(), x + 3.0, y + 7.5, 6.0, hex("#ff9aa9"), false);
    label(&boss.def.display_name.to_uppercase(), x + width - 3.0, y + 7.5, 7.0, hex("#ffffff"), true);
    let ratio = (boss.life / boss.life_max.max(1.0)).clamp(0.0, 1.0);
    let (bx, by, bw, bh) = (x + 3.0, y + 11.5, width - 6.0, 6.5);
    rect(bx - 1.0, by - 1.0, bw + 2.0, bh + 2.0, Color::new(0.0, 0.0, 0.0, 0.9));
    rect(bx, by, bw, bh, Color::new(0.22, 0.04, 0.08, 1.0));
    let fill = if ratio < 0.25 { hex("#ffb02e") } else { hex("#ff3d5a") };
    rect(bx, by, bw * ratio, bh, fill);
    rect(bx, by, bw * ratio, bh * 0.4, Color::new(1.0, 1.0, 1.0, 0.28));
}
