use super::Vfx;
use std::cell::RefCell;
use story_moves::MoveState;
use ume_engine::matchs::Match;

const STEP_EVERY_TICKS: i64 = 7;
const STEP_MIN_SPEED: f32 = 0.9;
const SPARKLE_EVERY_TICKS: i64 = 24;
const STEP_DUST: [f32; 3] = [0.82, 0.78, 0.7];
const KO_COLOR: [f32; 3] = [1.0, 0.82, 0.3];
const GATE_COLOR: [f32; 3] = [1.0, 0.85, 0.3];
const LOW_LIFE_RATIO: f32 = 0.25;
const LOW_LIFE_EVERY_TICKS: i64 = 40;
const LOCK_TRAUMA: f32 = 0.3;

#[derive(Default)]
struct JuiceState {
    celebrated: Vec<u32>,
    gate: bool,
    scrolling: bool,
    last_tick: i64,
    moves: MoveState,
}

thread_local! {
    static STATE: RefCell<JuiceState> = RefCell::new(JuiceState::default());
}

impl Vfx {
    pub(super) fn story_juice(&mut self, m: &Match) {
        if !m.lanes.active {
            return;
        }
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if m.game_time < state.last_tick {
                state.celebrated.clear();
                state.gate = false;
                state.scrolling = false;
                state.moves = MoveState::default();
            }
            state.last_tick = m.game_time;
            self.footsteps(m);
            self.pickup_sparkles(m);
            self.foe_defeats(m, &mut state);
            self.gate_cue(m, &mut state);
            self.lock_cue(m, &mut state);
            self.low_life_pulse(m);
            self.grab_cue(m, &mut state.moves);
            self.combo_cue(m, &mut state.moves);
            self.entrance_cue(m, &mut state.moves);
        });
    }

    fn footsteps(&mut self, m: &Match) {
        if m.game_time % STEP_EVERY_TICKS != 0 {
            return;
        }
        for f in m.roots().filter(|f| f.alive && f.pos.y >= -0.5) {
            if f.vel.x.abs() < STEP_MIN_SPEED || f.state_type != 'S' {
                continue;
            }
            self.dust(f.pos.x - f.facing * 6.0, f.lane, 2, 0.9, 0.4, STEP_DUST);
        }
    }

    fn pickup_sparkles(&mut self, m: &Match) {
        if m.game_time % SPARKLE_EVERY_TICKS != 0 {
            return;
        }
        for (k, p) in m.objects.pickups.iter().enumerate() {
            let phase = ((m.game_time / SPARKLE_EVERY_TICKS) as usize + k) % 3;
            let lift = 18.0 + phase as f32 * 7.0;
            self.glow(p.pos.x + (phase as f32 - 1.0) * 9.0, p.lane - lift, 9.0, 0.35, GATE_COLOR);
        }
    }

    fn foe_defeats(&mut self, m: &Match, state: &mut JuiceState) {
        for f in m.roots().filter(|f| f.side != 0 && !f.alive) {
            if state.celebrated.contains(&f.id) {
                continue;
            }
            state.celebrated.push(f.id);
            let (x, y) = (f.pos.x, f.lane - 40.0);
            self.float(x, y, crate::i18n::tr("NOCAUTE!", "K.O.!").to_string(), 13.0, KO_COLOR);
            self.ring(x, y, 70.0, 0.4, 2.2, KO_COLOR);
            self.spark_burst(x, y, 1.0, 14, 3.2, KO_COLOR, 3.1);
        }
    }

    fn gate_cue(&mut self, m: &Match, state: &mut JuiceState) {
        if m.lanes.gate == state.gate {
            return;
        }
        state.gate = m.lanes.gate;
        if m.lanes.gate {
            state.celebrated.clear();
            self.ring(m.stage.bound_right - 20.0, -40.0, 120.0, 0.6, 2.4, GATE_COLOR);
            self.sfx.push("confirm");
        }
    }
}

impl Vfx {
    fn lock_cue(&mut self, m: &Match, state: &mut JuiceState) {
        let scrolling = m.is_lane_scrolling();
        let landed = state.scrolling && !scrolling && !m.lanes.gate;
        state.scrolling = scrolling;
        if !landed {
            return;
        }
        for f in m.roots().filter(|f| f.side != 0 && f.alive) {
            self.dust(f.pos.x, f.lane, 8, 1.6, 0.5, STEP_DUST);
            self.spark_burst(f.pos.x, f.lane - 30.0, 1.0, 6, 2.0, GATE_COLOR, 3.1);
        }
        for edge in [m.stage.bound_left + 12.0, m.stage.bound_right - 12.0] {
            self.ring(edge, -30.0, 90.0, 0.5, 2.0, GATE_COLOR);
        }
        self.trauma = (self.trauma + LOCK_TRAUMA).min(1.0);
        self.sfx.push("guardbreak");
    }

    fn low_life_pulse(&mut self, m: &Match) {
        if m.game_time % LOW_LIFE_EVERY_TICKS != 0 {
            return;
        }
        let hurting = m
            .roots()
            .any(|f| f.side == 0 && f.alive && f.life_max > 0.0 && f.life / f.life_max < LOW_LIFE_RATIO);
        if hurting {
            self.vig = self.vig.max(0.35);
        }
    }
}

#[path = "story_moves.rs"]
mod story_moves;

#[cfg(test)]
#[path = "story_juice_tests.rs"]
mod story_juice_tests;
