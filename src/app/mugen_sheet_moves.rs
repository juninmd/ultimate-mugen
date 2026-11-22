use super::mugen_roster::Roster;
use super::*;
use crate::i18n::tr3;
use ume_engine::chardef::CharDef;

const LIST_X: f32 = 410.0;
const LIST_Y: f32 = 400.0;
const LIST_W: f32 = 830.0;
const ROW_H: f32 = 25.0;
const VISIBLE: usize = 10;

impl App {
    pub(super) fn sheet_moves(&mut self, def: Option<&CharDef>, failed: bool, ui: &mut Roster) {
        let title = tr3("GOLPES E CUSTO DE ENERGIA", "MOVES AND ENERGY COST", "GOLPES Y COSTE DE ENERGÍA");
        self.ui.text(title, LIST_X, LIST_Y - 12.0, 18.0, hex("#ffcc33"), Align::Left, true);
        let Some(def) = def else {
            let text = if failed {
                tr3("Falha ao carregar este personagem.", "Failed to load this character.", "Error al cargar este personaje.")
            } else {
                tr3("Carregando ficha…", "Loading sheet…", "Cargando ficha…")
            };
            self.ui.text(text, LIST_X, LIST_Y + 24.0, 18.0, hex("#a49cc0"), Align::Left, false);
            return;
        };
        let moves = &def.movelist;
        if moves.is_empty() {
            let text = tr3("Sem lista de golpes disponível.", "No move list available.", "Sin lista de golpes disponible.");
            self.ui.text(text, LIST_X, LIST_Y + 24.0, 18.0, hex("#a49cc0"), Align::Left, false);
            return;
        }
        ui.move_cur = ui.move_cur.min(moves.len() - 1);
        let top = ui.move_cur.saturating_sub(VISIBLE - 1).min(moves.len().saturating_sub(VISIBLE));
        let count = format!("{} {}", moves.len(), tr3("golpes", "moves", "golpes"));
        self.ui.text(&count, LIST_X + LIST_W, LIST_Y - 12.0, 15.0, hex("#a49cc0"), Align::Right, false);
        for (row, mv) in moves.iter().enumerate().skip(top).take(VISIBLE) {
            let y = LIST_Y + (row - top) as f32 * ROW_H;
            let on = row == ui.move_cur;
            let fill = if on { Color::new(0.13, 0.91, 1.0, 0.2) } else { Color::new(0.06, 0.04, 0.12, 0.7) };
            self.ui.rect(LIST_X, y, LIST_W, ROW_H - 3.0, fill);
            let name = super::mugen_catalog::short(&mv.name, 26);
            self.ui.text(&name, LIST_X + 10.0, y + 17.0, 16.0, WHITE, Align::Left, true);
            let input = super::mugen_catalog::short(&mv.input, 34);
            self.ui.text(&input, LIST_X + 300.0, y + 17.0, 15.0, hex("#9fe8a8"), Align::Left, false);
            if let Some(cost) = super::pause::labels::cost_label(mv) {
                self.ui.text(&cost, LIST_X + LIST_W - 10.0, y + 17.0, 15.0, hex("#ffcc33"), Align::Right, true);
            }
        }
        if moves.len() > VISIBLE {
            let track = ROW_H * VISIBLE as f32 - 3.0;
            let thumb = track * VISIBLE as f32 / moves.len() as f32;
            let ty = LIST_Y + track * top as f32 / moves.len() as f32;
            self.ui.rect(LIST_X + LIST_W + 6.0, LIST_Y, 4.0, track, Color::new(1.0, 1.0, 1.0, 0.1));
            self.ui.rect(LIST_X + LIST_W + 6.0, ty, 4.0, thumb, hex("#22e8ff"));
        }
    }
}
