use super::*;

impl App {
    pub(super) fn market_confirm_modal(&mut self, acts: &mut Vec<Act>) {
        let Some(count) = self.market_ui.confirm.as_ref().map(Vec::len) else { return };
        let (w, h) = (560.0, 210.0);
        let (x, y) = ((VW - w) / 2.0, (VH - h) / 2.0);
        self.ui.rect(0.0, 0.0, VW, VH, Color::new(0.0, 0.0, 0.0, 0.7));
        crate::menu_art::glass(&self.ui, x, y, w, h, 1.0);
        self.ui.text(mt("Remover do aparelho?"), x + w / 2.0, y + 52.0, 28.0, WHITE, Align::Center, true);
        let line = crate::i18n::fill(crate::i18n::tr3("{n} item(ns) instalado(s) serão removidos deste aparelho.", "{n} installed item(s) will be removed from this device.", "Se quitarán {n} elemento(s) instalado(s) de este dispositivo."), &[("n", &count.to_string())]);
        self.ui.text(&line, x + w / 2.0, y + 92.0, 18.0, hex("#c9c3dd"), Align::Center, false);
        let bw = (w - 60.0) / 2.0;
        if self.market_button(x + 20.0, y + h - 64.0, bw, mt("Cancelar"), "Esc", hex("#5865f2"), true) || self.c.pressed(nav::BACK) {
            acts.push(Act::CancelRemove);
        }
        if self.market_button(x + 40.0 + bw, y + h - 64.0, bw, mt("Remover"), "Enter", hex("#ff4d6d"), true) || self.c.pressed(nav::OK) {
            acts.push(Act::ConfirmRemove);
        }
    }
}
