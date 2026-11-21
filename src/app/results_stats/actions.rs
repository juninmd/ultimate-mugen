use super::*;
use std::sync::Arc;

thread_local! {
    static ICONS: [Option<Texture2D>; 4] = [
        texture(include_bytes!("../../../assets/ui/results/rematch.png")),
        texture(include_bytes!("../../../assets/ui/results/replay.png")),
        texture(include_bytes!("../../../assets/ui/results/fighters.png")),
        texture(include_bytes!("../../../assets/ui/results/menu.png")),
    ];
}

fn texture(bytes: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(bytes).ok()?;
    Some(Texture2D::from_rgba_shared(
        image.width as u16,
        image.height as u16,
        Arc::new(image.rgba?),
    ))
}

pub(super) fn draw_label(
    ui: &Ui,
    icon: usize,
    label: &str,
    (x, y, w): (f32, f32, f32),
    opacity: f32,
) {
    const TEXT: f32 = 20.0;
    const ICON: f32 = 23.0;
    const GAP: f32 = 5.0;
    let center = x + w * 0.5;
    let color = Color::new(1.0, 1.0, 1.0, opacity);
    let width_at = |size: f32| {
        measure_text(label, Some(&ui.display), (size * ui.s).max(9.0) as u16, 1.0).width / ui.s
    };
    let available = (w - ICON - GAP - 12.0).max(0.0);
    let full_width = width_at(TEXT);
    let size = (TEXT * available / full_width.max(1.0)).clamp(16.0, TEXT);
    let width = width_at(size);
    let with_icon = ICONS.with(|icons| {
        let Some(texture) = icons.get(icon).and_then(Option::as_ref) else {
            return false;
        };
        if width > available {
            return false;
        }
        draw_texture_ex(
            texture,
            ui.x(center - (width + ICON + GAP) * 0.5),
            ui.y(y + 15.0),
            color,
            DrawTextureParams {
                dest_size: Some(vec2(ICON * ui.s, ICON * ui.s)),
                ..Default::default()
            },
        );
        true
    });
    let label_center = center + if with_icon { (ICON + GAP) * 0.5 } else { 0.0 };
    ui.text(
        label,
        label_center,
        y + 34.0,
        if with_icon { size } else { TEXT },
        color,
        Align::Center,
        true,
    );
}
