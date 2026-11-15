use super::*;
use crate::i18n::tr3;

const FAMILY: [Mode; 6] = [Mode::Cpu, Mode::Versus, Mode::Watch, Mode::TeamSeq, Mode::TeamSimul, Mode::TeamTag];

impl App {
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.team = None;
        self.team_pick.clear();
        self.team_n = match mode {
            Mode::TeamSimul => 5,
            Mode::TeamSeq => 3,
            Mode::TeamTag => 2,
            _ => self.team_n,
        };
    }

    pub(crate) fn cycle_select_mode(&mut self) -> bool {
        let Some(at) = FAMILY.iter().position(|&m| m == self.mode) else {
            return false;
        };
        self.set_mode(FAMILY[(at + 1) % FAMILY.len()]);
        self.audio.sfx("confirm");
        self.screen = Screen::Select {
            cur: [self.sel.0, self.sel.1],
            picking: 0,
            picked: [None, None],
        };
        true
    }

    pub(crate) fn select_tail(&self) -> String {
        let rows = self.grid_rows().max(1);
        let total_rows = super::select_view::last_view_len().div_ceil(self.grid_cols(super::select_view::last_view_len()));
        let pages = total_rows.div_ceil(rows).max(1);
        let page = (self.sel_row.div_ceil(rows)).min(pages - 1) + 1;
        let search = format!("{} {page}/{pages} · {}", tr3("pág", "page", "pág"), self.search_label());
        if FAMILY.contains(&self.mode) {
            format!("[/] {search} · [M] {}", tr3("modo", "mode", "modo"))
        } else {
            format!("[/] {search}")
        }
    }
}
