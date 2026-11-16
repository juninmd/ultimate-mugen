use crate::app::{App, Screen};
use crate::controls::nav;
use crate::gfx::*;
use crate::story::{beats, choice, text, Phase};
use crate::ui::*;

impl App {
    pub(crate) async fn story_screen(&mut self, dt: f32) {
        let Screen::Story { t } = self.screen else { return };
        let Some(run) = self.story.clone() else {
            self.screen = Screen::Menu { cur: 0 };
            return;
        };
        self.story_epilogue_music(&run, t).await;
        if self.story_epilogue(&run, t, dt) {
            return;
        }
        let deciding = choice::pending(&run);
        if deciding {
            if self.c.pressed(nav::LEFT) {
                self.decide_mercy(true);
            } else if self.c.pressed(nav::RIGHT) {
                self.decide_mercy(false);
            }
        }
        let accent = run.stage_def().accent;
        let accent_color = Color::new(accent[0], accent[1], accent[2], 1.0);
        self.ui.background();
        self.ui.motes(40, 1.2);
        let reveal = (t * 2.5).min(1.0);
        self.ui.adapted_glass(-20.0, 230.0, VW + 40.0, 220.0, accent_color, reveal);
        self.ui.rect(0.0, 230.0, VW * reveal, 3.0, accent_color);
        let (headline, sub, line) = match run.phase {
            Phase::Intro => (
                text::stage_label(run.stage),
                text::stage_title(run.stage).to_string(),
                if deciding {
                    choice::prompt().to_string()
                } else {
                    beats::stage_beat(run.stage, beats::beat_index(t)).to_string()
                },
            ),
            Phase::Cleared => (
                text::stage_clear().to_string(),
                text::score_line(run.score),
                text::stage_title(run.stage.saturating_sub(1)).to_string(),
            ),
            Phase::Victory => (
                text::story_clear().to_string(),
                text::score_line(run.score),
                String::new(),
            ),
            Phase::Defeated => (
                text::game_over().to_string(),
                text::score_line(run.score),
                if run.can_continue() {
                    text::credits_line(run.credits)
                } else {
                    String::new()
                },
            ),
        };
        self.ui.text(&headline, VW / 2.0, 296.0, 26.0, accent_color, Align::Center, true);
        self.ui.text_bold(&sub, VW / 2.0, 360.0, 56.0, WHITE, BLACK, Align::Center);
        let line_color = if run.phase == Phase::Intro {
            Color::new(0.847, 0.816, 0.941, beats::beat_alpha(t))
        } else {
            hex("#d8d0f0")
        };
        self.ui.text(&line, VW / 2.0, 410.0, 20.0, line_color, Align::Center, false);
        if deciding {
            self.ui.text(choice::hint(), VW / 2.0, 470.0, 20.0, WHITE, Align::Center, false);
            self.ui.text(choice::spare_label(), VW * 0.3, 510.0, 26.0, hex("#8fe3a8"), Align::Center, true);
            self.ui.text(choice::finish_label(), VW * 0.7, 510.0, 26.0, hex("#ff7a6b"), Align::Center, true);
        } else if run.phase == Phase::Intro {
            self.draw_story_boss_profile(&run, accent_color, reveal);
        }
        if let (Phase::Cleared, Some(tally)) = (run.phase, run.tally) {
            self.ui.text(&text::tally_line(&tally), VW / 2.0, 480.0, 20.0, hex("#ffcc33"), Align::Center, true);
        }
        let prompt = match run.phase {
            _ if deciding => "",
            Phase::Defeated if run.can_continue() => text::continue_label(),
            Phase::Defeated | Phase::Victory => text::quit_label(),
            _ => text::press_start(),
        };
        let pulse = 0.65 + 0.35 * (t * 4.0).sin().abs();
        self.ui.text(prompt, VW / 2.0, 520.0, 22.0, Color::new(1.0, 1.0, 1.0, pulse), Align::Center, true);
        let confirmed = self.c.pressed(nav::OK | nav::START) && t > 0.35;
        if !deciding && (confirmed || self.clicked_rect(0.0, 230.0, VW, 220.0)) {
            self.audio.sfx("confirm");
            self.story_resume();
            return;
        }
        if self.c.pressed(nav::BACK) || is_key_pressed(KeyCode::Escape) {
            self.story_quit();
            return;
        }
        self.screen = Screen::Story { t: t + dt };
    }

    fn decide_mercy(&mut self, mercy: bool) {
        if let Some(run) = self.story.as_mut() {
            run.mercy = Some(mercy);
        }
        self.audio.sfx("select");
    }
}
