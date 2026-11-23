mod animation;
mod draw;
mod load;
mod player;
mod screen;
mod sprite_draw;

pub use player::{After, Playback};

use super::super::*;

impl App {
    pub(crate) fn start_route_storyboard_slot(&mut self, slot: usize, after: After) -> bool {
        loop {
            let path = self
                .arcade
                .as_mut()
                .and_then(|run| run.take_storyboard(slot));
            let Some(path) = path else {
                return false;
            };
            match load::load(self, &path) {
                Ok(playback) => {
                    let scene = playback.storyboard.start_scene;
                    if let Some(run) = self.arcade.as_mut() {
                        run.storyboard_player = Some(playback);
                        run.storyboard_after = Some(after);
                    }
                    self.audio.stop_music();
                    self.screen = Screen::Cutscene {
                        kind: 6,
                        page: scene,
                        t: 0.0,
                    };
                    return true;
                }
                Err(error) => eprintln!("arcade storyboard '{path}': {error}"),
            }
        }
    }

    pub(crate) fn route_storyboard_screen(&mut self) {
        screen::tick(self);
    }

    pub(crate) fn route_storyboard_done(&mut self) {
        let Some((slot, after)) = self.arcade.as_mut().and_then(|run| {
            run.storyboard_player = None;
            run.storyboard_after
                .take()
                .map(|after| (after.slot(run.ladder.len()), after))
        }) else {
            return;
        };
        if self.start_route_storyboard_slot(slot, after) {
            return;
        }
        after.continue_app(self);
    }
}
