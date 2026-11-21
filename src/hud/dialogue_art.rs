use crate::gfx::*;
use std::sync::Arc;

const SLICE: f32 = 6.0;
const DEST_EDGE: f32 = 3.5;
const IMAGE_SIZE: f32 = 32.0;

thread_local! {
    static FRAME: Option<Texture2D> = load_frame();
}

fn load_frame() -> Option<Texture2D> {
    let image =
        ume_formats::png::decode_png(include_bytes!("../../assets/ui/hud-faceoff-frame.png"))
            .ok()?;
    let rgba = image.rgba?;
    let texture =
        Texture2D::from_rgba_shared(image.width as u16, image.height as u16, Arc::new(rgba));
    texture.set_filter(FilterMode::Nearest);
    Some(texture)
}

pub(super) fn draw_frame<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    scale: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    alpha: f32,
) -> bool {
    FRAME.with(|frame| {
        let Some(texture) = frame else {
            return false;
        };
        let source = [0.0, SLICE, IMAGE_SIZE - SLICE, IMAGE_SIZE];
        let inset = DEST_EDGE.min(width * 0.5).min(height * 0.5);
        let dest_x = [x, x + inset, x + width - inset, x + width];
        let dest_y = [y, y + inset, y + height - inset, y + height];
        for row in 0..3 {
            for col in 0..3 {
                let top_left = p(dest_x[col], dest_y[row]);
                draw_texture_ex(
                    texture,
                    top_left.x,
                    top_left.y,
                    Color::new(1.0, 1.0, 1.0, alpha),
                    DrawTextureParams {
                        source: Some(Rect::new(
                            source[col],
                            source[row],
                            source[col + 1] - source[col],
                            source[row + 1] - source[row],
                        )),
                        dest_size: Some(vec2(
                            (dest_x[col + 1] - dest_x[col]) * scale,
                            (dest_y[row + 1] - dest_y[row]) * scale,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
        true
    })
}
