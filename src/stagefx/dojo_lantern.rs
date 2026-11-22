use crate::gfx::*;
use std::cell::OnceCell;

const PNG: &[u8] = include_bytes!("../../assets/props/dojo_lantern.png");

thread_local! {
    static TEXTURE: OnceCell<Option<Texture2D>> = const { OnceCell::new() };
}

fn load() -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(PNG).ok()?;
    let rgba = image.rgba?;
    let texture = Texture2D::from_rgba8(image.width as u16, image.height as u16, &rgba);
    texture.set_filter(FilterMode::Nearest);
    Some(texture)
}

pub(super) fn draw(center: Vec2, width: f32, height: f32, tint: Color) -> bool {
    TEXTURE.with(|slot| {
        let Some(texture) = slot.get_or_init(load).as_ref() else {
            return false;
        };
        draw_texture_ex(
            texture,
            center.x - width * 0.5,
            center.y - height * 0.5,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                ..Default::default()
            },
        );
        true
    })
}
