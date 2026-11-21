use std::sync::Arc;

use super::*;

const CELL: f32 = 48.0;
const DRAW_SIZE: f32 = 28.0;

pub(super) const ICON_OF: [usize; 13] = [0, 1, 4, 2, 5, 8, 9, 13, 10, 11, 7, 3, 15];

thread_local! {
    static SHEET: Option<Texture2D> = load_sheet();
}

fn load_sheet() -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(include_bytes!("../../assets/ui/menu-icons.png")).ok()?;
    let rgba = match image.rgba {
        Some(rgba) => rgba,
        None => ume_formats::png::png_indexed_to_rgba(&image),
    };
    let tex = Texture2D::from_rgba_shared(image.width as u16, image.height as u16, Arc::new(rgba));
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

impl App {
    pub(super) fn menu_icon(&self, index: usize, cx: f32, cy: f32, color: Color) {
        let cell = index.min(15);
        let col = (cell % 4) as f32;
        let row = (cell / 4) as f32;
        let sz = DRAW_SIZE * self.ui.s;
        let x = self.ui.x(cx) - sz * 0.5;
        let y = self.ui.y(cy) - sz * 0.5;
        let tint = Color::new(
            0.78 + color.r * 0.22,
            0.78 + color.g * 0.22,
            0.78 + color.b * 0.22,
            color.a,
        );
        SHEET.with(|sheet| {
            let Some(tex) = sheet else {
                return;
            };
            draw_texture_ex(
                tex,
                x,
                y,
                tint,
                DrawTextureParams {
                    source: Some(Rect::new(col * CELL, row * CELL, CELL, CELL)),
                    dest_size: Some(vec2(sz, sz)),
                    ..Default::default()
                },
            );
        });
    }
}
