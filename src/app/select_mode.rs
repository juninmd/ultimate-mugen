use super::*;
use crate::i18n::tr3;

pub(crate) const TEAM_MAX: usize = if cfg!(any(target_os = "android", target_os = "ios")) { 5 } else { 7 };
const TAG_MAX: usize = 3;

const FAMILY: [Mode; 6] = [Mode::Cpu, Mode::Versus, Mode::Watch, Mode::TeamSeq, Mode::TeamSimul, Mode::TeamTag];

pub(crate) fn in_family(mode: Mode) -> bool {
    FAMILY.contains(&mode)
}

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

    pub(crate) fn adjust_team_n(&mut self, delta: isize) {
        if self.mode == Mode::PowerArena {
            return;
        }
        let cap = if self.mode == Mode::TeamTag { TAG_MAX } else { TEAM_MAX };
        let next = (self.team_n as isize + delta).clamp(2, cap as isize) as usize;
        self.team_n = next.max(self.team_pick.len() + 1).min(cap);
        self.audio.sfx("select");
    }

    pub(crate) fn cycle_select_mode(&mut self) -> bool {
        self.step_select_mode(1)
    }

    pub(crate) fn step_select_mode(&mut self, delta: isize) -> bool {
        let Some(at) = FAMILY.iter().position(|&m| m == self.mode) else {
            return false;
        };
        let len = FAMILY.len() as isize;
        self.set_mode(FAMILY[(at as isize + delta).rem_euclid(len) as usize]);
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
