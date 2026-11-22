use crate::audio::Audio;
use crate::ui::Ui;
use std::cell::RefCell;
use ume_engine::matchs::Match;

pub(crate) mod builders;
pub(crate) mod catalog;
mod combat;
mod draw;
pub(crate) mod sprite;
mod stages;
mod stages_catalog;
pub(crate) mod state;
pub(crate) mod toml_config;
mod toml_eval;
pub(crate) mod toml_exchange;

pub(crate) use sprite::EggSprite;
pub(crate) use state::{ActiveEgg, EggStyle, FightTracker};

thread_local! {
    static TRACKER: RefCell<FightTracker> = RefCell::new(FightTracker::default());
}

pub fn tick(m: &Match, stage_name: &str, audio: &mut Audio) {
    TRACKER.with(|cell| {
        let mut tracker = cell.borrow_mut();
        tracker.sync_match(m);
        tracker.flush_pending(audio);
        combat::check(m, stage_name, &mut tracker, audio);
        stages::check(m, stage_name, &mut tracker, audio);
        toml_eval::check(m, stage_name, &mut tracker, audio);
        tracker.record_snapshot(m);
    });
}

pub fn draw(ui: &Ui, dt: f32, ribbon_top: f32) {
    TRACKER.with(|cell| {
        let mut tracker = cell.borrow_mut();
        tracker.advance(dt);
        draw::render_all(ui, &tracker.active, ribbon_top);
    });
}
