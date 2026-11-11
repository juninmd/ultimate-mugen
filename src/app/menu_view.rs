use super::{App, MENU};
use crate::gfx::*;
use crate::ui::*;
use crate::ui_tokens::*;

impl App {
    pub(super) fn menu_chrome(&self, accent: Color) {
        let ui = &self.ui;
        crate::menu_art::glass(ui, -400.0, 0.0, 400.0 + PANEL_W, VH, 1.0);
        ui.rect(-400.0, 0.0, 400.0 + PANEL_W, VH, Color::new(0.0, 0.0, 0.0, 0.18));
        ui.rect(PANEL_W, 0.0, 3.0, VH, Color::new(accent.r, accent.g, accent.b, 0.8));
        ui.logo(LIST_X + LIST_W * 0.5, 118.0, 74.0);
    }

    pub(super) fn menu_info(&self, cur_v: usize, accent: Color) {
        let ui = &self.ui;
        let right = VW - MARGIN;
        let title = self.menu_text(MENU[cur_v].0);
        let desc = crate::i18n::t(MENU[cur_v].1);
        let ta = self.menu_art_fade().clamp(0.0, 1.0);
        let scrim_y = 520.0;
        for i in 0..12 {
            let t = i as f32 / 12.0;
            ui.rect(
                PANEL_W + 3.0,
                scrim_y + t * (VH - scrim_y),
                VW - PANEL_W - 3.0,
                (VH - scrim_y) / 12.0 + 1.0,
                Color::new(0.02, 0.01, 0.06, 0.5 * t * ta),
            );
        }
        let big = if title.chars().count() > 12 { 44.0 } else { 64.0 };
        ui.text_bold(
            &title.to_uppercase(),
            right,
            590.0,
            big,
            Color::new(1.0, 1.0, 1.0, ta),
            Color::new(0.05, 0.02, 0.1, ta),
            Align::Right,
        );
        ui.rect(right - 400.0, 608.0, 400.0, 4.0, Color::new(accent.r, accent.g, accent.b, ta));
        ui.text(desc, right, 646.0, TEXT_LG, Color::new(1.0, 1.0, 1.0, ALPHA_BODY * ta), Align::Right, false);
        let (nm, nn) = (
            self.reg.chars.iter().filter(|c| c.mugen).count(),
            self.playable().len(),
        );
        let stats = crate::i18n::fill(
            crate::i18n::tr3(
                "{nn} lutadores · {st} cenários · {nm} MUGEN",
                "{nn} fighters · {st} stages · {nm} MUGEN",
                "{nn} luchadores · {st} escenarios · {nm} MUGEN",
            ),
            &[("nn", &nn.to_string()), ("st", &self.reg.stages.len().to_string()), ("nm", &nm.to_string())],
        );
        ui.text(&stats, right, 44.0, TEXT_MD, Color::new(1.0, 1.0, 1.0, ALPHA_BODY), Align::Right, false);
        if cur_v <= 4 {
            ui.text(
                crate::i18n::tr3("Dificuldade CPU (esq./dir.)", "CPU difficulty (left/right)", "Dificultad CPU (izq./der.)"),
                right,
                86.0,
                TEXT_SM,
                hex("#d8d0f0"),
                Align::Right,
                false,
            );
            for i in 0..8 {
                ui.rect(
                    right - 8.0 * 26.0 + 6.0 + i as f32 * 26.0,
                    96.0,
                    20.0,
                    12.0,
                    if i < self.ai_level { accent } else { Color::new(1.0, 1.0, 1.0, 0.18) },
                );
            }
        }
        ui.text(&crate::update::build_label(), LIST_X, VH - 20.0, TEXT_SM, hex("#a49cc0"), Align::Left, false);
        let pads = &self.c.pad_info;
        let pad_line = if pads.is_empty() {
            crate::i18n::tr3("teclado", "keyboard", "teclado").to_string()
        } else {
            format!("{} {}", pads.len(), crate::i18n::tr3("controle(s)", "controller(s)", "mando(s)"))
        };
        ui.text(&pad_line, LIST_X + LIST_W, VH - 20.0, TEXT_SM, hex("#a49cc0"), Align::Right, false);
    }
}
