use super::*;
use crate::i18n::tr3;

const PANEL: Color = Color::new(0.02, 0.01, 0.06, 0.62);

impl App {
    pub(super) fn photo_overlay(&self) {
        let Some(photo) = self.photo.as_ref() else { return };
        let line = Color::new(1.0, 1.0, 1.0, 0.16);
        for i in 1..3 {
            let (x, y) = (VW * i as f32 / 3.0, VH * i as f32 / 3.0);
            self.ui.rect(x - 0.5, 0.0, 1.0, VH, line);
            self.ui.rect(0.0, y - 0.5, VW, 1.0, line);
        }
        self.ui.rect(VW / 2.0 - 330.0, VH - 78.0, 660.0, 62.0, PANEL);
        let state = format!(
            "{}  ·  {}: {}  ·  HUD: {}  ·  Zoom {:.2}x",
            tr3("MODO FOTO", "PHOTO MODE", "MODO FOTO"),
            tr3("Filtro", "Filter", "Filtro"),
            photo.filter.label(),
            if photo.hud { "ON" } else { "OFF" },
            photo.view.zoom,
        );
        self.ui.text(&state, VW / 2.0, VH - 54.0, 17.0, hex("#ffcc33"), Align::Center, false);
        let capture = if super::photo_mode::CAN_SAVE {
            tr3("Enter foto", "Enter capture", "Enter foto")
        } else {
            ""
        };
        let keys = format!(
            "{}  {}",
            tr3(
                "WASD mover · +/- zoom · F filtro · H HUD · . quadro · R centro ·",
                "WASD move · +/- zoom · F filter · H HUD · . frame · R center ·",
                "WASD mover · +/- zoom · F filtro · H HUD · . cuadro · R centro ·",
            ),
            capture,
        );
        self.ui.text(keys.trim(), VW / 2.0, VH - 30.0, 13.0, hex("#d8d0f0"), Align::Center, false);
        if let Some((text, t)) = &photo.toast {
            let a = (t / 0.4).min(1.0);
            self.ui.rect(VW / 2.0 - 330.0, 36.0, 660.0, 36.0, Color::new(0.02, 0.01, 0.06, 0.7 * a));
            self.ui.text(text, VW / 2.0, 60.0, 15.0, Color::new(1.0, 1.0, 1.0, a), Align::Center, false);
        }
        if photo.flash > 0.0 {
            self.ui.rect(0.0, 0.0, VW, VH, Color::new(1.0, 1.0, 1.0, photo.flash * 0.7));
        }
    }

    pub(super) fn photo_hint(&self) {
        if self.session.as_ref().is_some_and(|s| !crate::photo::allowed(s)) {
            return;
        }
        let text = tr3("C  Modo foto", "C  Photo mode", "C  Modo foto");
        self.ui.text(text, VW - 28.0, VH - 22.0, 15.0, hex("#d8d0f0"), Align::Right, false);
    }
}
