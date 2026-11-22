use super::rows::Row;
use super::*;

const ROW_A: f32 = 624.0;
const ROW_B: f32 = 668.0;
const BUTTON_H: f32 = 38.0;

impl App {
    pub(super) fn market_button(&self, x: f32, y: f32, w: f32, label: &str, key: &str, accent: Color, enabled: bool) -> bool {
        let hover = enabled && self.hover_rect(x, y, w, BUTTON_H);
        let strength = if !enabled { 0.18 } else if hover { 0.95 } else { 0.62 };
        self.ui.adapted_plate(x, y, w, BUTTON_H, Some(alpha(accent, strength)));
        let ink = if enabled { WHITE } else { Color::new(1.0, 1.0, 1.0, 0.4) };
        self.ui.text(label, x + w / 2.0, y + BUTTON_H / 2.0 + 6.0, 17.0, ink, Align::Center, true);
        if !key.is_empty() {
            self.ui.text(key, x + w - 14.0, y + 14.0, 11.0, Color::new(1.0, 1.0, 1.0, if enabled { 0.65 } else { 0.3 }), Align::Right, true);
        }
        enabled && self.clicked_rect(x, y, w, BUTTON_H)
    }

    pub(super) fn market_action_bar(&self, rows: &[Row], rx: f32, rw: f32, modal: bool, acts: &mut Vec<Act>) {
        let missing = rows.iter().filter(|r| r.installed.is_none() && r.installable).count();
        let stale = rows.iter().filter(|r| r.outdated).count();
        let installed = rows.iter().filter(|r| r.installed.is_some()).count();
        let any = !rows.is_empty();
        let gap = 8.0;
        let top: [(&str, &str, &str, Color, Act, bool); 6] = [
            (mt("Marcar tudo"), "M", "", hex("#5865f2"), Act::MarkAll, any),
            (mt("Limpar marcas"), "N", "", hex("#5865f2"), Act::ClearMarks, !self.market_ui.marked.is_empty()),
            (mt("Instalar"), "I", "", hex("#2ecc71"), Act::Install, any),
            (mt("Ativar"), "E", "", hex("#22b8cf"), Act::Enable, any),
            (mt("Desativar"), "O", "", hex("#f59f00"), Act::Disable, any),
            (mt("Remover"), "X", "", hex("#ff4d6d"), Act::Remove, any),
        ];
        let w = (rw - gap * 5.0) / 6.0;
        for (i, (label, key, _, col, act, on)) in top.into_iter().enumerate() {
            if self.market_button(rx + i as f32 * (w + gap), ROW_A, w, label, key, col, on) && !modal {
                acts.push(act);
            }
        }
        let batch: [(String, &str, Color, Act, bool); 4] = [
            (format!("{} · {missing}", mt("Instalar tudo")), "Z", hex("#2ecc71"), Act::InstallAll, missing > 0),
            (format!("{} · {stale}", mt("Atualizar tudo")), "Y", hex("#ff9e3d"), Act::UpdateAll, stale > 0),
            (format!("{} · {installed}", mt("Remover tudo")), "B", hex("#ff4d6d"), Act::AskRemoveAll, installed > 0),
            (mt("Atualizar lista").to_string(), "R", hex("#8a2be2"), Act::Refresh, true),
        ];
        let w = (rw - gap * 3.0) / 4.0;
        for (i, (label, key, col, act, on)) in batch.into_iter().enumerate() {
            if self.market_button(rx + i as f32 * (w + gap), ROW_B, w, &label, key, col, on) && !modal {
                acts.push(act);
            }
        }
    }
}
