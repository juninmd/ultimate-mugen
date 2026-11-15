use crate::matchs::{Announce, AnnounceKind, Match};

pub const WINDOW_TICKS: i32 = 360;

impl Match {
    pub(crate) fn finish_arm(&mut self) {
        self.finish_window = 0;
        if self.winner_side < 0 || !self.won_by_ko { return; }
        let side = self.winner_side as usize;
        if self.wins[side] + 1 < self.rounds_to_win { return; }
        let armed = self.roots().any(|f| f.side == side && f.alive && f.def.commands.iter().any(|c| c.name.starts_with("fin_")));
        if armed { self.finish_window = WINDOW_TICKS; }
    }

    pub(crate) fn finish_holding(&mut self) -> bool {
        if self.finish_window <= 0 { return false; }
        if self.finish_window == WINDOW_TICKS {
            self.announce = Some(Announce { text: "FINISH HIM".into(), time: 120, total: 120, kind: AnnounceKind::Other });
        }
        self.finish_window -= 1;
        true
    }

    pub fn finish_done(&mut self) { self.finish_window = 0; }
}
