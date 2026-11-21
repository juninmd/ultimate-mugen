use super::*;
use std::{cell::OnceCell, sync::Arc};
use ume_engine::element::Element;

thread_local! {
    static ICONS: [OnceCell<Option<Texture2D>>; 7] = const { [const { OnceCell::new() }; 7] };
}

pub(super) fn draw(element: Option<Element>, position: Vec2, scale: f32, ui: &Ui) {
    let Some(element) = element else { return };
    let (index, bytes, label) = match element {
        Element::Fire => (
            0,
            &include_bytes!("../../assets/ui/elements/fire.png")[..],
            crate::i18n::tr3("Fogo", "Fire", "Fuego"),
        ),
        Element::Ice => (
            1,
            &include_bytes!("../../assets/ui/elements/ice.png")[..],
            crate::i18n::tr3("Gelo", "Ice", "Hielo"),
        ),
        Element::Lightning => (
            2,
            &include_bytes!("../../assets/ui/elements/lightning.png")[..],
            crate::i18n::tr3("Raio", "Lightning", "Rayo"),
        ),
        Element::Water => (
            3,
            &include_bytes!("../../assets/ui/elements/water.png")[..],
            crate::i18n::tr3("Água", "Water", "Agua"),
        ),
        Element::Earth => (
            4,
            &include_bytes!("../../assets/ui/elements/earth.png")[..],
            crate::i18n::tr3("Terra", "Earth", "Tierra"),
        ),
        Element::Wind => (
            5,
            &include_bytes!("../../assets/ui/elements/wind.png")[..],
            crate::i18n::tr3("Vento", "Wind", "Viento"),
        ),
        Element::Neutral => (
            6,
            &include_bytes!("../../assets/ui/elements/neutral.png")[..],
            crate::i18n::tr3("Neutro", "Neutral", "Neutro"),
        ),
    };
    opacity::scope(1.0, || {
        ICONS.with(|icons| {
            let icon = icons[index].get_or_init(|| {
                let png = ume_formats::png::decode_png(bytes).ok()?;
                Some(Texture2D::from_rgba_shared(
                    png.width as u16,
                    png.height as u16,
                    Arc::new(png.rgba?),
                ))
            });
            if let Some(icon) = icon {
                draw_texture_ex(
                    icon,
                    position.x,
                    position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                        ..Default::default()
                    },
                );
            } else {
                let size = (5.0 * scale).max(9.0);
                draw_text_ex(
                    label,
                    position.x,
                    position.y + size,
                    TextParams {
                        font: Some(&ui.body),
                        font_size: size as u16,
                        color: WHITE,
                        ..Default::default()
                    },
                );
            }
        })
    });
}
