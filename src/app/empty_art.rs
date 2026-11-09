use super::market_text::mt;
use super::*;

impl App {
    pub(super) fn market_empty_state(&self, browse: bool, has_repo: bool, listing_now: bool, rx: f32, ry: f32, rw: f32) {
        let msg = if !browse {
            mt("Nenhum item instalado ainda. Use Explorar para baixar.")
        } else if !has_repo {
            mt("Adicione um repositório para começar.")
        } else if listing_now {
            mt("Lendo o repositório no GitHub…")
        } else {
            mt("Nada encontrado aqui (ou ainda não listado). Pressione R para atualizar.")
        };
        let cx = rx + rw / 2.0;
        let pulse = 0.45 + 0.4 * (ume_shell::quad::get_time() as f32 * 2.2).sin().abs();
        let icon = Color::new(0.13, 0.9, 1.0, pulse);
        self.ui.rect(cx - 18.0, ry + 194.0, 36.0, 4.0, icon);
        self.ui.rect(cx - 2.0, ry + 178.0, 4.0, 36.0, icon);
        self.ui.text(
            msg,
            rx + rw / 2.0,
            ry + 240.0,
            20.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }

    pub(super) fn replays_empty_state(&self) {
        let cx = VW / 2.0;
        let pulse = 0.5 + 0.4 * (ume_shell::quad::get_time() as f32 * 2.0).sin().abs();
        self.ui.quad(
            [
                vec2(cx - 14.0, 224.0),
                vec2(cx - 14.0, 264.0),
                vec2(cx + 20.0, 244.0),
                vec2(cx + 20.0, 244.0),
            ],
            Color::new(1.0, 0.8, 0.2, pulse),
        );
        self.ui.text(
            crate::i18n::t("replays.empty"),
            VW / 2.0,
            300.0,
            26.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
        self.ui.text(
            crate::i18n::t("replays.empty_hint"),
            VW / 2.0,
            340.0,
            20.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }
}
