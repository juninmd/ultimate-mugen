use crate::app::{App, Screen};
use crate::controls::nav;
use crate::gfx::*;
use crate::story::{text, Phase};
use crate::ui::*;

impl App {
    pub(crate) fn story_screen(&mut self, dt: f32) {
        let Screen::Story { t } = self.screen else { return };
        let Some(run) = self.story.clone() else {
            self.screen = Screen::Menu { cur: 0 };
            return;
        };
        let accent = run.stage_def().accent;
        let accent_color = Color::new(accent[0], accent[1], accent[2], 1.0);
        self.ui.background();
        self.ui.motes(40, 1.2);
        let reveal = (t * 2.5).min(1.0);
        self.ui.rect(0.0, 230.0, VW, 220.0, Color::new(0.02, 0.01, 0.06, 0.78 * reveal));
        self.ui.rect(0.0, 230.0, VW, 4.0, accent_color);
        self.ui.rect(0.0, 446.0, VW, 4.0, accent_color);
        let (headline, sub, line) = match run.phase {
            Phase::Intro => (
                text::stage_label(run.stage),
                text::stage_title(run.stage).to_string(),
                text::stage_line(run.stage).to_string(),
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
        self.ui.text(&line, VW / 2.0, 410.0, 20.0, hex("#d8d0f0"), Align::Center, false);
        let prompt = match run.phase {
            Phase::Defeated if run.can_continue() => text::continue_label(),
            Phase::Defeated | Phase::Victory => text::quit_label(),
            _ => text::press_start(),
        };
        let pulse = 0.65 + 0.35 * (t * 4.0).sin().abs();
        self.ui.text(prompt, VW / 2.0, 520.0, 22.0, Color::new(1.0, 1.0, 1.0, pulse), Align::Center, true);
        let confirmed = self.c.pressed(nav::OK | nav::START) && t > 0.35;
        if confirmed || self.clicked_rect(0.0, 230.0, VW, 220.0) {
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
}
