use crate::gfx::*;
use crate::render::Gfx;
use crate::ui::hex;
use ume_engine::matchs::{Match, ARENA_HALF_WIDTH, ARENA_PLATFORMS};

thread_local! {
    static BACKDROP: Option<Texture2D> = load_backdrop();
}

fn load_backdrop() -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(include_bytes!("../assets/stages/power_arena.png")).ok()?;
    let texture = Texture2D::from_rgba8(image.width as u16, image.height as u16, &image.rgba?);
    texture.set_filter(FilterMode::Nearest);
    Some(texture)
}

pub(crate) fn draw(g: &Gfx, battle: &Match, shx: f32, shy: f32) {
    let scale = g.view_scale;
    BACKDROP.with(|backdrop| {
        if let Some(texture) = backdrop {
            draw_texture_ex(texture, g.ox, g.oy, WHITE, DrawTextureParams {
                dest_size: Some(vec2(battle.view_w * scale, battle.view_h * scale)),
                ..Default::default()
            });
        }
    });
    let x = |world: f32| g.ox + (world - battle.camera.x + battle.view_w / 2.0 + shx) * scale;
    let y = |world: f32| g.oy + (world - battle.camera.y + battle.stage.zoffset + shy) * scale;
    for (left, right, top) in [(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH, 0.0)].into_iter().chain(ARENA_PLATFORMS) {
        let depth = if top == 0.0 { 38.0 } else { 14.0 };
        let width = (right - left) * scale;
        draw_rectangle(x(left), y(top), width, depth * scale, hex("#363654"));
        draw_rectangle(x(left), y(top), width, 3.0 * scale, hex("#a5f0f3"));
        draw_rectangle(x(left), y(top + depth - 3.0), width, 3.0 * scale, hex("#19192d"));
        let mut seam = left + 24.0;
        while seam < right {
            draw_rectangle(x(seam), y(top + 3.0), scale, (depth - 6.0) * scale, hex("#525572"));
            seam += 24.0;
        }
        if top == 0.0 {
            for edge in [left, right - 8.0] {
                draw_rectangle(x(edge), y(top), 8.0 * scale, depth * scale, hex("#e69a58"));
            }
        }
    }
}
