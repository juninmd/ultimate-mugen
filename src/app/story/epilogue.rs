use crate::app::{App, Screen};
use crate::controls::nav;
use crate::gfx::*;
use crate::story::epilogue::Epilogue;
use crate::story::{epilogue, epilogue_music};
use std::borrow::Cow;
use crate::story::{Phase, StoryRun};

const INPUT_DELAY: f32 = 0.25;
const TEXT_REVEALED: f32 = 99.0;

impl App {
    pub(crate) fn story_epilogue(&mut self, run: &StoryRun, t: f32, dt: f32) -> bool {
        if run.phase != Phase::Victory {
            return false;
        }
        let player = self.player_char_id();
        let Some(script) = self.epilogue_script(&player, run.mercy) else {
            return false;
        };
        let beats = script.beats(crate::profile::lang());
        if run.epilogue > beats.len() {
            return false;
        }
        let boss = self.story_char(epilogue::final_boss(&player));
        let revealed = self.draw_story_epilogue(&script, &beats, run.epilogue, t, boss);
        if self.skip_chip() || self.c.pressed(nav::BACK | nav::START) || is_key_pressed(KeyCode::Escape) {
            self.story_epilogue_goto(beats.len() + 1);
            return true;
        }
        let advance = self.c.pressed(nav::OK) || self.tap_any() || is_key_pressed(KeyCode::Space);
        if advance && t > INPUT_DELAY {
            if revealed {
                self.audio.sfx("select");
                self.story_epilogue_goto(run.epilogue + 1);
            } else {
                self.screen = Screen::Story { t: TEXT_REVEALED };
            }
            return true;
        }
        self.screen = Screen::Story { t: t + dt };
        true
    }

    pub(crate) async fn story_epilogue_music(&mut self, run: &StoryRun, t: f32) {
        if run.phase != Phase::Victory || t > 0.0 {
            return;
        }
        let player = self.player_char_id();
        let Some(script) = self.epilogue_script(&player, run.mercy) else {
            return;
        };
        if run.epilogue == 0 {
            let track = epilogue_music::wav();
            self.audio
                .play_music_file(&track, crate::audio::MusicLoop::default())
                .await;
        } else if run.epilogue > script.beats(crate::profile::lang()).len() {
            self.audio.stop_music();
        }
    }

    fn epilogue_script(&self, player: &str, mercy: Option<bool>) -> Option<Cow<'static, Epilogue>> {
        let name = self.reg.chars.get(self.sel.0).map_or("", |entry| entry.name.as_str());
        epilogue::of_named(player, name, mercy)
    }

    pub(crate) fn story_epilogue_page(&self) -> Option<usize> {
        self.story
            .as_ref()
            .filter(|run| run.phase == Phase::Victory)
            .map(|run| run.epilogue)
    }

    pub(crate) fn story_epilogue_goto(&mut self, page: usize) {
        if let Some(run) = self.story.as_mut() {
            run.epilogue = page;
        }
        reset_textures();
        self.screen = Screen::Story { t: 0.0 };
    }

    pub(crate) fn story_epilogue_begin(&mut self, hero: usize, page: usize) {
        let mut run = StoryRun::default();
        run.stage = crate::story::STAGES.len() - 1;
        run.wave = run.stage_def().waves.len();
        run.won_fight(0, 1.0);
        self.mode = crate::app::Mode::Story;
        self.sel.0 = hero;
        let _ = self.reg.chars[hero].load(1);
        self.session = None;
        self.story = Some(run);
        self.story_epilogue_goto(page);
    }
}
