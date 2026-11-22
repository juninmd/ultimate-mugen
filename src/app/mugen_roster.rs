use super::mugen_roster_nav::{self as nav_grid, Dir, Move, PER_PAGE};
use super::*;
use crate::holo::{self, CardState};
use crate::i18n::tr3;
use std::cell::Cell;

const AREA: (f32, f32, f32, f32) = (40.0, 238.0, 1200.0, 404.0);
const MAX_CARD_W: f32 = 150.0;
const STREAMS_PER_FRAME: usize = 3;
const CHIP: (f32, f32, f32, f32) = (1000.0, 142.0, 240.0, 30.0);

#[derive(Clone, Copy, Default)]
pub(super) struct Roster {
    pub cur: usize,
    pub button: bool,
    pub all: bool,
    pub sheet: bool,
    pub move_cur: usize,
    pub asked: bool,
    started: bool,
    mouse: (f32, f32),
}

thread_local! {
    static STATE: Cell<Roster> = const { Cell::new(Roster { cur: 0, button: false, all: false, sheet: false, move_cur: 0, asked: false, started: false, mouse: (0.0, 0.0) }) };
}

pub(super) fn state() -> Roster {
    STATE.with(Cell::get)
}

pub(super) fn set(r: Roster) {
    STATE.with(|s| s.set(r));
}

pub(super) fn sheet_open() -> bool {
    state().sheet
}

pub(super) fn button_focused() -> bool {
    state().button
}

fn cols_of(count: usize) -> usize {
    holo::mosaic(count.max(1), AREA.2, AREA.3, MAX_CARD_W).cols
}

impl App {
    pub(super) fn roster_list(&self) -> Vec<usize> {
        let all = state().all;
        (0..self.reg.chars.len())
            .filter(|&i| {
                let e = &self.reg.chars[i];
                !e.boss && (all || e.mugen)
            })
            .collect()
    }

    pub(super) fn mugen_roster(&mut self, list: &[usize]) -> usize {
        let mut ui = state();
        let total = list.len();
        if !ui.started {
            ui.started = true;
            ui.button = total == 0;
        }
        ui.cur = ui.cur.min(total.saturating_sub(1));
        self.roster_keys(&mut ui, total);
        let page = ui.cur / PER_PAGE;
        let start = page * PER_PAGE;
        let count = nav_grid::page_len(total, page);
        let (ax, ay, aw, ah) = AREA;
        let m = holo::mosaic(count.max(1), aw, ah, MAX_CARD_W);
        let cell_h = holo::cell_height(&m);
        let mouse = mouse_position();
        let moved = (mouse.0 - ui.mouse.0).abs() + (mouse.1 - ui.mouse.1).abs() > 0.5;
        ui.mouse = mouse;
        let mut streamed = 0;
        for k in 0..count {
            let idx = list[start + k];
            let e = &self.reg.chars[idx];
            if e.mugen && !e.has_portrait_attempt() && streamed < STREAMS_PER_FRAME {
                e.stream_portrait();
                streamed += 1;
            }
            let (x, y) = holo::origin(&m, k, count, ax, ay, aw, ah);
            if self.hover_rect(x, y, m.w, cell_h) && moved && !ui.sheet {
                ui.cur = start + k;
                ui.button = false;
            }
            if self.clicked_rect(x, y, m.w, cell_h) {
                ui.cur = start + k;
                ui.button = false;
                self.open_sheet(&mut ui);
            }
        }
        for k in 0..count {
            let idx = list[start + k];
            let (x, y) = holo::origin(&m, k, count, ax, ay, aw, ah);
            let pending = self.reg.chars[idx].mugen && !self.reg.chars[idx].has_portrait_attempt();
            let state = if pending { CardState::Loading } else { CardState::Ready };
            let side = (self.reg.chars[idx].mugen as usize) ^ 1;
            let card = self.holo_card_at(idx, state, side);
            self.holo_paint(&card, x, y, m.w, m.h, false);
            if m.labels {
                let name = super::mugen_catalog::short(&card.name, (m.w / 8.5) as usize);
                self.ui.text(&name, x + m.w / 2.0, y + m.h + 22.0, 14.0, WHITE, Align::Center, true);
            }
            if start + k == ui.cur && !ui.button {
                let pulse = 0.65 + 0.35 * (self.ui.t * 5.0).sin();
                self.ui.rect_lines(x - 4.0, y - 4.0, m.w + 8.0, m.h + 8.0, 3.0, alpha(hex("#ffcc33"), pulse));
            }
        }
        if total == 0 {
            let empty = tr3("Nenhum personagem importado ainda.", "No characters imported yet.", "Ningún personaje importado todavía.");
            self.ui.text(empty, VW / 2.0, 420.0, 22.0, hex("#a49cc0"), Align::Center, false);
        }
        self.roster_chrome(&mut ui, list, total);
        set(ui);
        page
    }

    fn roster_keys(&mut self, ui: &mut Roster, total: usize) {
        let dirs = [(nav::UP, Dir::Up), (nav::DOWN, Dir::Down), (nav::LEFT, Dir::Left), (nav::RIGHT, Dir::Right)];
        for (bit, dir) in dirs {
            if !self.c.pressed(bit) {
                continue;
            }
            if ui.button {
                if dir == Dir::Down && total > 0 {
                    ui.button = false;
                }
                continue;
            }
            match nav_grid::go(ui.cur, total, dir, &cols_of) {
                Move::To(i) => ui.cur = i,
                Move::Button => ui.button = true,
            }
            self.audio.sfx("move");
        }
        if !ui.button && total > 0 && self.c.pressed(nav::OK) {
            self.open_sheet(ui);
        }
        if is_key_pressed(KeyCode::F) {
            ui.all = !ui.all;
            ui.cur = 0;
        }
    }

    fn open_sheet(&mut self, ui: &mut Roster) {
        self.audio.sfx("confirm");
        ui.sheet = true;
        ui.move_cur = 0;
        ui.asked = false;
    }

    fn roster_chrome(&mut self, ui: &mut Roster, list: &[usize], total: usize) {
        let (cx, cy, cw, ch) = CHIP;
        self.ui.adapted_plate(cx, cy, cw, ch, Some(alpha(hex("#22e8ff"), 0.5)));
        let (mugen, all) = (tr3("SÓ M.U.G.E.N", "M.U.G.E.N ONLY", "SOLO M.U.G.E.N"), tr3("TODOS", "ALL", "TODOS"));
        let label = format!("[F] {}", if ui.all { all } else { mugen });
        self.ui.text(&label, cx + cw / 2.0, cy + 21.0, 16.0, WHITE, Align::Center, true);
        if self.clicked_rect(cx, cy, cw, ch) {
            ui.all = !ui.all;
            ui.cur = 0;
        }
        let pages = nav_grid::page_count(total);
        let page = ui.cur / PER_PAGE;
        for (n, (x, label)) in [(1130.0, "<"), (1190.0, ">")].into_iter().enumerate() {
            self.ui.adapted_plate(x, 672.0, 48.0, 30.0, None);
            self.ui.text(label, x + 24.0, 693.0, 20.0, WHITE, Align::Center, true);
            if self.clicked_rect(x, 672.0, 48.0, 30.0) {
                let step = if n == 0 { page.saturating_sub(1) } else { (page + 1).min(pages - 1) };
                ui.cur = (step * PER_PAGE).min(total.saturating_sub(1));
            }
        }
        if let Some(&idx) = list.get(ui.cur) {
            let e = &self.reg.chars[idx];
            let by = if e.author.is_empty() { String::new() } else { format!("  ·  {} {}", tr3("por", "by", "por"), e.author) };
            let text = super::mugen_catalog::short(&format!("{}{by}", e.name), 56);
            self.ui.text(&text, 1240.0, 656.0, 18.0, hex("#ffcc33"), Align::Right, true);
        }
    }
}

impl App {
    pub(crate) fn roster_tour_focus(&mut self, cur: usize, sheet: bool, all: bool) {
        let mut ui = state();
        ui.cur = cur;
        ui.sheet = sheet;
        ui.all = all;
        ui.asked = false;
        ui.move_cur = 0;
        ui.button = false;
        ui.started = true;
        set(ui);
    }
}
