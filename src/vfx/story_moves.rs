use super::Vfx;
use ume_engine::matchs::Match;
use ume_engine::native::throw::is_victim_state;

const GRAB_COLOR: [f32; 3] = [1.0, 0.55, 0.2];
const COMBO_COLOR: [f32; 3] = [1.0, 0.85, 0.3];
const ENTRY_DUST: [f32; 3] = [0.82, 0.78, 0.7];
const COMBO_MARKS: [i32; 4] = [3, 5, 8, 12];

#[derive(Default)]
pub(super) struct MoveState {
    held: Vec<u32>,
    combo_mark: i32,
    seen: Vec<u32>,
}

impl Vfx {
    pub(super) fn grab_cue(&mut self, m: &Match, state: &mut MoveState) {
        for f in m.roots() {
            let held = f.alive && is_victim_state(f.state_no);
            let known = state.held.contains(&f.id);
            if held && !known {
                state.held.push(f.id);
                self.float(f.pos.x, f.lane - 60.0, crate::i18n::tr("AGARRÃO!", "GRAB!").to_string(), 14.0, GRAB_COLOR);
                self.ring(f.pos.x, f.lane - 30.0, 60.0, 0.4, 2.0, GRAB_COLOR);
                self.sfx.push("guardbreak");
            } else if !held && known {
                state.held.retain(|&id| id != f.id);
            }
        }
    }

    pub(super) fn combo_cue(&mut self, m: &Match, state: &mut MoveState) {
        let Some(hero) = m.roots().find(|f| f.side == 0) else {
            return;
        };
        if hero.combo_count < COMBO_MARKS[0] {
            state.combo_mark = 0;
            return;
        }
        let mark = COMBO_MARKS.iter().rev().copied().find(|&c| hero.combo_count >= c).unwrap_or(0);
        if mark <= state.combo_mark {
            return;
        }
        state.combo_mark = mark;
        let label = format!("{} {}", hero.combo_count, crate::i18n::tr("GOLPES", "HITS"));
        self.float(hero.pos.x, hero.lane - 90.0, label, 16.0, COMBO_COLOR);
        self.ring(hero.pos.x, hero.lane - 40.0, 50.0, 0.3, 1.8, COMBO_COLOR);
    }

    pub(super) fn entrance_cue(&mut self, m: &Match, state: &mut MoveState) {
        for f in m.roots().filter(|f| f.side != 0 && f.alive) {
            if state.seen.contains(&f.id) {
                continue;
            }
            state.seen.push(f.id);
            self.dust(f.pos.x, f.lane, 6, 1.4, 0.5, ENTRY_DUST);
        }
    }
}
