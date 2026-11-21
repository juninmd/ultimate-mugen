use std::sync::Arc;

use crate::gfx::*;
use crate::ui::{Ui, VH, VW};

thread_local! {
    static MENU: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-menu.png"));
    static LOADING: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-loading.png"));
    static SELECT: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-select.png"));
    static VERSUS: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-versus.png"));
    static RECORDS: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-records.png"));
    static RESULTS: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-results.png"));
    static MARKET: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-market.png"));
    static ARCHIVE: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-archive.png"));
    static WORKSHOP: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-workshop.png"));
    static LAB: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-lab.png"));
    static STORY: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-story.png"));
    static TOWER: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-tower.png"));
    static CUTSCENE: Option<Texture2D> = decode(include_bytes!("../assets/ui/arcade-cutscene.png"));
}

#[derive(Clone, Copy)]
pub(crate) enum ScreenArt {
    Menu,
    Loading,
    Select,
    Versus,
    Results,
    Records,
    Market,
    Archive,
    Workshop,
    Lab,
    Story,
    Tower,
    Cutscene,
}

fn decode(data: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(data).ok()?;
    Some(Texture2D::from_rgba_shared(
        image.width as u16,
        image.height as u16,
        Arc::new(match image.rgba {
            Some(rgba) => rgba,
            None => ume_formats::png::png_indexed_to_rgba(&image),
        }),
    ))
}

impl ScreenArt {
    pub(crate) fn draw(self, ui: &Ui, opacity: f32) -> bool {
        let texture = match self {
            Self::Menu => &MENU,
            Self::Loading => &LOADING,
            Self::Select => &SELECT,
            Self::Versus => &VERSUS,
            Self::Results => &RESULTS,
            Self::Records => &RECORDS,
            Self::Market => &MARKET,
            Self::Archive => &ARCHIVE,
            Self::Workshop => &WORKSHOP,
            Self::Lab => &LAB,
            Self::Story => &STORY,
            Self::Tower => &TOWER,
            Self::Cutscene => &CUTSCENE,
        };
        texture.with(|texture| {
            let Some(texture) = texture else {
                return false;
            };
            let color = match self {
                Self::Menu => Color::new(0.68, 0.67, 0.76, opacity),
                Self::Loading | Self::Results | Self::Records | Self::Market | Self::Archive => Color::new(0.62, 0.61, 0.7, opacity),
                Self::Loading | Self::Results | Self::Records | Self::Workshop | Self::Lab => Color::new(0.62, 0.61, 0.7, opacity),
                Self::Loading | Self::Results | Self::Records | Self::Story | Self::Tower => Color::new(0.62, 0.61, 0.7, opacity),
                Self::Select => Color::new(0.65, 0.65, 0.74, opacity),
                Self::Versus => Color::new(0.95, 0.91, 1.0, opacity),
                Self::Cutscene => Color::new(0.72, 0.7, 0.82, opacity),
            };
            let source = match self {
                Self::Menu if !crate::tweaks::current().reduce_motion => {
                    let (w, h) = (texture.width(), texture.height());
                    let crop = 0.025;
                    let x = w * crop * (0.5 + 0.5 * (ui.t * 0.28).sin());
                    Some(Rect::new(
                        x,
                        h * crop * 0.5,
                        w * (1.0 - crop),
                        h * (1.0 - crop),
                    ))
                }
                _ => None,
            };
            draw_texture_ex(
                texture,
                ui.x(0.0),
                ui.y(0.0),
                color,
                DrawTextureParams {
                    source,
                    dest_size: Some(vec2(VW * ui.s, VH * ui.s)),
                    ..Default::default()
                },
            );
            true
        })
    }

    pub(crate) fn background(self, ui: &Ui) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            crate::ui::hex("#0b0715"),
        );
        if !self.draw(ui, 1.0) {
            ui.background();
        }
    }
}
