use crate::app::{App, Screen};
use crate::controls::nav;
use crate::gfx::*;
use crate::story::{beats, choice, helper, prefight, text, Phase};
use crate::ui::*;

impl App {
    fn story_prompt(&self, prompt: &str, accent: Color) -> bool {
        let (x, y, w, h) = (VW / 2.0 - 224.0, 488.0, 448.0, 48.0);
        let hover = self.hover_rect(x, y, w, h);
        self.ui.adapted_plate(x, y, w, h, hover.then_some(accent));
        self.ui.text(prompt, VW / 2.0, y + 31.0, 22.0, WHITE, Align::Center, true);
        self.tapped_in(x, y, w, h)
    }

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
        let talking = prefight::pending(&run);
        let choosing_helper = !talking && helper::pending(&run);
        let deciding = !talking && (choice::pending(&run) || choosing_helper);
        if choosing_helper {
            let confirmed = self.c.pressed(nav::OK | nav::START) && t > 0.35;
            self.story_helper_input(&run, confirmed);
        } else if deciding {
            if self.c.pressed(nav::LEFT) || self.tapped_in(VW * 0.18, 484.0, VW * 0.24, 56.0) {
                self.decide_mercy(true);
            } else if self.c.pressed(nav::RIGHT) || self.tapped_in(VW * 0.58, 484.0, VW * 0.24, 56.0) {
                self.decide_mercy(false);
            }
        }
        let accent = run.stage_accent();
        let accent_color = Color::new(accent[0], accent[1], accent[2], 1.0);
        crate::screen_art::ScreenArt::Story.background(&self.ui);
        self.ui.motes(40, 1.2);
        let reveal = (t * 2.5).min(1.0);
        self.ui.adapted_glass(-20.0, 230.0, VW + 40.0, 220.0, accent_color, reveal);
        self.ui.rect(0.0, 230.0, VW * reveal, 3.0, accent_color);
        let campaign = text::campaign_title();
        if !campaign.is_empty() && run.phase == Phase::Intro {
            self.ui.text(
                campaign,
                VW / 2.0,
                214.0,
                16.0,
                Color::new(accent[0], accent[1], accent[2], 0.85 * reveal),
                Align::Center,
                true,
            );
        }
        let (headline, sub, line) = match run.phase {
            Phase::Intro => (
                text::stage_label(run.stage),
                text::stage_title(run.stage).to_string(),
                if talking {
                    self.story_prefight_line(&run)
                } else if choosing_helper {
                    helper::prompt().to_string()
                } else if deciding {
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
                crate::story::rank::line(crate::story::rank::of_run(&run)),
            ),
            Phase::Defeated => (
                if run.can_continue() { text::knocked_out() } else { text::game_over() }.to_string(),
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
        if choosing_helper {
            self.draw_story_helper_pick(&run, accent_color);
        } else if deciding {
            self.ui.text(choice::hint(), VW / 2.0, 470.0, 20.0, WHITE, Align::Center, false);
            self.ui.text(choice::spare_label(), VW * 0.3, 510.0, 26.0, hex("#8fe3a8"), Align::Center, true);
            self.ui.text(choice::finish_label(), VW * 0.7, 510.0, 26.0, hex("#ff7a6b"), Align::Center, true);
        } else if run.phase == Phase::Intro {
            self.draw_story_boss_profile(&run, accent_color, reveal);
        }
        if run.phase == Phase::Defeated {
            self.ui.text(crate::story::tips::of(run.stage), VW / 2.0, 470.0, 18.0, hex("#ffcc33"), Align::Center, false);
        }
        if let (Phase::Cleared, Some(tally)) = (run.phase, run.tally) {
            self.ui.text(&text::tally_line(&tally), VW / 2.0, 480.0, 20.0, hex("#ffcc33"), Align::Center, true);
        }
        let prompt = match run.phase {
            _ if talking => crate::i18n::tr3("OK para avançar", "OK to advance", "OK para avanzar"),
            _ if deciding => "",
            Phase::Defeated if run.can_continue() => text::continue_with_credit(),
            Phase::Defeated | Phase::Victory => text::quit_label(),
            _ => text::press_start(),
        };
        let confirmed = self.c.pressed(nav::OK | nav::START) && t > 0.35;
        let advances = !deciding && (self.story_prompt(prompt, accent_color) || confirmed
            || self.tapped_in(0.0, 230.0, VW, 220.0));
        if talking && advances {
            self.audio.sfx("select");
            if let Some(run) = self.story.as_mut() {
                run.prefight += 1;
            }
            self.screen = Screen::Story { t: 0.5 };
            return;
        }
        if !talking && advances {
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
