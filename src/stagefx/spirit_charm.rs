use crate::gfx::*;
use std::cell::OnceCell;

const PNG: &[u8] = include_bytes!("../../assets/props/spirit_ofuda.png");

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

pub(super) fn draw(top: Vec2, width: f32, height: f32, angle: f32) -> bool {
    TEXTURE.with(|slot| {
        let Some(texture) = slot.get_or_init(load).as_ref() else {
            return false;
        };
        draw_texture_ex(
            texture,
            top.x - width * 0.5,
            top.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                rotation: angle,
                pivot: Some(top),
                ..Default::default()
            },
        );
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_ofuda_png_decodes_as_rgba() {
        let image = ume_formats::png::decode_png(PNG).expect("Spirit charm PNG should decode");
        assert_eq!((image.width, image.height), (128, 192));
        assert!(image.rgba.is_some());
    }
}
