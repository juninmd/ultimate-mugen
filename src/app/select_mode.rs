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
        let search = self.search_label();
        if FAMILY.contains(&self.mode) {
            format!("[/] {search} · [M] {}", tr3("modo", "mode", "modo"))
        } else {
            format!("[/] {search}")
        }
    }
}
