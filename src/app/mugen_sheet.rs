use super::mugen_roster::{self, Roster};
use super::*;
use crate::holo::CardState;
use crate::i18n::tr3;
use crate::registry::Src;
use std::sync::Arc;
use ume_engine::chardef::CharDef;

const CARD: (f32, f32, f32, f32) = (60.0, 112.0, 300.0, 375.0);
const INFO_X: f32 = 410.0;
const INFO_W: f32 = 830.0;
const GOLD: &str = "#ffcc33";

impl App {
    pub(super) fn mugen_sheet(&mut self, list: &[usize]) {
        crate::screen_art::ScreenArt::Archive.background(&self.ui);
        let mut ui = mugen_roster::state();
        if list.is_empty() {
            ui.sheet = false;
            mugen_roster::set(ui);
            return;
        }
        ui.cur = ui.cur.min(list.len() - 1);
        self.sheet_keys(&mut ui, list.len());
        if !ui.sheet {
            mugen_roster::set(ui);
            return;
        }
        let i = list[ui.cur];
        let def = self.sheet_def(i, &mut ui);
        let failed = self.reg.chars[i].errs_has(1);
        self.sheet_identity(i, ui.cur, list.len());
        self.sheet_facts(i, def.as_deref());
        self.sheet_moves(def.as_deref(), failed, &mut ui);
        self.sheet_footer();
        mugen_roster::set(ui);
    }

    fn sheet_keys(&mut self, ui: &mut Roster, total: usize) {
        let back = (1000.0, 64.0, 240.0, 34.0);
        if self.c.pressed(nav::BACK) || self.clicked_rect(back.0, back.1, back.2, back.3) {
            ui.sheet = false;
            return;
        }
        let step = |ui: &mut Roster, next: usize| {
            ui.cur = next;
            ui.move_cur = 0;
            ui.asked = false;
        };
        let (cx, cy, cw, ch) = CARD;
        let arrows_y = cy + ch + 36.0;
        let prev_click = self.clicked_rect(cx, arrows_y, 110.0, 28.0);
        let next_click = self.clicked_rect(cx + cw - 110.0, arrows_y, 110.0, 28.0);
        if (self.c.pressed(nav::LEFT) || prev_click) && ui.cur > 0 {
            step(ui, ui.cur - 1);
        }
        if (self.c.pressed(nav::RIGHT) || next_click) && ui.cur + 1 < total {
            step(ui, ui.cur + 1);
        }
        if self.c.pressed(nav::UP) {
            ui.move_cur = ui.move_cur.saturating_sub(1);
        }
        if self.c.pressed(nav::DOWN) {
            ui.move_cur += 1;
        }
    }

    fn sheet_def(&mut self, i: usize, ui: &mut Roster) -> Option<Arc<CharDef>> {
        if !ui.asked {
            ui.asked = true;
            return None;
        }
        let e = &self.reg.chars[i];
        if e.mugen {
            e.stream_preview(1);
            e.peek(1)
        } else {
            e.load(1).ok()
        }
    }

    fn sheet_identity(&mut self, i: usize, cur: usize, total: usize) {
        self.ui.text(tr3("FICHA DO PERSONAGEM", "CHARACTER SHEET", "FICHA DEL PERSONAJE"), 40.0, 56.0, 30.0, hex(GOLD), Align::Left, true);
        let (bx, by, bw, bh) = (1000.0, 64.0, 240.0, 34.0);
        self.ui.adapted_plate(bx, by, bw, bh, Some(alpha(hex("#22e8ff"), 0.5)));
        self.ui.text(tr3("< VOLTAR (ESC)", "< BACK (ESC)", "< VOLVER (ESC)"), bx + bw / 2.0, by + 23.0, 16.0, WHITE, Align::Center, true);
        let (x, y, w, h) = CARD;
        crate::menu_art::glass(&self.ui, x - 14.0, y - 10.0, w + 28.0, h + 74.0, 1.0);
        let side = if self.reg.chars[i].mugen { 0 } else { 1 };
        let card = self.holo_card_at(i, CardState::Ready, side);
        self.holo_paint(&card, x, y, w, h, false);
        let counter = format!("{} / {total}", cur + 1);
        self.ui.text(&counter, x + w / 2.0, y + h + 28.0, 16.0, hex("#a49cc0"), Align::Center, false);
        self.ui.text("<", x + 55.0, y + h + 56.0, 22.0, hex("#7fe9ff"), Align::Center, true);
        self.ui.text(">", x + w - 55.0, y + h + 56.0, 22.0, hex("#7fe9ff"), Align::Center, true);
        self.draw_stat_gauges(x - 8.0, y + h + 92.0, w + 16.0, i);
    }

    fn sheet_facts(&mut self, i: usize, def: Option<&CharDef>) {
        let e = &self.reg.chars[i];
        let name = e.name.clone();
        self.ui.text(&super::mugen_catalog::short(&name, 28), INFO_X, 140.0, 40.0, WHITE, Align::Left, true);
        let profile = crate::profile::of(e);
        let lang = crate::profile::lang();
        if let Some(p) = profile {
            let value = p.nationality.pick(lang).to_uppercase();
            let has_flag = self.ui.flag(&p.country, INFO_X, 152.0, 44.0, 1.0);
            let nx = if has_flag { INFO_X + 56.0 } else { INFO_X };
            self.ui.text(&value, nx, 172.0, 18.0, hex("#7fe9ff"), Align::Left, true);
        }
        let author = if e.author.is_empty() { tr3("desconhecido", "unknown", "desconocido").to_string() } else { e.author.clone() };
        let source = match &e.src {
            Src::Mugen { def, .. } => format!("M.U.G.E.N · {}", super::mugen_catalog::short(def, 44)),
            Src::Native(_) => tr3("Nativo UMF", "Native UMF", "UMF nativo").to_string(),
        };
        let version = def
            .filter(|_| e.mugen)
            .map(|d| format!("{:.2}", d.mugen_version))
            .unwrap_or_else(|| "—".into());
        let palettes = def.map(|d| d.palettes.keys().filter(|k| k.0 == 1).count().max(1).to_string()).unwrap_or_else(|| "…".into());
        let rows = [
            (tr3("Autor", "Author", "Autor"), author),
            (tr3("Série", "Series", "Serie"), if e.series.is_empty() { "—".into() } else { e.series.clone() }),
            (tr3("Fonte", "Source", "Fuente"), source),
            (tr3("Versão", "Version", "Versión"), version),
            (tr3("Paletas", "Palettes", "Paletas"), palettes),
        ];
        for (n, (label, value)) in rows.iter().enumerate() {
            let y = 204.0 + n as f32 * 22.0;
            self.ui.text(label, INFO_X, y, 14.0, hex("#8a8fa6"), Align::Left, true);
            self.ui.text(value, INFO_X + 90.0, y, 16.0, WHITE, Align::Left, false);
        }
        let bio = match profile {
            Some(p) => p.bio.pick(lang).to_string(),
            None => e.description.clone(),
        };
        let lines = crate::profile::wrap(&bio, INFO_W, |s| self.ui.measure(s, 15.0, false));
        for (n, line) in lines.iter().take(3).enumerate() {
            self.ui.text(line, INFO_X, 330.0 + n as f32 * 18.0, 15.0, Color::new(0.88, 0.86, 0.96, 0.9), Align::Left, false);
        }
    }

    fn sheet_footer(&self) {
        let hint = tr3(
            "ESQ/DIR personagem anterior/próximo   ·   CIMA/BAIXO golpes   ·   ESC voltar",
            "LEFT/RIGHT previous/next character   ·   UP/DOWN moves   ·   ESC back",
            "IZQ/DER personaje anterior/siguiente   ·   ARRIBA/ABAJO golpes   ·   ESC volver",
        );
        self.ui.text(hint, 40.0, 700.0, 15.0, hex("#8a8fa6"), Align::Left, false);
    }
}
