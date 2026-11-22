use crate::gfx::*;
use crate::render::Gfx;
use crate::ui::{alpha, hex};
use ume_engine::matchs::Match;

const FLOAT_HEIGHT: f32 = 24.0;
const SPARKS: usize = 5;

pub(crate) fn draw(g: &Gfx, battle: &Match, shx: f32, shy: f32) {
    let Some(elixir) = battle.elixir.as_ref().filter(|e| !e.taken()) else { return };
    let scale = g.view_scale;
    let sx = g.ox + (elixir.x - battle.camera.x + battle.view_w / 2.0 + shx) * scale;
    let ground = g.oy + (elixir.y - battle.camera.y + battle.stage.zoffset + shy) * scale;
    if sx < g.ox - 40.0 * scale || sx > g.ox + (battle.view_w + 40.0) * scale {
        return;
    }
    let size_scale = scale * (battle.view_w / 426.0).clamp(1.0, 2.6);
    let t = battle.game_time as f32;
    let cy = ground - (FLOAT_HEIGHT + (t * 0.07).sin() * 3.0) * size_scale;
    let pulse = 0.8 + 0.2 * (t * 0.11).sin();
    for (radius, a) in [(22.0, 0.10), (15.0, 0.16), (10.0, 0.26)] {
        draw_circle(sx, cy, radius * size_scale * pulse, alpha(hex("#7dffc4"), a));
    }
    draw_rectangle(sx - 3.0 * size_scale, cy - 13.0 * size_scale, 6.0 * size_scale, 8.0 * size_scale, hex("#d9fff0"));
    draw_rectangle(sx - 4.0 * size_scale, cy - 16.0 * size_scale, 8.0 * size_scale, 3.5 * size_scale, hex("#ffcc33"));
    draw_circle(sx, cy, 8.5 * size_scale, hex("#2fe89a"));
    draw_circle(sx, cy + 1.5 * size_scale, 6.0 * size_scale, hex("#7dffc4"));
    draw_circle(sx - 3.0 * size_scale, cy - 3.0 * size_scale, 2.0 * size_scale, alpha(WHITE, 0.85));
    for k in 0..SPARKS {
        let angle = t * 0.04 + k as f32 * std::f32::consts::TAU / SPARKS as f32;
        let reach = (17.0 + 3.0 * (t * 0.09 + k as f32).sin()) * size_scale;
        let size = 2.0 * size_scale;
        draw_rectangle(sx + angle.cos() * reach - size / 2.0, cy + angle.sin() * reach * 0.8 - size / 2.0, size, size, alpha(hex("#ffe27a"), 0.9));
    }
}
