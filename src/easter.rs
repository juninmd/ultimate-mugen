use crate::audio::Audio;
use crate::ui::Ui;
use std::cell::RefCell;
use ume_engine::matchs::Match;

pub(crate) mod builders;
pub(crate) mod catalog;
mod combat;
mod draw;
pub(crate) mod sprite;
pub(crate) mod state;

pub(crate) use sprite::EggSprite;
pub(crate) use state::{ActiveEgg, EggStyle, FightTracker};

thread_local! {
    static TRACKER: RefCell<FightTracker> = RefCell::new(FightTracker::default());
}

pub fn tick(m: &Match, stage_name: &str, audio: &mut Audio) {
    TRACKER.with(|cell| {
        let mut tracker = cell.borrow_mut();
        tracker.sync_match(m);
        combat::check(m, stage_name, &mut tracker, audio);
        tracker.record_snapshot(m);
    });
}

pub fn draw(ui: &Ui, dt: f32) {
    TRACKER.with(|cell| {
        let mut tracker = cell.borrow_mut();
        tracker.advance(dt);
        draw::render_all(ui, &tracker.active);
    });
}
