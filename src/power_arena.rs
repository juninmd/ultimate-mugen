use crate::gfx::*;
use crate::render::Gfx;
use crate::ui::hex;
use crate::ui::{alpha, Ui, VW};
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
        let reach = battle.view_w / 2.0 + 40.0;
        let (first, last) = (battle.camera.x - reach, battle.camera.x + reach);
        let sections = (battle.arena_half / ARENA_HALF_WIDTH * 8.0) as i32;
        let base_width = battle.arena_half * 2.0 / sections as f32;
        for section in 0..sections {
            let left = -battle.arena_half + section as f32 * base_width;
            if left + base_width >= first && left <= last {
                draw_platform(platform.as_ref(), x(left), y(0.0), base_width * scale, 30.0 * scale);
            }
        }
        let period = ARENA_HALF_WIDTH * 2.0;
        let tiles = (battle.arena_half / ARENA_HALF_WIDTH).round() as i32;
        for tile in 0..tiles {
            let shift = (tile * 2 + 1 - tiles) as f32 * ARENA_HALF_WIDTH;
            if shift + period / 2.0 < first || shift - period / 2.0 > last {
                continue;
            }
            for (left, right, top) in ARENA_PLATFORMS {
                let depth = ((right - left) * 0.13).clamp(15.0, 22.0);
                let screen_y = y(top);
                if screen_y + depth * scale >= g.oy && screen_y <= g.oy + battle.view_h * scale {
                    draw_platform(platform.as_ref(), x(left + shift), screen_y, (right - left) * scale, depth * scale);
                }
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

const MAP_W: f32 = 440.0;
const MAP_H: f32 = 58.0;
const ROOF: f32 = -420.0;

pub(crate) fn minimap(ui: &Ui, battle: &Match, human: &[u32]) {
    if battle.swarm.is_none() {
        return;
    }
    let span = battle.arena_half * 2.0;
    if span <= battle.view_w * 1.1 {
        return;
    }
    let (left, top) = ((VW - MAP_W) / 2.0, 146.0);
    ui.rect(left - 4.0, top - 4.0, MAP_W + 8.0, MAP_H + 8.0, Color::new(0.02, 0.03, 0.08, 0.72));
    ui.rect(left, top + MAP_H - 3.0, MAP_W, 3.0, alpha(hex("#a5f0f3"), 0.5));
    let at = |world: f32| left + ((world + battle.arena_half) / span).clamp(0.0, 1.0) * MAP_W;
    let row = |world_y: f32| top + 4.0 + ((world_y - ROOF) / -ROOF).clamp(0.0, 1.0) * (MAP_H - 10.0);
    let view_l = at(battle.camera.x - battle.view_w / 2.0);
    let view_r = at(battle.camera.x + battle.view_w / 2.0);
    ui.rect_lines(view_l, top, (view_r - view_l).max(3.0), MAP_H, 2.0, hex("#ffcc33"));
    for fighter in battle.roots().filter(|f| f.alive && !f.destroyed) {
        let foe = fighter.side != 0;
        let you = human.contains(&fighter.id);
        let (size, color) = match (foe, you) {
            (true, _) => (4.0, hex("#ff4d6d")),
            (false, true) => (8.0, hex("#ffffff")),
            (false, false) => (6.0, hex("#22e8ff")),
        };
        ui.rect(at(fighter.pos.x) - size / 2.0, row(fighter.pos.y) - size / 2.0, size, size, color);
    }
}
