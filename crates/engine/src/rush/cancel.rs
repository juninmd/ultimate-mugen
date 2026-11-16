use super::*;
use crate::fighter::{AfterImage, Fighter};
use crate::input::k;
use crate::matchs::{Event, Match};
use crate::palette::{AfterImageParams, PalFx};

const JUMP_FALL_STATE: i32 = 50;
const CROUCH_STATE: i32 = 11;
const STAND_STATE: i32 = 0;
const TRAIL_LENGTH: usize = 8;
const FLASH_ADD: [f32; 3] = [50.0, 90.0, 170.0];

fn eligible(f: &Fighter) -> bool {
    !f.is_helper
        && !f.destroyed
        && f.alive
        && !f.ikemen.standby
        && f.move_type == 'A'
        && f.state_type != 'L'
        && f.rush.cooldown == 0
        && f.power >= COST
        && (f.move_hit > 0 || f.move_guarded > 0)
        && (1..=CONTACT_WINDOW).contains(&f.move_contact)
}

pub(super) fn landing_state(f: &Fighter) -> i32 {
    match f.state_type {
        'A' => JUMP_FALL_STATE,
        'C' => CROUCH_STATE,
        _ => STAND_STATE,
    }
}

fn trail() -> AfterImage {
    let params = AfterImageParams {
        time_gap: 2,
        frame_gap: 2,
        pal_add: [10.0, 40.0, 110.0],
        pal_mul: [0.55, 0.75, 1.0],
        ..Default::default()
    };
    AfterImage {
        time: OVERDRIVE_TICKS,
        length: TRAIL_LENGTH,
        trail: Vec::new(),
        params,
        ticks: 0,
    }
}

pub(super) fn flash() -> PalFx {
    PalFx {
        time: FLASH_TICKS,
        add: FLASH_ADD,
        ..Default::default()
    }
}

pub(super) fn swallow_start(f: &mut Fighter) {
    f.history.mask_latest(k::S);
    let (commands, active) = (&f.def.commands, &mut f.cmd.active);
    active.retain(|name| {
        !commands
            .iter()
            .any(|c| *c.name == **name && c.steps.iter().any(|s| s.btns & k::S != 0))
    });
}

impl Match {
    pub(crate) fn rush_step(&mut self) {
        for i in 0..self.fighters.len() {
            self.fighters[i].rush.tick();
            self.track_guard_hold(i);
            if self.rush_ready(i) {
                self.rush_cancel(i);
            } else if self.whiff_ready(i) {
                self.whiff_cancel(i);
            } else if let Some(target) = self.dash_cancel_target(i) {
                self.dash_cancel(i, target);
            }
        }
    }

    pub fn rush_window(&self, i: usize) -> Option<f32> {
        let f = &self.fighters[i];
        if self.round_state != 2 || self.pause.is_some() || !eligible(f) {
            return None;
        }
        Some(((CONTACT_WINDOW + 1 - f.move_contact) as f32 / CONTACT_WINDOW as f32).clamp(0.0, 1.0))
    }

    pub fn rush_ready(&self, i: usize) -> bool {
        let f = &self.fighters[i];
        f.rush.requested() && self.round_state == 2 && self.pause.is_none() && eligible(f)
    }

    fn rush_cancel(&mut self, i: usize) {
        let (side, id, guarded) = {
            let f = &self.fighters[i];
            (f.side, f.id, f.move_guarded > 0 && f.move_hit == 0)
        };
        self.add_power(i, -COST);
        let target = landing_state(&self.fighters[i]);
        let own = self.fighters[i].def.clone();
        self.change_state(i, target, own);
        let f = &mut self.fighters[i];
        f.ctrl = true;
        f.hit_pause = 0;
        f.hit_shake = 0;
        f.air_dashes = 0;
        if f.state_type != 'A' {
            f.vel.x = 0.0;
        }
        swallow_start(f);
        f.palfx = Some(flash());
        f.after_image = Some(trail());
        f.rush.begin();
        for rival in self
            .fighters
            .iter_mut()
            .filter(|r| r.side != side && !r.is_helper && r.move_type == 'H')
        {
            rival.hit_over_time += VICTIM_STUN_BONUS;
            rival.ghv.hittime += VICTIM_STUN_BONUS;
        }
        self.slow_mo = self.slow_mo.max(SLOW_MOTION_TICKS);
        self.events.push(Event::Rush {
            fighter: id,
            side,
            guarded,
        });
    }
}
