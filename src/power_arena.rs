use crate::gfx::*;
use crate::render::Gfx;
use crate::ui::hex;
use ume_engine::matchs::{Match, ARENA_HALF_WIDTH, ARENA_PLATFORMS};

thread_local! {
    static BACKDROP: Option<Texture2D> = load_texture(include_bytes!("../assets/stages/power_arena.png"));
    static PLATFORM: Option<Texture2D> = load_texture(include_bytes!("../assets/stages/power_arena_platform.png"));
}

fn load_texture(bytes: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(bytes).ok()?;
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
    PLATFORM.with(|platform| {
        let sections = 8;
        let base_width = ARENA_HALF_WIDTH * 2.0 / sections as f32;
        for section in 0..sections {
            let left = -ARENA_HALF_WIDTH + section as f32 * base_width;
            draw_platform(platform.as_ref(), x(left), y(0.0), base_width * scale, 30.0 * scale);
        }
        for (left, right, top) in ARENA_PLATFORMS {
            let depth = ((right - left) * 0.13).clamp(15.0, 22.0);
            let screen_y = y(top);
            if screen_y + depth * scale >= g.oy && screen_y <= g.oy + battle.view_h * scale {
                draw_platform(platform.as_ref(), x(left), screen_y, (right - left) * scale, depth * scale);
            }
        }
    });
}

fn draw_platform(texture: Option<&Texture2D>, x: f32, y: f32, width: f32, depth: f32) {
    if let Some(texture) = texture {
        draw_texture_ex(texture, x, y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(width, depth)),
            ..Default::default()
        });
    } else {
        draw_rectangle(x, y, width, depth, hex("#363654"));
        draw_rectangle(x, y, width, depth * 0.15, hex("#a5f0f3"));
    }
}
