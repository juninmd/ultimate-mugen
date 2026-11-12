use super::*;
use crate::i18n::tr3;

const BTN_Y: f32 = 508.0;
const BTN_H: f32 = 28.0;
const BTN_W: f32 = 298.0;
const BTN_X: f32 = 56.0;

impl App {
    pub(super) fn market_links(&mut self, current: Option<(String, String)>, modal: bool) {
        let (repo_key, item_id) = match &current {
            Some((k, i)) => (Some(k.clone()), Some(i.clone())),
            None => (None, None),
        };
        let records = &self.market.state.repos;
        let repo = repo_key
            .as_deref()
            .and_then(|k| records.iter().find(|r| r.key == k))
            .or_else(|| records.get(self.market_ui.repo))
            .map(|r| r.repo.clone());
        let Some(repo) = repo else { return };
        let item_url = item_id.as_deref().map(|id| repo.tree_url(id));
        let entries = [
            (tr3("Abrir repositório", "Open repository", "Abrir repositorio"), "G", Some(repo.web_url()), KeyCode::G),
            (tr3("Abrir recurso", "Open resource", "Abrir recurso"), "L", item_url, KeyCode::L),
        ];
        let mut chosen: Option<String> = None;
        for (i, (label, key, url, code)) in entries.into_iter().enumerate() {
            let (x, y) = (BTN_X, BTN_Y + i as f32 * (BTN_H + 6.0));
            let live = url.is_some() && !modal;
            let hover = live && self.hover_rect(x, y, BTN_W, BTN_H);
            let a = if !live { 0.1 } else if hover { 0.5 } else { 0.28 };
            self.ui.rect(x, y, BTN_W, BTN_H, Color::new(0.35, 0.4, 0.95, a));
            let ink = if live { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) };
            self.ui.text(&format!("{label}  [{key}]"), x + BTN_W / 2.0, y + 20.0, 15.0, ink, Align::Center, true);
            if live && (self.clicked_rect(x, y, BTN_W, BTN_H) || is_key_pressed(code)) {
                chosen = url;
            }
        }
        if let Some(url) = chosen {
            self.market.notice_error = !crate::open_link::open(&url);
            self.market.notice = if self.market.notice_error {
                tr3("Não foi possível abrir o navegador: ", "Could not open the browser: ", "No se pudo abrir el navegador: ").to_string() + &url
            } else {
                url
            };
        }
    }
}
