use super::draw_chrome::{CYAN, GOLD, INK, MUTED, PINK};
use super::labels::{cost_label, truncate_to};
use super::input_fit::fit_input_size;
use super::layout::{Columns, Layout, SCROLLBAR_W};
use super::skin::{clip, fade, pill, unclip};
use super::state::Action;
use crate::app::*;
use crate::ui::{hex, FaceShape};
use ume_engine::chardef::MoveInfo;

impl App {
    fn button_width(&self, ch: char, size: f32) -> f32 {
        let bit = match ch {
            'x' => ume_engine::input::k::X,
            'y' => ume_engine::input::k::Y,
            'z' => ume_engine::input::k::Z,
            'a' => ume_engine::input::k::A,
            'b' => ume_engine::input::k::B,
            'c' => ume_engine::input::k::C,
            _ => ume_engine::input::k::S,
        };
        let f = self.c.face(bit);
        match f.shape {
            FaceShape::Round => size,
            _ => (self.ui.measure(f.text, size * 0.62, true) + size * 0.6).max(size),
        }
    }

    pub(super) fn pause_moves_list(&self, lay: &Layout, moves: &[MoveInfo], slot_of: &dyn Fn(&str) -> Option<usize>, a: f32) {
        let ui = &self.ui;
        let f = lay.font;
        let l = lay.list;
        let cols = Columns::new(l.w, lay.compact);
        let heading = fade(hex(MUTED), a);
        let hy = lay.heading_y;
        ui.text(crate::i18n::tr("GOLPE", "MOVE"), l.x + cols.name_x, hy, 14.0 * f, heading, Align::Left, true);
        ui.text(crate::i18n::tr("COMANDO", "INPUT"), l.x + cols.input_x, hy, 14.0 * f, heading, Align::Left, true);
        ui.text(crate::i18n::tr("CUSTO", "COST"), l.x + cols.cost_right, hy, 14.0 * f, heading, Align::Right, true);
        if moves.is_empty() {
            ui.text(crate::i18n::t("pause.no_moves"), l.x + 12.0, l.y + 40.0, 20.0 * f, fade(hex(INK), a), Align::Left, false);
            return;
        }
        clip(ui, l);
        let first = (self.pause.scroll / lay.pitch).floor().max(0.0) as usize;
        let last = (first + lay.visible_rows() + 2).min(moves.len());
        for (i, mv) in moves.iter().enumerate().take(last).skip(first) {
            let row = lay.move_row(i, self.pause.scroll);
            let y = row.y;
            let on = i == self.pause.move_cur;
            let hover = self.hover_rect(row.x, row.y, row.w, row.h) && self.hover_rect(l.x, l.y, l.w, l.h);
            let (top, bot, edge) = if on {
                (Color::new(1.0, 0.3, 0.43, 0.55), Color::new(0.6, 0.12, 0.3, 0.45), Some(fade(hex(PINK), 0.9 * a)))
            } else if hover {
                (Color::new(1.0, 1.0, 1.0, 0.14), Color::new(1.0, 1.0, 1.0, 0.08), None)
            } else {
                let v = if i % 2 == 0 { 0.07 } else { 0.04 };
                (Color::new(1.0, 1.0, 1.0, v), Color::new(1.0, 1.0, 1.0, v * 0.6), None)
            };
            pill(ui, row, 7.0, fade(top, a), fade(bot, a), edge);
            if on {
                ui.rect(row.x, row.y + 6.0, 4.0, row.h - 12.0, fade(hex(CYAN), a));
            }
            let mid = y + lay.row_h / 2.0;
            if let Some(slot) = slot_of(&mv.command) {
                let s = 30.0 * f.min(1.2);
                let bx = l.x + cols.badge_x;
                ui.rect(bx, mid - s / 2.0, s, s, fade(hex(CYAN), a));
                ui.text(&(slot + 1).to_string(), bx + s / 2.0, mid + 7.0 * f, 20.0 * f, fade(hex("#06202a"), a), Align::Center, true);
            }
            let name = truncate_to(&mv.name, cols.name_w, &|s| ui.measure(s, 20.0 * f, false));
            ui.text(&name, l.x + cols.name_x, mid + 7.0 * f, 20.0 * f, fade(hex(INK), a), Align::Left, false);
            let base = 28.0 * f.min(1.15);
            let size = fit_input_size(&mv.input, cols.input_w, base, 13.0, &|c, s| self.button_width(c, s));
            ui.input_icons(&mv.input, l.x + cols.input_x, mid + size / 2.0, size, |b| self.c.face(b));
            if let Some(cost) = cost_label(mv) {
                let tw = ui.measure(&cost, 15.0 * f, true) + 22.0;
                let pr = super::layout::Rect::new(l.x + cols.cost_right - tw, mid - 14.0 * f, tw, 28.0 * f);
                pill(ui, pr, 6.0, fade(Color::new(1.0, 0.8, 0.2, 0.2), a), fade(Color::new(1.0, 0.8, 0.2, 0.08), a), Some(fade(hex(GOLD), 0.5 * a)));
                ui.text(&cost, pr.x + tw / 2.0, mid + 5.5 * f, 15.0 * f, fade(hex(GOLD), a), Align::Center, true);
            }
        }
        unclip();
        self.pause_scrollbar(lay, moves.len(), a);
    }

    fn pause_scrollbar(&self, lay: &Layout, total: usize, a: f32) {
        let max = lay.max_scroll(total);
        if max <= 0.0 {
            return;
        }
        let l = lay.list;
        let x = l.right() - SCROLLBAR_W + 4.0;
        self.ui.rect(x, l.y, 4.0, l.h, Color::new(1.0, 1.0, 1.0, 0.08 * a));
        let thumb = (l.h * l.h / (l.h + max)).max(36.0);
        let ty = l.y + (l.h - thumb) * (self.pause.scroll / max).clamp(0.0, 1.0);
        self.ui.rect(x - 1.0, ty, 6.0, thumb, fade(hex(CYAN), 0.85 * a));
    }

    pub(super) fn pause_actions_list(&self, lay: &Layout, actions: &[Action], a: f32) {
        let ui = &self.ui;
        let f = lay.font;
        let l = lay.list;
        let _ = l;
        for (i, act) in actions.iter().enumerate() {
            let on = i == self.pause.cur;
            let r = lay.action_row(i);
            let (x, y, row_h) = (r.x, r.y, r.h);
            let hover = self.hover_rect(r.x, r.y, r.w, r.h);
            let (top, bot, edge) = if on {
                (Color::new(1.0, 0.3, 0.43, 0.85), Color::new(0.62, 0.12, 0.3, 0.85), Some(fade(hex("#ffd0da"), 0.7 * a)))
            } else if hover {
                (Color::new(1.0, 1.0, 1.0, 0.16), Color::new(1.0, 1.0, 1.0, 0.08), None)
            } else {
                (Color::new(1.0, 1.0, 1.0, 0.08), Color::new(1.0, 1.0, 1.0, 0.04), None)
            };
            pill(ui, r, 10.0, fade(top, a), fade(bot, a), edge);
            let (title, sub) = self.action_text(*act);
            let mid = y + row_h / 2.0;
            ui.text(&title, x + 24.0, mid - 2.0, 24.0 * f, fade(hex(INK), a), Align::Left, false);
            ui.text(sub, x + 24.0, mid + 20.0 * f, 15.0 * f, fade(if on { hex("#ffe3ea") } else { hex(MUTED) }, a), Align::Left, false);
            if *act == Action::Hitboxes {
                let on_off = self.session.as_ref().is_some_and(|s| s.r.debug);
                let (txt, col) = if on_off { ("ON", hex("#3ddc97")) } else { ("OFF", hex(MUTED)) };
                let pr = super::layout::Rect::new(r.right() - 90.0, mid - 15.0 * f, 66.0, 30.0 * f);
                pill(ui, pr, 7.0, fade(Color::new(0.0, 0.0, 0.0, 0.35), a), fade(Color::new(0.0, 0.0, 0.0, 0.2), a), Some(fade(col, 0.8 * a)));
                ui.text(txt, pr.x + pr.w / 2.0, mid + 6.0 * f, 17.0 * f, fade(col, a), Align::Center, true);
            } else {
                ui.arrow(r.right() - 36.0, mid, 20.0 * f, (1.0, 0.0), fade(if on { WHITE } else { hex(MUTED) }, a));
            }
        }
    }

    fn action_text(&self, act: Action) -> (String, &'static str) {
        use crate::i18n::{t, tr};
        match act {
            Action::Continue => (t("pause.continue").to_string(), tr("Voltar para a luta", "Back to the fight")),
            Action::Restart => (t("pause.restart").to_string(), tr("Recomeca a partida do zero", "Start the match over")),
            Action::Change => (t("pause.change").to_string(), tr("Volta para a selecao de personagens", "Back to character select")),
            Action::Hitboxes => (t("pause.hitboxes").to_string(), tr("Mostra caixas de dano e colisao", "Show hit and hurt boxes")),
            Action::Settings => (t("pause.settings").to_string(), tr("Video, audio e controles", "Video, audio and controls")),
            Action::MainMenu => (t("pause.menu").to_string(), tr("Sai da luta atual", "Leave the current match")),
        }
    }
}
