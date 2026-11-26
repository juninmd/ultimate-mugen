mod beat;
mod config;
mod ease;
mod rig;
#[cfg(test)]
mod tests;

use ume_engine::matchs::Match;

pub use beat::{Sample, Subject};
pub use rig::Rig;

use beat::sample;
use config::{config, Config};

const TICKS_PER_SECOND: f32 = 60.0;

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub subject: Subject,
    pub sample: Sample,
}

impl Frame {
    pub fn covers(&self, side: usize, winner_side: i32) -> bool {
        match self.subject {
            Subject::Both => true,
            Subject::P1 => side == 0,
            Subject::P2 => side == 1,
            Subject::Winner => side as i32 == winner_side,
        }
    }
}

pub fn frame_with(m: &Match, cfg: &Config) -> Option<Frame> {
    if !cfg.enabled || m.training {
        return None;
    }
    let seconds = m.round_time.max(0) as f32 / TICKS_PER_SECOND;
    let beats = match m.round_state {
        1 if m.round_no == 1 && m.intro_ticks > 0 && !m.intro_skipped => &cfg.intro,
        4 if m.winner_side >= 0 => &cfg.victory,
        _ => return None,
    };
    sample(beats, seconds).map(|(subject, sample)| Frame { subject, sample })
}

pub fn frame(m: &Match) -> Option<Frame> {
    frame_with(m, config())
}

pub fn view_for(m: &Match, side: usize) -> Sample {
    if crate::tweaks::current().reduce_motion {
        return Sample::default();
    }
    frame(m)
        .filter(|f| f.covers(side, m.winner_side))
        .map_or_else(Sample::default, |f| f.sample)
}

const CHEST: f32 = 0.55;

pub fn close_up(scale: &mut f32, feet_y: &mut f32, height: f32, view: &Sample) {
    if view.detail <= 0.0 {
        return;
    }
    let zoom = 1.0 + view.detail;
    let full = height * *scale;
    let lean = (view.detail * 2.0).min(1.0);
    let hold = view.aim * (1.0 - lean) + CHEST * lean;
    *feet_y += (view.aim * zoom - hold) * full;
    *scale *= zoom;
}
