use crate::app::{App, Mode, Screen};
use crate::controls::nav;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::story::checkpoint::Checkpoint;
use crate::story::{text, StoryRun};
use crate::ui::*;

impl App {
    pub(crate) fn story_begin(&mut self, me: usize) {
        self.enter_story(me, StoryRun::default());
    }

    fn enter_story(&mut self, me: usize, run: StoryRun) {
        self.story = Some(run);
        self.survival = None;
        self.arcade = None;
        self.sel.0 = me;
        self.ending_shown = false;
        self.story_prepare_fight();
        self.screen = Screen::Story { t: 0.0 };
    }

    pub(crate) fn open_story_checkpoint(&mut self) -> bool {
        let Some(checkpoint) = Checkpoint::load() else {
            return false;
        };
        let Some(hero) = self
            .reg
            .chars
            .iter()
            .position(|entry| entry.key == checkpoint.fighter && !entry.broken.get())
        else {
            return false;
        };
        self.go(Screen::StoryCheckpoint {
            checkpoint,
            hero,
            resume: true,
        });
        true
    }

    pub(crate) fn story_checkpoint_menu(&mut self) {
        let Screen::StoryCheckpoint {
            checkpoint,
            hero,
            mut resume,
        } = &mut self.screen
        else {
            return;
        };
        let hero = *hero;
        let Some(run) = checkpoint.restore() else {
            self.go(Screen::Menu { cur: 0 });
            return;
        };
        let palette = checkpoint.palette;
        if self.c.pressed(nav::BACK) {
            self.go(Screen::Menu { cur: 0 });
            return;
        }
        if self.c.pressed(nav::LEFT | nav::RIGHT) {
            resume = !resume;
            self.audio.sfx("select");
        }
        crate::screen_art::ScreenArt::Story.background(&self.ui);
        let accent = hex("#bb80f0");
        self.ui
            .adapted_glass(260.0, 166.0, 760.0, 440.0, accent, 1.0);
        self.ui.text_bold(
            tr3("História", "Story", "Historia"),
            640.0,
            238.0,
            42.0,
            WHITE,
            BLACK,
            Align::Center,
        );
        self.ui.text(
            &self.reg.chars[hero].name,
            640.0,
            292.0,
            28.0,
            accent,
            Align::Center,
            true,
        );
        self.ui.text(
            text::stage_title(run.stage),
            640.0,
            340.0,
            24.0,
            WHITE,
            Align::Center,
            true,
        );
        self.ui.text(
            &text::score_line(run.score),
            640.0,
            382.0,
            19.0,
            WHITE,
            Align::Center,
            false,
        );
        self.ui.text(
            &text::credits_line(run.credits),
            640.0,
            414.0,
            19.0,
            WHITE,
            Align::Center,
            false,
        );
        let resume_tap = self.story_checkpoint_option(
            300.0,
            tr3("Retomar", "Resume", "Continuar"),
            resume,
            accent,
        );
        let new_tap = self.story_checkpoint_option(
            660.0,
            tr3("Nova campanha", "New campaign", "Nueva campaña"),
            !resume,
            accent,
        );

        if !resume_tap && !new_tap && !self.c.pressed(nav::OK | nav::START) {
            if let Screen::StoryCheckpoint {
                resume: selection, ..
            } = &mut self.screen
            {
                *selection = resume;
            }
            return;
        }

        self.audio.sfx("confirm");
        self.set_mode(Mode::Story);
        if resume_tap || (!new_tap && resume) {
            self.pal[0] = palette;
            self.enter_story(hero, run);
        } else {
            self.go(Screen::Select {
                cur: [self.sel.0, self.sel.1],
                picking: 0,
                picked: [None, None],
            });
        }
    }

    fn story_checkpoint_option(&self, x: f32, label: &str, selected: bool, accent: Color) -> bool {
        let (y, w, h) = (486.0, 320.0, 66.0);
        let active = selected || self.hover_rect(x, y, w, h);
        self.ui.adapted_plate(x, y, w, h, active.then_some(accent));
        self.ui.text(
            label,
            x + w * 0.5,
            y + 41.0,
            24.0,
            WHITE,
            Align::Center,
            true,
        );
        self.tapped_in(x, y, w, h)
    }
}
