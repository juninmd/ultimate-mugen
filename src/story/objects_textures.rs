use crate::gfx::*;
use std::cell::RefCell;
use std::collections::HashMap;
use ume_formats::png::decode_png;

const BUNDLED: [(&str, &[u8]); 8] = [
    (
        "crate",
        include_bytes!("../../assets/props/story/crate.png"),
    ),
    (
        "barrel",
        include_bytes!("../../assets/props/story/barrel.png"),
    ),
    ("car", include_bytes!("../../assets/props/story/car.png")),
    (
        "apple",
        include_bytes!("../../assets/props/story/apple.png"),
    ),
    (
        "pizza",
        include_bytes!("../../assets/props/story/pizza.png"),
    ),
    (
        "roast",
        include_bytes!("../../assets/props/story/roast.png"),
    ),
    ("gem", include_bytes!("../../assets/props/story/gem.png")),
    (
        "battery",
        include_bytes!("../../assets/props/story/battery.png"),
    ),
];
pub fn is_bundled(sprite: &str) -> bool {
    BUNDLED.iter().any(|(name, _)| *name == sprite)
}

thread_local! {
    static TEXTURES: RefCell<HashMap<String, Option<Texture2D>>> = RefCell::default();
}

pub fn with_texture(sprite: &str, paint: impl FnOnce(&Texture2D)) {
    TEXTURES.with(|cache| {
        let mut cache = cache.borrow_mut();
        let slot = cache.entry(sprite.to_string()).or_insert_with(|| {
            let png = BUNDLED.iter().find(|(name, _)| *name == sprite)?.1;
            let d = decode_png(png).ok()?;
            let rgba = d
                .rgba
                .clone()
                .unwrap_or_else(|| ume_formats::png::png_indexed_to_rgba(&d));
            let texture = Texture2D::from_rgba8(d.width as u16, d.height as u16, &rgba);
            texture.set_filter(FilterMode::Nearest);
            Some(texture)
        });
        if let Some(texture) = slot.as_ref() {
            paint(texture);
        }
    });
}
