use super::state::Action;
use crate::app::*;
use std::sync::Arc;

const ICON_SIZE: usize = 64;

fn load(bytes: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(bytes).ok()?;
    let rgba = if let Some(rgba) = image.rgba {
        rgba
    } else {
        ume_formats::png::png_indexed_to_rgba(&image)
    };
    let texture =
        Texture2D::from_rgba_shared(image.width as u16, image.height as u16, Arc::new(rgba));
    texture.set_filter(FilterMode::Nearest);
    Some(texture)
}

thread_local! {
    static ICONS: [Option<Texture2D>; 7] = [
        load(include_bytes!("../../../assets/ui/pause-icons/continue.png")),
        load(include_bytes!("../../../assets/ui/pause-icons/restart.png")),
        load(include_bytes!("../../../assets/ui/pause-icons/change.png")),
        load(include_bytes!("../../../assets/ui/pause-icons/dummy.png")),
        load(include_bytes!("../../../assets/ui/pause-icons/hitboxes.png")),
        load(include_bytes!("../../../assets/ui/pause-icons/settings.png")),
        load(include_bytes!("../../../assets/ui/pause-icons/main-menu.png")),
    ];
}

fn index(action: Action) -> usize {
    match action {
        Action::Continue => 0,
        Action::Restart => 1,
        Action::Change => 2,
        Action::Dummy => 3,
        Action::Hitboxes => 4,
        Action::Settings => 5,
        Action::MainMenu => 6,
    }
}

pub(super) fn draw(ui: &Ui, action: Action, cx: f32, cy: f32, size: f32, alpha: f32) {
    ICONS.with(|icons| {
        let Some(Some(texture)) = icons.get(index(action)) else {
            return;
        };
        let size = size.min(ICON_SIZE as f32) * ui.s;
        draw_texture_ex(
            texture,
            ui.x(cx) - size * 0.5,
            ui.y(cy) - size * 0.5,
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    });
}
